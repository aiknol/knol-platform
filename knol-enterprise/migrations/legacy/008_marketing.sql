-- Marketing service tables
-- Campaign definitions, publish audit log, admin actions
-- Updated for zero-cost marketing strategy (5 phases, 11 campaigns)

CREATE TABLE IF NOT EXISTS marketing_campaigns (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(64) NOT NULL UNIQUE,
    cron        VARCHAR(64) NOT NULL,
    channels    TEXT[] NOT NULL DEFAULT '{}',
    enabled     BOOLEAN NOT NULL DEFAULT true,
    phase       VARCHAR(32) NOT NULL DEFAULT 'content_engine',
    description TEXT NOT NULL DEFAULT '',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS marketing_publish_log (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign     VARCHAR(64) NOT NULL,
    channel      VARCHAR(32) NOT NULL,
    success      BOOLEAN NOT NULL,
    message_id   TEXT,
    url          TEXT,
    error        TEXT,
    published_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_publish_log_campaign ON marketing_publish_log (campaign);
CREATE INDEX idx_publish_log_channel  ON marketing_publish_log (channel);
CREATE INDEX idx_publish_log_time     ON marketing_publish_log (published_at DESC);

CREATE TABLE IF NOT EXISTS marketing_audit (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action     VARCHAR(32) NOT NULL,
    campaign   VARCHAR(64),
    details    JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Marketing analytics / stats snapshots (daily)
CREATE TABLE IF NOT EXISTS marketing_stats (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metric_name   VARCHAR(64) NOT NULL,
    metric_value  DOUBLE PRECISION NOT NULL DEFAULT 0,
    recorded_at   DATE NOT NULL DEFAULT CURRENT_DATE,
    metadata      JSONB,
    UNIQUE (metric_name, recorded_at)
);

CREATE INDEX idx_marketing_stats_metric ON marketing_stats (metric_name, recorded_at DESC);

-- Seed zero-cost marketing campaigns (5 phases, 11 campaigns)
-- Names MUST match the campaign names in service-marketing/src/scheduler/campaigns.rs

-- Phase: Launch (disabled by default — enable for launch day)
INSERT INTO marketing_campaigns (name, cron, channels, enabled, phase, description)
VALUES
    ('launch_hn',           '0 0 13 * * *',       ARRAY['hackernews'],                     false, 'launch',         'Launch Day 1: Show HN post (8am ET Tuesday/Wednesday)'),
    ('launch_reddit',       '0 0 15 * * *',       ARRAY['reddit'],                         false, 'launch',         'Launch Day 2: Reddit blitz — r/rust, r/LocalLLaMA, r/MachineLearning, r/selfhosted'),
    ('launch_devto',        '0 0 14 * * *',       ARRAY['devto','hashnode'],               false, 'launch',         'Launch Day 3: Dev.to + Hashnode article — Why We Rewrote in Rust'),
    ('launch_twitter',      '0 0 14 * * *',       ARRAY['twitter'],                        false, 'launch',         'Launch Day 4: Twitter/X launch thread'),
    ('launch_producthunt',  '0 0 12 * * *',       ARRAY['producthunt'],                    false, 'launch',         'Launch Day 5: Product Hunt listing (Category: Developer Tools > AI)')
ON CONFLICT (name) DO NOTHING;

-- Phase: Content Engine (daily/weekly recurring)
INSERT INTO marketing_campaigns (name, cron, channels, enabled, phase, description)
VALUES
    ('daily_twitter',       '0 0 14 * * *',       ARRAY['twitter'],                        true,  'content_engine', 'Daily tweet with day-of-week rotation: Mon=tip, Tue=benchmark, Wed=showcase, Thu=architecture, Fri=community'),
    ('weekly_content',      '0 0 15 * * TUE',     ARRAY['blog','devto','hashnode','medium','linkedin','reddit'], true, 'content_engine', 'Weekly blog + cross-post to Dev.to/Hashnode/Medium + LinkedIn + Reddit')
ON CONFLICT (name) DO NOTHING;

-- Phase: Community (weekly engagement)
INSERT INTO marketing_campaigns (name, cron, channels, enabled, phase, description)
VALUES
    ('mcp_content',         '0 0 15 * * WED',     ARRAY['devto','blog'],                   true,  'community',      'MCP ecosystem content: tutorials, integration guides')
ON CONFLICT (name) DO NOTHING;

-- Phase: Conversion (ongoing nurture)
INSERT INTO marketing_campaigns (name, cron, channels, enabled, phase, description)
VALUES
    ('monthly_newsletter',  '0 0 16 1 * *',       ARRAY['email','github','twitter'],       true,  'conversion',     'Monthly newsletter + GitHub metadata update + summary tweet'),
    ('seo_content',         '0 0 14 * * THU',     ARRAY['blog'],                           true,  'conversion',     'SEO-targeted blog posts: AI memory layer, Mem0 alternative, context engineering'),
    ('weekly_digest',       '0 0 10 * * FRI',     ARRAY['email'],                          true,  'conversion',     'Weekly usage digest email for self-hosted users (opt-in)')
ON CONFLICT (name) DO NOTHING;
