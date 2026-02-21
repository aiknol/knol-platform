// =============================================================================
// Knol Marketing — Content Publisher
// Routes generated content to the appropriate channel adapter and publishes
// =============================================================================

const fs = require('fs');
const path = require('path');

const twitter = require('../channels/twitter');
const linkedin = require('../channels/linkedin');
const reddit = require('../channels/reddit');
const devto = require('../channels/devto');
const hackernews = require('../channels/hackernews');
const github = require('../channels/github');
const email = require('../channels/email');
const blog = require('../channels/blog');
const limiter = require('./limit-intelligence');

const LOG_DIR = path.join(__dirname, '..', 'data');
const LOG_FILE = path.join(LOG_DIR, 'publish-log.json');

// Load credentials from environment
function loadCredentials() {
  return {
    twitter: {
      apiKey: process.env.TWITTER_API_KEY,
      apiSecret: process.env.TWITTER_API_SECRET,
      accessToken: process.env.TWITTER_ACCESS_TOKEN,
      accessTokenSecret: process.env.TWITTER_ACCESS_TOKEN_SECRET,
    },
    linkedin: {
      accessToken: process.env.LINKEDIN_ACCESS_TOKEN,
      personUrn: process.env.LINKEDIN_PERSON_URN,
    },
    reddit: {
      clientId: process.env.REDDIT_CLIENT_ID,
      clientSecret: process.env.REDDIT_CLIENT_SECRET,
      username: process.env.REDDIT_USERNAME,
      password: process.env.REDDIT_PASSWORD,
    },
    devto: {
      apiKey: process.env.DEVTO_API_KEY,
    },
    github: {
      token: process.env.GITHUB_TOKEN,
    },
    email: {
      smtpHost: process.env.SMTP_HOST,
      smtpPort: process.env.SMTP_PORT || '587',
      smtpUser: process.env.SMTP_USER,
      smtpPass: process.env.SMTP_PASS,
    },
  };
}

function hasApiCredentials(channel, credentials) {
  switch (channel) {
    case 'twitter':
      return Boolean(
        credentials?.twitter?.apiKey &&
        credentials?.twitter?.apiSecret &&
        credentials?.twitter?.accessToken &&
        credentials?.twitter?.accessTokenSecret
      );
    case 'linkedin':
      return Boolean(credentials?.linkedin?.accessToken && credentials?.linkedin?.personUrn);
    case 'reddit':
    case 'reddit_engagement':
      return Boolean(
        credentials?.reddit?.clientId &&
        credentials?.reddit?.clientSecret &&
        credentials?.reddit?.username &&
        credentials?.reddit?.password
      );
    case 'devto':
      return Boolean(credentials?.devto?.apiKey);
    case 'github':
      return Boolean(credentials?.github?.token);
    case 'email':
      return Boolean(
        credentials?.email?.smtpHost &&
        credentials?.email?.smtpUser &&
        credentials?.email?.smtpPass
      );
    default:
      return false;
  }
}

function endpointForChannel(channel, content) {
  switch (channel) {
    case 'twitter':
      return Array.isArray(content?.tweets) ? 'thread_create' : 'tweet_create';
    case 'linkedin':
      return content?.articleUrl ? 'article_share' : 'ugc_create';
    case 'reddit':
      return 'submit_post';
    case 'reddit_engagement':
      return 'engage_comment';
    case 'devto':
      return 'article_create';
    case 'github':
      return content?.type === 'release' ? 'release_create' : 'repo_update';
    case 'email':
      return 'newsletter_send';
    case 'blog':
      return 'blog_publish';
    default:
      return 'publish';
  }
}

function cloneContent(content) {
  if (content == null) return {};
  try {
    return JSON.parse(JSON.stringify(content));
  } catch {
    return typeof content === 'object' ? { ...content } : { text: String(content) };
  }
}

function splitUrlSuffix(url) {
  const m = String(url).match(/^(.+?)([),.!?;:]*)$/);
  return m ? { core: m[1], suffix: m[2] } : { core: String(url), suffix: '' };
}

