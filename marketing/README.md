# Knol Marketing Engine

Autonomous marketing service for Knol — runs on **$0/month** using only free-tier APIs and GitHub Actions.

## Architecture

```
engine/
  generate.js    ← Content generation (templates + optional Claude API)
  publish.js     ← Routes content to channel adapters
  scheduler.js   ← Cron orchestrator (daily/weekly/monthly campaigns)
  limits.js      ← Limit/cooldown state inspector
  setup.js       ← First-time setup + credential validation

channels/
  twitter.js     ← Twitter API v2 (OAuth 1.0a)
  linkedin.js    ← LinkedIn API (with manual fallback)
  reddit.js      ← Reddit API (script app)
  devto.js       ← Dev.to article publishing
  hackernews.js  ← HN monitoring + submission prep (manual)
  github.js      ← GitHub releases, stats, metadata
  email.js       ← Newsletter via SMTP (self-managed subscribers)
  blog.js        ← Jekyll blog on GitHub Pages

analytics/
  report.js      ← Metrics collection + HTML dashboard

schedules/
  calendar.json  ← Campaign schedule + content rotation
```

## Quick Start

```bash
# 1. Setup
cp .env.example .env  # Fill in your API keys
node engine/setup.js

# 2. Preview (dry run)
node engine/scheduler.js --run daily --dry-run

# 3. Run a campaign
node engine/scheduler.js --run daily
node engine/scheduler.js --run weekly
node engine/scheduler.js --run monthly

# 4. Run all due campaigns
node engine/scheduler.js --run all

# 5. Generate dashboard
node analytics/report.js --dashboard

# 6. Inspect live rate-limit/cooldown state
node engine/limits.js
```

## Campaign Schedule

| Cadence | Schedule | Channels | Content |
|---------|----------|----------|---------|
| Daily | 2pm UTC | Twitter | Rotating tweets (technical, launch, comparison) |
| Weekly | Tue 3pm UTC | Blog, Dev.to, LinkedIn, Reddit | Blog post + cross-posting |
| Monthly | 1st 4pm UTC | Email, GitHub, Twitter thread, HN | Newsletter + repo update + thread |

## Autonomous Operation

Marketing runs fully autonomously via GitHub Actions (`.github/workflows/marketing.yml`):

- **Daily**: Cron triggers tweet at 2pm UTC
- **Weekly**: Cron triggers content push on Tuesdays
- **Monthly**: Cron triggers big push on the 1st
- **Manual**: `workflow_dispatch` for on-demand campaigns
- **Cost**: $0 (GitHub Actions free tier: 2,000 min/month)

## Free Tier Limits

| Service | Limit | Monthly Usage |
|---------|-------|---------------|
| Twitter | 1,500 tweets/mo | ~30 tweets |
| Reddit | 60 req/min | ~4 posts |
| Dev.to | 30 articles/day | ~4 articles |
| GitHub API | 5,000 req/hr | ~100 requests |
| Gmail SMTP | 500 emails/day | ~500 emails |
| GitHub Pages | Unlimited | 1 blog |
| GitHub Actions | 2,000 min/mo | ~60 minutes |

## Limit-Safe Automation (Implemented)

The engine now includes:

- Per-channel policy guardrails (`policies/channel-policies.json`)
- Safety factor budgeting (defaults to 80% of configured max)
- Min-spacing and quiet-hour enforcement
- Duplicate content suppression window
- Deferred queue for posts blocked by limits (`data/deferred-queue.json`)
- Cooldown tracking from 429 + rate-limit headers (`data/rate-limit-state.json`)
