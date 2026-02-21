// =============================================================================
// Knol Marketing — Analytics & Reporting
// Aggregates metrics from all channels into a single report
// Generates HTML dashboard (self-hosted, no external analytics needed)
// =============================================================================

const fs = require('fs');
const path = require('path');

const github = require('../channels/github');
const hackernews = require('../channels/hackernews');

const DATA_DIR = path.join(__dirname, '..', 'data');
const REPORT_DIR = path.join(__dirname, '..', 'reports');
const METRICS_FILE = path.join(DATA_DIR, 'metrics-history.json');
const ACQUISITION_FILE = path.join(DATA_DIR, 'acquisition-events.json');

// ---------------------------------------------------------------------------
// Collect metrics from all sources
// ---------------------------------------------------------------------------

async function collectMetrics() {
  const credentials = {
    github: { token: process.env.GITHUB_TOKEN },
  };

  const metrics = {
    timestamp: new Date().toISOString(),
    github: { stars: 0, forks: 0, views: 0, clones: 0 },
    content: { blogPosts: 0, tweets: 0, linkedinPosts: 0, redditPosts: 0, devtoPosts: 0 },
    email: { subscribers: 0, sent: 0 },
    engagement: { hnMentions: 0 },
    automation: { deferred: 0, rateLimited: 0 },
    attribution: {
      trackedLinks: 0,
      trackedPosts: 0,
      topVariants: [],
      channelSuccess: {},
      starsPerDay30d: 0,
      clicks: 0,
      signups: 0,
    },
  };

  // GitHub stats
  try {
    const ghStats = await github.getRepoStats(credentials.github);
    if (ghStats.success) {
      metrics.github = {
        stars: ghStats.stars,
        forks: ghStats.forks,
        watchers: ghStats.watchers,
        views: ghStats.views14d,
        uniqueVisitors: ghStats.uniqueVisitors14d,
        clones: ghStats.clones14d,
        openIssues: ghStats.openIssues,
      };
    }
  } catch (e) {
    console.warn('GitHub stats failed:', e.message);
  }

  // Publish log stats
  try {
    const publishLog = JSON.parse(fs.readFileSync(path.join(DATA_DIR, 'publish-log.json'), 'utf8'));
    const thisMonth = publishLog.filter(e => {
      const d = new Date(e.timestamp);
      const now = new Date();
      return d.getMonth() === now.getMonth() && d.getFullYear() === now.getFullYear();
    });

    metrics.content.tweets = thisMonth.filter(e => e.channel === 'twitter' && e.success).length;
    metrics.content.linkedinPosts = thisMonth.filter(e => e.channel === 'linkedin' && e.success).length;
    metrics.content.redditPosts = thisMonth.filter(e => e.channel === 'reddit' && e.success).length;
    metrics.content.devtoPosts = thisMonth.filter(e => e.channel === 'devto' && e.success).length;
    metrics.content.blogPosts = thisMonth.filter(e => e.channel === 'blog' && e.success).length;
    metrics.automation.deferred = thisMonth.filter(e => e.deferred).length;
    metrics.automation.rateLimited = thisMonth.filter(e => e.data?.rateLimited || e.rateLimited).length;

    const variantStats = new Map();
    const channelStats = new Map();
    for (const entry of thisMonth) {
      const variantId = entry?.attribution?.variantId || '';
      const tracking = entry?.tracking || {};
      const trackedLinks = Number.isFinite(tracking.utmUrlsApplied) ? tracking.utmUrlsApplied : 0;

      if (trackedLinks > 0) {
        metrics.attribution.trackedPosts += 1;
        metrics.attribution.trackedLinks += trackedLinks;
      }

      const ch = entry.channel || 'unknown';
      const chAgg = channelStats.get(ch) || { success: 0, fail: 0 };
      if (entry.success) chAgg.success += 1; else chAgg.fail += 1;
      channelStats.set(ch, chAgg);

      if (!variantId) continue;
      const agg = variantStats.get(variantId) || { variantId, attempts: 0, success: 0 };
      agg.attempts += 1;
      if (entry.success) agg.success += 1;
      variantStats.set(variantId, agg);
    }

    metrics.attribution.topVariants = Array.from(variantStats.values())
      .sort((a, b) => {
        const rb = b.attempts ? b.success / b.attempts : 0;
        const ra = a.attempts ? a.success / a.attempts : 0;
        if (rb !== ra) return rb - ra;
        return b.attempts - a.attempts;
      })
      .slice(0, 5)
      .map((v) => ({
        variantId: v.variantId,
        attempts: v.attempts,
        successRate: Number(((v.success / Math.max(v.attempts, 1)) * 100).toFixed(1)),
      }));

    metrics.attribution.channelSuccess = Object.fromEntries(
      Array.from(channelStats.entries()).map(([ch, agg]) => [
        ch,
        {
          attempts: agg.success + agg.fail,
          successRate: Number(((agg.success / Math.max(agg.success + agg.fail, 1)) * 100).toFixed(1)),
        },
      ]),
    );
  } catch {}

  // Email stats
  try {
    const subscribers = JSON.parse(fs.readFileSync(path.join(DATA_DIR, 'subscribers.json'), 'utf8'));
    metrics.email.subscribers = subscribers.filter(s => !s.unsubscribed).length;

    const emailLog = JSON.parse(fs.readFileSync(path.join(DATA_DIR, 'email-log.json'), 'utf8'));
    metrics.email.sent = emailLog.reduce((sum, e) => sum + (e.sent || 0), 0);
  } catch {}

  // HN mentions
  try {
    const hnResults = await hackernews.searchDiscussions('knol memory AI');
    metrics.engagement.hnMentions = hnResults.stories?.length || 0;
  } catch {}

  // Optional acquisition events (click/sign-up attribution)
  try {
    const events = JSON.parse(fs.readFileSync(ACQUISITION_FILE, 'utf8'));
    const now = new Date();
    const thisMonth = events.filter((e) => {
      const d = new Date(e.timestamp || e.date || 0);
      return d.getUTCFullYear() === now.getUTCFullYear() && d.getUTCMonth() === now.getUTCMonth();
    });
    metrics.attribution.clicks = thisMonth.reduce((sum, e) => sum + (Number(e.clicks) || 0), 0);
    metrics.attribution.signups = thisMonth.reduce((sum, e) => sum + (Number(e.signups) || 0), 0);
  } catch {}

  // 30-day star velocity from history snapshots
  try {
    const history = loadMetricsHistory();
    const cutoff = Date.now() - (30 * 24 * 60 * 60 * 1000);
    const window = history.filter((h) => new Date(h.timestamp).getTime() >= cutoff);
    if (window.length >= 2) {
      const first = window[0].github?.stars || 0;
      const last = window[window.length - 1].github?.stars || 0;
      metrics.attribution.starsPerDay30d = Number(((last - first) / 30).toFixed(2));
    }
  } catch {}

  // Save to history
  saveMetrics(metrics);
  return metrics;
}

