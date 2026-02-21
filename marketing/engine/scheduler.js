// =============================================================================
// Knol Marketing — Campaign Scheduler
// Orchestrates daily/weekly/monthly marketing campaigns autonomously
// Can run via: cron, GitHub Actions, or `node scheduler.js --run <cadence>`
// =============================================================================

const fs = require('fs');
const path = require('path');
const { generateContent } = require('./generate');
const { publishToChannel, loadCredentials } = require('./publish');

const SCHEDULE_FILE = path.join(__dirname, '..', 'schedules', 'calendar.json');
const STATE_FILE = path.join(__dirname, '..', 'data', 'scheduler-state.json');
const DEFERRED_FILE = path.join(__dirname, '..', 'data', 'deferred-queue.json');

// ---------------------------------------------------------------------------
// Campaign Definitions
// ---------------------------------------------------------------------------

const CAMPAIGNS = {
  // DAILY — lightweight social posts
  daily: {
    name: 'Daily Social',
    channels: ['twitter'],
    tasks: [
      {
        id: 'daily-tweet',
        description: 'Post a technical/promotional tweet',
        generate: async () => {
          const categories = ['tweet_technical', 'tweet_launch', 'tweet_comparison'];
          const category = categories[dayOfYear() % categories.length];
          const generated = await generateContent('tweet', category, { withMeta: true });
          const template = generated.content;
          // Templates are raw strings for tweets
          return {
            text: typeof template === 'string' ? template : (template.text || template.body || JSON.stringify(template)),
            __meta: generated.meta,
          };
        },
        publish: async (content, creds) => {
          return publishToChannel('twitter', { text: content.text, __meta: content.__meta }, creds);
        },
      },
    ],
  },

  // WEEKLY — content + engagement (runs on Tuesdays)
  weekly: {
    name: 'Weekly Content',
    preferredDay: 2, // Tuesday
    channels: ['twitter', 'linkedin', 'reddit', 'blog'],
    tasks: [
      {
        id: 'weekly-blog',
        description: 'Publish a blog post',
        generate: async () => {
          const topics = require('../channels/blog').generateTopicIdeas();
          const topic = topics[weekOfYear() % topics.length];
          const generated = await generateContent('blog', 'blog_technical', { withMeta: true });
          const template = generated.content;
          const body = typeof template.body === 'string'
            ? template.body
            : typeof template.text === 'string'
              ? template.text
              : `# ${template.title || topic.title}\n\n${template.description || topic.description || 'Engineering deep dive from the Knol team.'}\n\nGitHub: https://github.com/aiknol/knol\nDocs: https://aiknol.com/docs`;
          return {
            title: template.title || topic.title,
            text: body,
            tags: template.tags || topic.tags,
            __meta: generated.meta,
          };
        },
        publish: async (content, creds) => {
          // Publish blog post
          const blogResult = await publishToChannel('blog', {
            title: content.title,
            body: content.text,
            tags: content.tags || ['rust', 'ai', 'memory'],
            __meta: content.__meta,
          }, creds);

          // Cross-post to Dev.to
          if (blogResult.success) {
            await publishToChannel('devto', {
              title: content.title,
              body: content.text,
              tags: content.tags || ['rust', 'ai', 'opensource', 'webdev'],
              canonicalUrl: blogResult.data?.url,
              __meta: content.__meta,
            }, creds);
          }

          return blogResult;
        },
      },
      {
        id: 'weekly-linkedin',
        description: 'Post LinkedIn update',
        generate: async () => {
          const generated = await generateContent('linkedin', 'linkedin_technical', { withMeta: true });
          const template = generated.content;
          return {
            text: typeof template === 'string' ? template : (template.text || template.body || template),
            __meta: generated.meta,
          };
        },
        publish: async (content, creds) => {
          return publishToChannel('linkedin', { text: content.text, __meta: content.__meta }, creds);
        },
      },
      {
        id: 'weekly-reddit',
        description: 'Post to a relevant subreddit',
        generate: async () => {
          const subs = ['rust', 'opensource', 'selfhosted', 'LocalLLaMA'];
          const sub = subs[weekOfYear() % subs.length];
          const category = sub === 'rust' ? 'reddit_rust' : 'reddit_ml';
          const generated = await generateContent('reddit', category, { withMeta: true });
          const template = generated.content;
          return {
            subreddit: normalizeSubreddit(template.subreddit || sub),
            title: template.title || `Knol: Memory layer for AI (${sub})`,
            text: template.body || template.text || template,
            __meta: generated.meta,
          };
        },
        publish: async (content, creds) => {
          return publishToChannel('reddit', {
            subreddit: content.subreddit || 'rust',
            title: content.title,
            text: content.text,
            kind: 'self',
            __meta: content.__meta,
          }, creds);
        },
      },
      {
        id: 'weekly-reddit-engagement',
        description: 'Engage in relevant Reddit discussions',
        generate: async () => {
          return {
            query: '(memory OR rag OR "ai agents" OR llm)',
            subreddits: ['rust', 'MachineLearning', 'LocalLLaMA', 'opensource', 'selfhosted'],
            limitPerSub: 3,
            limit: 8,
            minScore: 3,
            minComments: 0,
            commentTemplate: 'Maintainer here. We built an open-source memory stack for this exact problem. Happy to share design tradeoffs if useful: https://github.com/aiknol/knol',
            __meta: { templateCategory: 'reddit_engagement', variantId: 'reddit_engagement:v0', variantIndex: 0 },
          };
        },
        publish: async (content, creds) => {
          const result = await publishToChannel('reddit_engagement', content, creds);
          console.log('\n=== Reddit Engagement Opportunities ===');
          for (const opp of (result.data?.opportunities || []).slice(0, 5)) {
            console.log(`  [r/${opp.subreddit}] ${opp.title}`);
            console.log(`  ${opp.permalink || opp.url}\n`);
          }
          return result;
        },
      },
    ],
  },

  // MONTHLY — big pushes + analytics
  monthly: {
    name: 'Monthly Push',
    preferredDay: 1, // 1st of month
    channels: ['twitter', 'linkedin', 'email', 'github', 'hackernews'],
    tasks: [
      {
        id: 'monthly-newsletter',
        description: 'Send monthly newsletter',
        generate: async () => {
          const generated = await generateContent('email', 'email_weekly', { withMeta: true });
          const template = generated.content;
          return {
            subject: template.subject || `Knol Monthly Update`,
            text: template.body || template.text || (typeof template === 'string' ? template : JSON.stringify(template)),
            __meta: generated.meta,
          };
        },
        publish: async (content, creds) => {
          return publishToChannel('email', {
            subject: content.subject || `Knol Monthly Update — ${new Date().toLocaleDateString('en', { month: 'long', year: 'numeric' })}`,
            text: content.text,
            htmlContent: content.text,
            __meta: content.__meta,
          }, creds);
        },
      },
      {
        id: 'monthly-github-update',
        description: 'Update GitHub repo metadata + stats',
        generate: () => ({
          type: 'metadata',
          description: 'Open-source long-term memory layer for AI agents. Rust + pgvector + NATS.',
          topics: ['memory', 'llm', 'ai-agents', 'rust', 'pgvector', 'knowledge-graph', 'rag', 'vector-search'],
          __meta: { templateCategory: 'github_metadata', variantId: 'github_metadata:v0', variantIndex: 0 },
        }),
        publish: async (content, creds) => {
          return publishToChannel('github', content, creds);
        },
      },
      {
        id: 'monthly-hn-monitor',
        description: 'Find HN engagement opportunities',
        generate: async () => {
          const hn = require('../channels/hackernews');
          const opportunities = await hn.findEngagementOpportunities();
          return {
            opportunities: opportunities.slice(0, 5),
            showHN: hn.generateShowHN(),
          };
        },
        publish: async (content) => {
          // HN is always manual — log opportunities
          console.log('\n=== HN Engagement Opportunities ===');
          for (const opp of (content.opportunities || [])) {
            console.log(`  [${opp.points} pts] ${opp.title}`);
            console.log(`  ${opp.hnUrl}\n`);
          }
          console.log('Show HN submit URL:', content.showHN?.submitUrl);
          return { success: true, manual: true, opportunities: content.opportunities?.length || 0 };
        },
      },
      {
        id: 'monthly-thread',
        description: 'Post a Twitter thread about progress',
        generate: async () => {
          const generated = await generateContent('tweet', 'tweet_launch', { withMeta: true });
          const template = generated.content;
          return {
            text: typeof template === 'string' ? template : (template.text || template.body || JSON.stringify(template)),
            __meta: generated.meta,
          };
        },
        publish: async (content, creds) => {
          const tweets = [
            content.text,
            '🧵 Some highlights from this month:\n\n• Performance improvements\n• New API features\n• Community growth\n\nThread below 👇',
            'If you\'re building AI agents that need to remember context across sessions, check us out:\n\nhttps://github.com/aiknol/knol\n\nStar ⭐ if you find it useful!',
          ];
          return publishToChannel('twitter', { tweets, __meta: content.__meta }, creds);
        },
      },
    ],
  },
};

