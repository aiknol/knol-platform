-- Marketing service tables
-- Campaign definitions, publish audit log, admin actions

CREATE TABLE IF NOT EXISTS marketing_campaigns (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(64) NOT NULL UNIQUE,
    cron        VARCHAR(64) NOT NULL,
    channels    TEXT[] NOT NULL DEFAULT '{}',
    enabled     BOOLEAN NOT NULL DEFAULT true,
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

-- Seed default campaigns
INSERT INTO marketing_campaigns (name, cron, channels, enabled)
VALUES
    ('daily',   '0 0 14 * * *',     ARRAY['twitter'], true),
    ('weekly',  '0 0 15 * * TUE',   ARRAY['blog','devto','linkedin','reddit'], true),
    ('monthly', '0 0 16 1 * *',     ARRAY['email','github','twitter'], true)
ON CONFLICT (name) DO NOTHING;