function appendUtm(urlString, attribution) {
  const { core, suffix } = splitUrlSuffix(urlString);
  let url;
  try {
    url = new URL(core);
  } catch {
    return urlString;
  }

  const params = url.searchParams;
  params.set('utm_source', attribution.utmSource);
  params.set('utm_medium', attribution.utmMedium);
  params.set('utm_campaign', attribution.utmCampaign);
  if (attribution.utmContent) params.set('utm_content', attribution.utmContent);
  url.search = params.toString();
  return `${url.toString()}${suffix}`;
}

function rewriteTextUrls(text, attribution, stats) {
  if (typeof text !== 'string') return text;
  const urlRegex = /https?:\/\/[^\s)]+/g;
  return text.replace(urlRegex, (url) => {
    stats.urls += 1;
    const rewritten = appendUtm(url, attribution);
    if (rewritten !== url) stats.utmUrls += 1;
    return rewritten;
  });
}

function buildAttribution(meta, channel) {
  const cadence = meta?.cadence || 'ad_hoc';
  const taskId = meta?.taskId || 'manual';
  const variantId = meta?.variantId || '';
  return {
    utmSource: channel,
    utmMedium: 'free_promo',
    utmCampaign: `${cadence}_${taskId}`.replace(/[^a-zA-Z0-9_-]/g, '_'),
    utmContent: variantId || undefined,
    cadence,
    taskId,
    variantId,
    templateCategory: meta?.templateCategory || '',
    variantIndex: Number.isFinite(meta?.variantIndex) ? meta.variantIndex : null,
  };
}

function applyAttribution(content, channel) {
  const working = cloneContent(content);
  const meta = working.__meta || {};
  const attribution = buildAttribution(meta, channel);
  const stats = { urls: 0, utmUrls: 0 };

  if (typeof working.text === 'string') working.text = rewriteTextUrls(working.text, attribution, stats);
  if (typeof working.body === 'string') working.body = rewriteTextUrls(working.body, attribution, stats);
  if (typeof working.htmlContent === 'string') working.htmlContent = rewriteTextUrls(working.htmlContent, attribution, stats);
  if (typeof working.url === 'string') {
    stats.urls += 1;
    const rewritten = appendUtm(working.url, attribution);
    if (rewritten !== working.url) stats.utmUrls += 1;
    working.url = rewritten;
  }
  if (typeof working.canonicalUrl === 'string') {
    stats.urls += 1;
    const rewritten = appendUtm(working.canonicalUrl, attribution);
    if (rewritten !== working.canonicalUrl) stats.utmUrls += 1;
    working.canonicalUrl = rewritten;
  }
  if (Array.isArray(working.tweets)) {
    working.tweets = working.tweets.map((t) => rewriteTextUrls(t, attribution, stats));
  }

  working.__meta = { ...meta, ...attribution };
  working.__tracking = {
    urlsDetected: stats.urls,
    utmUrlsApplied: stats.utmUrls,
  };
  return working;
}

function summarizeContent(content) {
  return {
    text: typeof content?.text === 'string' ? content.text.slice(0, 240) : undefined,
    title: typeof content?.title === 'string' ? content.title.slice(0, 140) : undefined,
    subreddit: content?.subreddit,
    kind: content?.kind,
    urlsDetected: content?.__tracking?.urlsDetected || 0,
    utmUrlsApplied: content?.__tracking?.utmUrlsApplied || 0,
  };
}

function extractRetryAndReset(result) {
  const headers = result?.headers || {};
  const retryAfter = headers['retry-after'] || headers['Retry-After'];
  const reset = headers['x-rate-limit-reset'] || headers['x-ratelimit-reset'] || headers['X-RateLimit-Reset'];
  return {
    retryAfterSeconds: retryAfter ? parseInt(String(retryAfter), 10) : null,
    resetEpochSeconds: reset ? parseInt(String(reset), 10) : null,
  };
}