function saveMetrics(metrics) {
  if (!fs.existsSync(DATA_DIR)) fs.mkdirSync(DATA_DIR, { recursive: true });
  let history = [];
  try { history = JSON.parse(fs.readFileSync(METRICS_FILE, 'utf8')); } catch {}
  history.push(metrics);
  if (history.length > 365) history = history.slice(-365); // Keep 1 year
  fs.writeFileSync(METRICS_FILE, JSON.stringify(history, null, 2));
}

function loadMetricsHistory() {
  try {
    return JSON.parse(fs.readFileSync(METRICS_FILE, 'utf8'));
  } catch {
    return [];
  }
}

// ---------------------------------------------------------------------------
// Generate HTML Dashboard
// ---------------------------------------------------------------------------

function generateDashboard(metrics, history) {
  const prev = history.length > 1 ? history[history.length - 2] : null;

  function delta(current, previous, key) {
    if (!previous) return '';
    const path = key.split('.');
    let c = current, p = previous;
    for (const k of path) { c = c?.[k]; p = p?.[k]; }
    if (c == null || p == null) return '';
    const diff = c - p;
    if (diff === 0) return '<span style="color:#666">→ 0</span>';
    return diff > 0
      ? `<span style="color:#22c55e">↑ +${diff}</span>`
      : `<span style="color:#ef4444">↓ ${diff}</span>`;
  }

  // Sparkline data (last 30 entries for stars)
  const starHistory = history.slice(-30).map(h => h.github?.stars || 0);
  const maxStars = Math.max(...starHistory, 1);
  const sparklinePoints = starHistory.map((v, i) =>
    `${(i / Math.max(starHistory.length - 1, 1)) * 200},${40 - (v / maxStars) * 35}`
  ).join(' ');

  return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Knol Marketing Dashboard</title>
<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; background: #0a0a0a; color: #e5e5e5; }
  .header { padding: 24px 32px; border-bottom: 1px solid #222; display: flex; align-items: center; gap: 16px; }
  .header h1 { color: #6E56CF; font-size: 20px; }
  .header .date { color: #666; font-size: 14px; margin-left: auto; }
  .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 16px; padding: 24px 32px; }
  .card { background: #141414; border: 1px solid #222; border-radius: 12px; padding: 20px; }
  .card .label { font-size: 12px; color: #888; text-transform: uppercase; letter-spacing: 0.5px; }
  .card .value { font-size: 32px; font-weight: 700; margin: 8px 0 4px; }
  .card .delta { font-size: 13px; }
  .section { padding: 16px 32px; }
  .section h2 { font-size: 16px; color: #999; margin-bottom: 12px; }
  table { width: 100%; border-collapse: collapse; background: #141414; border-radius: 8px; overflow: hidden; }
  th, td { padding: 10px 16px; text-align: left; border-bottom: 1px solid #1a1a1a; font-size: 14px; }
  th { background: #1a1a1a; color: #888; font-weight: 500; text-transform: uppercase; font-size: 11px; letter-spacing: 0.5px; }
  .badge { display: inline-block; padding: 2px 8px; border-radius: 4px; font-size: 11px; }
  .badge-success { background: #052e16; color: #22c55e; }
  .badge-warn { background: #422006; color: #f59e0b; }
  .badge-info { background: #172554; color: #60a5fa; }
  svg { display: block; }
  .footer { padding: 24px 32px; color: #444; font-size: 12px; text-align: center; }
</style>
</head>
<body>

<div class="header">
  <h1>Knol</h1>
  <span style="color:#666;font-size:14px">Marketing Dashboard</span>
  <span class="date">${new Date().toLocaleDateString('en', { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' })}</span>
</div>

<div class="grid">
  <div class="card">
    <div class="label">GitHub Stars</div>
    <div class="value">${metrics.github.stars}</div>
    <div class="delta">${delta(metrics, prev, 'github.stars')}</div>
    <svg width="200" height="40" style="margin-top:8px">
      <polyline points="${sparklinePoints}" fill="none" stroke="#6E56CF" stroke-width="2"/>
    </svg>
  </div>
  <div class="card">
    <div class="label">GitHub Forks</div>
    <div class="value">${metrics.github.forks}</div>
    <div class="delta">${delta(metrics, prev, 'github.forks')}</div>
  </div>
  <div class="card">
    <div class="label">Repo Views (14d)</div>
    <div class="value">${metrics.github.views || 0}</div>
    <div class="delta">${delta(metrics, prev, 'github.views')}</div>
  </div>
  <div class="card">
    <div class="label">Repo Clones (14d)</div>
    <div class="value">${metrics.github.clones || 0}</div>
    <div class="delta">${delta(metrics, prev, 'github.clones')}</div>
  </div>
  <div class="card">
    <div class="label">Newsletter Subs</div>
    <div class="value">${metrics.email.subscribers}</div>
    <div class="delta">${delta(metrics, prev, 'email.subscribers')}</div>
  </div>
  <div class="card">
    <div class="label">Emails Sent</div>
    <div class="value">${metrics.email.sent}</div>
  </div>
  <div class="card">
    <div class="label">Deferred by Policy</div>
    <div class="value">${metrics.automation.deferred}</div>
  </div>
  <div class="card">
    <div class="label">Rate-Limited Events</div>
    <div class="value">${metrics.automation.rateLimited}</div>
  </div>
  <div class="card">
    <div class="label">Tracked Links (Month)</div>
    <div class="value">${metrics.attribution.trackedLinks}</div>
  </div>
  <div class="card">
    <div class="label">Signups (Month)</div>
    <div class="value">${metrics.attribution.signups}</div>
  </div>
</div>

<div class="section">
  <h2>Content Published This Month</h2>
  <table>
    <tr><th>Channel</th><th>Posts</th><th>Status</th></tr>
    <tr><td>Twitter</td><td>${metrics.content.tweets}</td><td><span class="badge ${metrics.content.tweets > 0 ? 'badge-success' : 'badge-warn'}">${metrics.content.tweets > 0 ? 'Active' : 'Pending'}</span></td></tr>
    <tr><td>LinkedIn</td><td>${metrics.content.linkedinPosts}</td><td><span class="badge ${metrics.content.linkedinPosts > 0 ? 'badge-success' : 'badge-warn'}">${metrics.content.linkedinPosts > 0 ? 'Active' : 'Pending'}</span></td></tr>
    <tr><td>Reddit</td><td>${metrics.content.redditPosts}</td><td><span class="badge ${metrics.content.redditPosts > 0 ? 'badge-success' : 'badge-warn'}">${metrics.content.redditPosts > 0 ? 'Active' : 'Pending'}</span></td></tr>
    <tr><td>Dev.to</td><td>${metrics.content.devtoPosts}</td><td><span class="badge ${metrics.content.devtoPosts > 0 ? 'badge-success' : 'badge-warn'}">${metrics.content.devtoPosts > 0 ? 'Active' : 'Pending'}</span></td></tr>
    <tr><td>Blog</td><td>${metrics.content.blogPosts}</td><td><span class="badge ${metrics.content.blogPosts > 0 ? 'badge-success' : 'badge-warn'}">${metrics.content.blogPosts > 0 ? 'Active' : 'Pending'}</span></td></tr>
    <tr><td>HN Mentions</td><td>${metrics.engagement.hnMentions}</td><td><span class="badge badge-info">Monitoring</span></td></tr>
  </table>
</div>

<div class="section">
  <h2>Attribution (30d/Month)</h2>
  <table>
    <tr><th>Metric</th><th>Value</th><th>Status</th></tr>
    <tr><td>Stars / Day (30d)</td><td>${metrics.attribution.starsPerDay30d}</td><td><span class="badge badge-info">Velocity</span></td></tr>
    <tr><td>Tracked Link Clicks</td><td>${metrics.attribution.clicks}</td><td><span class="badge badge-info">Attribution</span></td></tr>
    <tr><td>Attributed Signups</td><td>${metrics.attribution.signups}</td><td><span class="badge ${metrics.attribution.signups > 0 ? 'badge-success' : 'badge-warn'}">${metrics.attribution.signups > 0 ? 'Growing' : 'Needs signal'}</span></td></tr>
  </table>
</div>

<div class="section">
  <h2>Top Variants (Month)</h2>
  <table>
    <tr><th>Variant</th><th>Attempts</th><th>Success Rate</th></tr>
    ${(metrics.attribution.topVariants || []).map((v) =>
      `<tr><td>${v.variantId}</td><td>${v.attempts}</td><td>${v.successRate}%</td></tr>`
    ).join('') || '<tr><td colspan="3">No variant data yet</td></tr>'}
  </table>
</div>

<div class="section">
  <h2>Channel Success (Month)</h2>
  <table>
    <tr><th>Channel</th><th>Attempts</th><th>Success Rate</th></tr>
    ${Object.entries(metrics.attribution.channelSuccess || {}).map(([channel, stats]) =>
      `<tr><td>${channel}</td><td>${stats.attempts}</td><td>${stats.successRate}%</td></tr>`
    ).join('') || '<tr><td colspan="3">No channel data yet</td></tr>'}
  </table>
</div>

<div class="section">
  <h2>Cost Summary</h2>
  <table>
    <tr><th>Service</th><th>Tier</th><th>Cost</th><th>Limits</th></tr>
    <tr><td>Twitter API</td><td>Free</td><td>$0</td><td>1,500 tweets/mo</td></tr>
    <tr><td>Reddit API</td><td>Free</td><td>$0</td><td>60 req/min</td></tr>
    <tr><td>Dev.to API</td><td>Free</td><td>$0</td><td>30 articles/day</td></tr>
    <tr><td>GitHub API</td><td>Free</td><td>$0</td><td>5,000 req/hr</td></tr>
    <tr><td>Gmail SMTP</td><td>Free</td><td>$0</td><td>500 emails/day</td></tr>
    <tr><td>GitHub Pages</td><td>Free</td><td>$0</td><td>Unlimited</td></tr>
    <tr><td>GitHub Actions</td><td>Free</td><td>$0</td><td>2,000 min/mo</td></tr>
    <tr><td colspan="2" style="font-weight:700">Total</td><td style="color:#22c55e;font-weight:700">$0/mo</td><td></td></tr>
  </table>
</div>

<div class="footer">
  Generated by Knol Marketing Engine · ${new Date().toISOString()} · All metrics auto-collected
</div>

</body>
</html>`;
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

async function main() {
  const args = process.argv.slice(2);

  console.log('Collecting metrics from all channels...');
  const metrics = await collectMetrics();
  const history = loadMetricsHistory();

  console.log('\n📊 Knol Marketing Report');
  console.log('========================');
  console.log(`GitHub: ${metrics.github.stars} stars, ${metrics.github.forks} forks, ${metrics.github.views || 0} views (14d)`);
  console.log(`Content: ${metrics.content.tweets} tweets, ${metrics.content.linkedinPosts} linkedin, ${metrics.content.redditPosts} reddit, ${metrics.content.blogPosts} blog`);
  console.log(`Email: ${metrics.email.subscribers} subscribers, ${metrics.email.sent} sent`);
  console.log(`Automation: ${metrics.automation.deferred} deferred, ${metrics.automation.rateLimited} rate-limited`);
  console.log(`HN: ${metrics.engagement.hnMentions} mentions found`);
  console.log(`Attribution: ${metrics.attribution.trackedLinks} tracked links, ${metrics.attribution.clicks} clicks, ${metrics.attribution.signups} signups`);

  if (args.includes('--html') || args.includes('--dashboard')) {
    if (!fs.existsSync(REPORT_DIR)) fs.mkdirSync(REPORT_DIR, { recursive: true });
    const html = generateDashboard(metrics, history);
    const filename = `dashboard-${new Date().toISOString().split('T')[0]}.html`;
    const filepath = path.join(REPORT_DIR, filename);
    fs.writeFileSync(filepath, html);
    console.log(`\nDashboard: ${filepath}`);

    // Also write as latest
    fs.writeFileSync(path.join(REPORT_DIR, 'dashboard-latest.html'), html);
  }

  return metrics;
}

if (require.main === module) {
  main().catch(console.error);
}

module.exports = { collectMetrics, generateDashboard, loadMetricsHistory };