// ---------------------------------------------------------------------------
// State Management
// ---------------------------------------------------------------------------

function loadState() {
  try {
    return JSON.parse(fs.readFileSync(STATE_FILE, 'utf8'));
  } catch {
    return { lastRun: {}, history: [] };
  }
}

function saveState(state) {
  const dir = path.dirname(STATE_FILE);
  if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
  fs.writeFileSync(STATE_FILE, JSON.stringify(state, null, 2));
}

function queueDeferred(taskId, cadence, details = {}) {
  const dir = path.dirname(DEFERRED_FILE);
  if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });

  let queue = [];
  try {
    queue = JSON.parse(fs.readFileSync(DEFERRED_FILE, 'utf8'));
  } catch {}

  queue.push({
    taskId,
    cadence,
    deferredAt: new Date().toISOString(),
    reason: details.reason || 'deferred',
    waitSeconds: details.waitSeconds || 0,
    nextAllowedAt: details.nextAllowedAt || null,
  });

  // Keep queue bounded
  if (queue.length > 500) queue = queue.slice(-500);
  fs.writeFileSync(DEFERRED_FILE, JSON.stringify(queue, null, 2));
}

function shouldRun(cadence, campaign, state) {
  const now = new Date();
  const lastRun = state.lastRun[cadence];
  const weekday = now.getUTCDay();
  const dayOfMonth = now.getUTCDate();

  if (cadence === 'weekly' && Number.isInteger(campaign.preferredDay) && weekday !== campaign.preferredDay) {
    return {
      allowed: false,
      reason: `outside_preferred_weekday (today=${weekday}, preferred=${campaign.preferredDay})`,
    };
  }
  if (cadence === 'monthly' && Number.isInteger(campaign.preferredDay) && dayOfMonth !== campaign.preferredDay) {
    return {
      allowed: false,
      reason: `outside_preferred_month_day (today=${dayOfMonth}, preferred=${campaign.preferredDay})`,
    };
  }

  if (!lastRun) return { allowed: true, reason: 'first_run' };

  const last = new Date(lastRun);
  const hoursSince = (now - last) / (1000 * 60 * 60);

  switch (cadence) {
    case 'daily':
      return { allowed: hoursSince >= 20, reason: 'too_recent' }; // At least 20h between daily
    case 'weekly':
      return { allowed: hoursSince >= 144, reason: 'too_recent' }; // At least 6 days
    case 'monthly':
      return { allowed: hoursSince >= 672, reason: 'too_recent' }; // At least 28 days
    default:
      return { allowed: true, reason: 'default' };
  }
}

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