// Publish to a specific channel
async function publishToChannel(channel, content, credentials) {
  const result = { channel, timestamp: new Date().toISOString(), content: {} };
  const normalized = cloneContent(content);
  const contentForPublish = applyAttribution(normalized, channel);
  const policyChannel = channel === 'reddit_engagement' ? 'reddit' : channel;
  result.content = summarizeContent(contentForPublish);
  result.attribution = contentForPublish.__meta || {};
  result.tracking = contentForPublish.__tracking || {};

  const endpoint = endpointForChannel(channel, contentForPublish);
  const useApi = hasApiCredentials(channel, credentials);
  const preflight = useApi ? limiter.shouldAllowPublish(policyChannel, endpoint, normalized) : { allowed: true, reason: 'manual_or_no_api' };

  if (!preflight.allowed) {
    result.success = false;
    result.deferred = true;
    result.data = {
      error: `Deferred by policy: ${preflight.reason}`,
      reason: preflight.reason,
      waitSeconds: preflight.waitSeconds || 0,
      nextAllowedAt: preflight.nextAllowedAt || null,
    };
    logPublish(result);
    return result;
  }

  if (useApi && preflight.hash) {
    limiter.reservePublish(policyChannel, preflight.hash);
  }

  try {
    switch (channel) {
      case 'twitter': {
        if (Array.isArray(contentForPublish.tweets)) {
          // Thread
          const results = await twitter.postThread(contentForPublish.tweets, credentials.twitter);
          result.success = results.every(r => r.success);
          result.data = results;
        } else {
          result.data = await twitter.postTweet(contentForPublish.text, credentials.twitter);
          result.success = result.data.success;
        }
        break;
      }

      case 'linkedin': {
        if (contentForPublish.articleUrl) {
          result.data = await linkedin.postArticle(contentForPublish.title, contentForPublish.text, contentForPublish.articleUrl, credentials.linkedin);
        } else {
          result.data = await linkedin.postUpdate(contentForPublish.text, credentials.linkedin);
        }
        result.success = result.data.success;
        break;
      }

      case 'reddit': {
        const auth = await reddit.authenticate(credentials.reddit);
        if (!auth.success) {
          result.success = false;
          result.data = { error: `Auth failed: ${auth.error}` };
          break;
        }

        result.data = await reddit.submitPost(
          contentForPublish.subreddit,
          contentForPublish.title,
          contentForPublish.text,
          auth.token,
          contentForPublish.kind || 'self'
        );
        result.success = result.data.success;
        break;
      }

      case 'reddit_engagement': {
        const auth = await reddit.authenticate(credentials.reddit);
        if (!auth.success) {
          result.success = false;
          result.data = { error: `Auth failed: ${auth.error}` };
          break;
        }

        const opportunities = await reddit.findEngagementOpportunities(auth.token, {
          query: contentForPublish.query,
          subreddits: contentForPublish.subreddits,
          limitPerSub: contentForPublish.limitPerSub,
          limit: contentForPublish.limit,
          minScore: contentForPublish.minScore,
          minComments: contentForPublish.minComments,
        });

        if (!opportunities.success) {
          result.success = false;
          result.data = { error: opportunities.error || 'Failed to load opportunities' };
          break;
        }

        const maxAutoReplies = Number.parseInt(process.env.REDDIT_ENGAGE_MAX_AUTO || '1', 10);
        const autoReply = process.env.REDDIT_ENGAGE_AUTOREPLY === 'true';
        const commentTemplate = contentForPublish.commentTemplate || '';
        const comments = [];

        if (autoReply && commentTemplate && Array.isArray(opportunities.opportunities)) {
          for (const opp of opportunities.opportunities.slice(0, Math.max(0, maxAutoReplies))) {
            if (!opp?.name) continue;
            const commentText = commentTemplate.replaceAll('{{title}}', opp.title || '');
            const posted = await reddit.postComment(opp.name, commentText, auth.token);
            comments.push({ thread: opp.permalink || opp.url, success: posted.success, statusCode: posted.statusCode });
          }
        }

        result.success = true;
        result.data = {
          manual: !autoReply,
          opportunities: opportunities.opportunities || [],
          comments,
        };
        break;
      }

      case 'devto': {
        result.data = await devto.publishArticle({
          title: contentForPublish.title,
          body: contentForPublish.body || contentForPublish.text,
          tags: contentForPublish.tags,
          series: contentForPublish.series,
          canonicalUrl: contentForPublish.canonicalUrl,
        }, credentials.devto);
        result.success = result.data.success;
        break;
      }

      case 'hackernews': {
        // HN is manual — generate submission content
        const showHN = hackernews.generateShowHN();
        result.data = { ...showHN, manual: true };
        result.success = true;
        break;
      }

      case 'github': {
        if (contentForPublish.type === 'release') {
          result.data = await github.createRelease(
            contentForPublish.tag, contentForPublish.name, contentForPublish.body, credentials.github
          );
        } else if (contentForPublish.type === 'metadata') {
          result.data = await github.updateRepoMetadata(
            contentForPublish.description, contentForPublish.topics, credentials.github
          );
        } else {
          result.data = await github.getRepoStats(credentials.github);
        }
        result.success = result.data?.success ?? true;
        break;
      }

      case 'email': {
        const htmlBody = email.generateNewsletterHtml(contentForPublish.htmlContent || contentForPublish.text);
        const textBody = contentForPublish.text;
        result.data = await email.sendNewsletter(contentForPublish.subject, htmlBody, textBody, credentials.email);
        result.success = result.data.sent > 0 || result.data.manual;
        break;
      }

      case 'blog': {
        const post = blog.createPost(contentForPublish.title, contentForPublish.body || contentForPublish.text, contentForPublish.tags);
        result.data = post;
        result.success = post.success;
        break;
      }

      default:
        result.success = false;
        result.data = { error: `Unknown channel: ${channel}` };
    }
  } catch (e) {
    result.success = false;
    result.data = { error: e.message, stack: e.stack };
  }

  if (useApi) {
    if (result?.data?.headers) {
      limiter.updateFromHeaders(policyChannel, endpoint, result.data.headers);
    }
    if (result?.data?.rateLimited || result?.rateLimited || result?.data?.statusCode === 429) {
      const retryMeta = extractRetryAndReset(result.data || result);
      limiter.markRateLimited(policyChannel, endpoint, retryMeta);
    }
  }

  // Log result
  logPublish(result);
  return result;
}

