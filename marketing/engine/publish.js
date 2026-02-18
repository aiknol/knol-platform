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
  const endpoint = endpointForChannel(channel, content);
  const useApi = hasApiCredentials(channel, credentials);
  const preflight = useApi ? limiter.shouldAllowPublish(channel, endpoint, content) : { allowed: true, reason: 'manual_or_no_api' };

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
    limiter.reservePublish(channel, preflight.hash);
  }

  try {
    switch (channel) {
      case 'twitter': {
        if (Array.isArray(content.tweets)) {
          // Thread
          const results = await twitter.postThread(content.tweets, credentials.twitter);
          result.success = results.every(r => r.success);
          result.data = results;
        } else {
          result.data = await twitter.postTweet(content.text, credentials.twitter);
          result.success = result.data.success;
        }
        break;
      }

      case 'linkedin': {
        if (content.articleUrl) {
          result.data = await linkedin.postArticle(content.title, content.text, content.articleUrl, credentials.linkedin);
        } else {
          result.data = await linkedin.postUpdate(content.text, credentials.linkedin);
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
          content.subreddit,
          content.title,
          content.text,
          auth.token,
          content.kind || 'self'
        );
        result.success = result.data.success;
        break;
      }

      case 'devto': {
        result.data = await devto.publishArticle({
          title: content.title,
          body: content.body || content.text,
          tags: content.tags,
          series: content.series,
          canonicalUrl: content.canonicalUrl,
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
        if (content.type === 'release') {
          result.data = await github.createRelease(
            content.tag, content.name, content.body, credentials.github
          );
        } else if (content.type === 'metadata') {
          result.data = await github.updateRepoMetadata(
            content.description, content.topics, credentials.github
          );
        } else {
          result.data = await github.getRepoStats(credentials.github);
        }
        result.success = result.data?.success ?? true;
        break;
      }

      case 'email': {
        const htmlBody = email.generateNewsletterHtml(content.htmlContent || content.text);
        const textBody = content.text;
        result.data = await email.sendNewsletter(content.subject, htmlBody, textBody, credentials.email);
        result.success = result.data.sent > 0 || result.data.manual;
        break;
      }

      case 'blog': {
        const post = blog.createPost(content.title, content.body || content.text, content.tags);
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
      limiter.updateFromHeaders(channel, endpoint, result.data.headers);
    }
    if (result?.data?.rateLimited || result?.rateLimited || result?.data?.statusCode === 429) {
      const retryMeta = extractRetryAndReset(result.data || result);
      limiter.markRateLimited(channel, endpoint, retryMeta);
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
    content: undefined, // Don't log full content
    contentPreview: JSON.stringify(result.content).substring(0, 100),
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
    console.log('Channels: twitter, linkedin, reddit, devto, hackernews, github, email, blog');
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