async function runCampaign(cadence, options = {}) {
  const { dryRun = false, force = false } = options;
  const campaign = CAMPAIGNS[cadence];

  if (!campaign) {
    console.error(`Unknown cadence: ${cadence}`);
    return { success: false, error: `Unknown cadence: ${cadence}` };
  }

  const state = loadState();

  const eligibility = shouldRun(cadence, campaign, state);
  if (!force && !eligibility.allowed) {
    console.log(`[${cadence}] Skipping — ${eligibility.reason} (last: ${state.lastRun[cadence] || 'never'})`);
    return { success: true, skipped: true };
  }

  console.log(`\n${'='.repeat(60)}`);
  console.log(`[${new Date().toISOString()}] Running ${campaign.name} campaign`);
  console.log(`${'='.repeat(60)}\n`);

  const credentials = loadCredentials();
  const results = [];

  for (const task of campaign.tasks) {
    console.log(`  → ${task.description}...`);

    try {
      // Generate content
      const generated = await task.generate();
      const content = annotateTaskMeta(generated, {
        cadence,
        campaignName: campaign.name,
        taskId: task.id,
      });

      if (dryRun) {
        console.log(`    [DRY RUN] Would publish:`, JSON.stringify(content).substring(0, 150));
        results.push({ taskId: task.id, success: true, dryRun: true });
        continue;
      }

      // Publish
      const result = await task.publish(content, credentials);
      results.push({ taskId: task.id, ...result });

      if (result.deferred) {
        queueDeferred(task.id, cadence, result.data || {});
      }

      console.log(`    ${result.success ? '✓' : (result.deferred ? '⏸' : '✗')} ${result.success ? 'Published' : result.data?.error || 'Failed'}`);

      // Delay between tasks
      await new Promise(r => setTimeout(r, 3000));
    } catch (e) {
      console.error(`    ✗ Error: ${e.message}`);
      results.push({ taskId: task.id, success: false, error: e.message });
    }
  }

  // Update state
  const successCount = results.filter(r => r.success).length;
  const deferredCount = results.filter(r => r.deferred).length;
  const failedCount = results.length - successCount - deferredCount;

  if (!dryRun) {
    if (successCount > 0) {
      state.lastRun[cadence] = new Date().toISOString();
    } else {
      console.log(`  No successful tasks for ${cadence}; preserving previous lastRun for faster retry.`);
    }

    state.history.push({
      cadence,
      timestamp: new Date().toISOString(),
      results: results.map(r => ({ taskId: r.taskId, success: r.success, deferred: !!r.deferred })),
      successCount,
      deferredCount,
      failedCount,
    });
    // Keep last 100 history entries
    if (state.history.length > 100) state.history = state.history.slice(-100);
    saveState(state);
  }

  console.log(`\n  Summary: ${successCount}/${results.length} succeeded, ${deferredCount} deferred, ${failedCount} failed\n`);

  return { success: true, results };
}