// Publish to multiple channels
async function publishToAll(contentMap, options = {}) {
  const credentials = loadCredentials();
  const results = {};
  const { dryRun = false, sequential = false } = options;

  const channels = Object.keys(contentMap);

  if (dryRun) {
    console.log('[DRY RUN] Would publish to:', channels.join(', '));
    for (const [ch, content] of Object.entries(contentMap)) {
      console.log(`  ${ch}:`, JSON.stringify(content).substring(0, 100));
    }
    return { dryRun: true, channels };
  }

  if (sequential) {
    for (const ch of channels) {
      results[ch] = await publishToChannel(ch, contentMap[ch], credentials);
      // 2s delay between channels to be nice to APIs
      await new Promise(r => setTimeout(r, 2000));
    }
  } else {
    const promises = channels.map(ch =>
      publishToChannel(ch, contentMap[ch], credentials).then(r => [ch, r])
    );
    const settled = await Promise.allSettled(promises);
    for (const s of settled) {
      if (s.status === 'fulfilled') {
        const [ch, result] = s.value;
        results[ch] = result;
      }
    }
  }

  return results;
}

// Logging
function logPublish(result) {
  if (!fs.existsSync(LOG_DIR)) fs.mkdirSync(LOG_DIR, { recursive: true });

  let log = [];
  try { log = JSON.parse(fs.readFileSync(LOG_FILE, 'utf8')); } catch {}

  log.push({
    ...result,
    content: undefined, // Avoid logging full raw content payload.
    contentPreview: JSON.stringify(result.content || {}).substring(0, 240),
  });

  // Keep last 1000 entries
  if (log.length > 1000) log = log.slice(-1000);
  fs.writeFileSync(LOG_FILE, JSON.stringify(log, null, 2));
}

// CLI entry point
async function main() {
  const args = process.argv.slice(2);
  const channel = args[0];
  const contentFile = args[1];

  if (!channel) {
    console.log('Usage: node publish.js <channel> [content.json]');
    console.log('Channels: twitter, linkedin, reddit, reddit_engagement, devto, hackernews, github, email, blog');
    process.exit(1);
  }

  let content;
  if (contentFile) {
    content = JSON.parse(fs.readFileSync(contentFile, 'utf8'));
  } else {
    // Read from stdin
    const chunks = [];
    process.stdin.on('data', c => chunks.push(c));
    await new Promise(r => process.stdin.on('end', r));
    content = JSON.parse(Buffer.concat(chunks).toString());
  }

  const credentials = loadCredentials();
  const result = await publishToChannel(channel, content, credentials);
  console.log(JSON.stringify(result, null, 2));
}

if (require.main === module) {
  main().catch(console.error);
}

module.exports = { publishToChannel, publishToAll, loadCredentials };