// Run all due campaigns
async function runAll(options = {}) {
  const cadences = ['daily', 'weekly', 'monthly'];
  const results = {};

  for (const cadence of cadences) {
    results[cadence] = await runCampaign(cadence, options);
  }

  return results;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function dayOfYear() {
  const now = new Date();
  const start = new Date(now.getFullYear(), 0, 0);
  return Math.floor((now - start) / (1000 * 60 * 60 * 24));
}

function weekOfYear() {
  const now = new Date();
  const start = new Date(now.getFullYear(), 0, 1);
  return Math.ceil(((now - start) / (1000 * 60 * 60 * 24) + start.getDay() + 1) / 7);
}

function annotateTaskMeta(content, details) {
  const fallback = content && typeof content === 'object' ? content : { text: String(content ?? '') };
  const meta = fallback.__meta || {};
  return {
    ...fallback,
    __meta: {
      ...meta,
      cadence: details.cadence,
      campaignName: details.campaignName,
      taskId: details.taskId,
    },
  };
}

function normalizeSubreddit(name) {
  const value = String(name || '').trim();
  return value.replace(/^r\//i, '');
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

async function main() {
  const args = process.argv.slice(2);
  const runIndex = args.indexOf('--run');
  const dryRun = args.includes('--dry-run');
  const force = args.includes('--force');

  if (runIndex === -1) {
    console.log('Knol Marketing Scheduler');
    console.log('========================');
    console.log('Usage:');
    console.log('  node scheduler.js --run daily     Run daily campaign');
    console.log('  node scheduler.js --run weekly    Run weekly campaign');
    console.log('  node scheduler.js --run monthly   Run monthly campaign');
    console.log('  node scheduler.js --run all       Run all due campaigns');
    console.log('');
    console.log('Options:');
    console.log('  --dry-run    Preview without publishing');
    console.log('  --force      Run even if recently ran');
    console.log('');

    const state = loadState();
    console.log('Last runs:');
    for (const [cadence, time] of Object.entries(state.lastRun || {})) {
      console.log(`  ${cadence}: ${time}`);
    }
    return;
  }

  const cadence = args[runIndex + 1];
  if (cadence === 'all') {
    await runAll({ dryRun, force });
  } else {
    await runCampaign(cadence, { dryRun, force });
  }
}

if (require.main === module) {
  main().catch(console.error);
}

module.exports = { runCampaign, runAll, CAMPAIGNS };
