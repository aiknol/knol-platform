-- Stripe billing, team invites, and usage alerts for the SaaS tenant app.

-- Add Stripe columns to tenants
ALTER TABLE tenants
  ADD COLUMN IF NOT EXISTS stripe_customer_id    TEXT UNIQUE,
  ADD COLUMN IF NOT EXISTS stripe_subscription_id TEXT UNIQUE,
  ADD COLUMN IF NOT EXISTS subscription_status    TEXT NOT NULL DEFAULT 'none'
      CHECK (subscription_status IN ('none','trialing','active','past_due','canceled','unpaid')),
  ADD COLUMN IF NOT EXISTS billing_period_start   TIMESTAMPTZ,
  ADD COLUMN IF NOT EXISTS billing_period_end     TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_tenants_stripe_customer
    ON tenants (stripe_customer_id) WHERE stripe_customer_id IS NOT NULL;

-- Update plan CHECK constraint to include 'builder' and 'growth'
ALTER TABLE tenants DROP CONSTRAINT IF EXISTS tenants_plan_check;
ALTER TABLE tenants ADD CONSTRAINT tenants_plan_check
    CHECK (plan IN ('free','builder','growth','developer','pro','team','enterprise'))
    NOT VALID;

-- Stripe event log (idempotent webhook processing)
CREATE TABLE IF NOT EXISTS stripe_event_log (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stripe_event_id TEXT NOT NULL UNIQUE,
    event_type      TEXT NOT NULL,
    tenant_id       UUID REFERENCES tenants(id) ON DELETE SET NULL,
    processed       BOOLEAN NOT NULL DEFAULT false,
    payload         JSONB,
    error           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_stripe_event_log_event_id
    ON stripe_event_log (stripe_event_id);
CREATE INDEX IF NOT EXISTS idx_stripe_event_log_tenant
    ON stripe_event_log (tenant_id, created_at DESC);

-- Team invitations
CREATE TABLE IF NOT EXISTS team_invites (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email           TEXT NOT NULL,
    role            TEXT NOT NULL DEFAULT 'developer'
                    CHECK (role IN ('admin','developer','viewer')),
    invited_by      UUID NOT NULL REFERENCES app_users(id),
    token_hash      TEXT NOT NULL UNIQUE,
    status          TEXT NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending','accepted','revoked','expired')),
    expires_at      TIMESTAMPTZ NOT NULL,
    accepted_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_team_invites_tenant
    ON team_invites (tenant_id, status);
CREATE INDEX IF NOT EXISTS idx_team_invites_email
    ON team_invites (email, status);
CREATE INDEX IF NOT EXISTS idx_team_invites_token
    ON team_invites (token_hash) WHERE status = 'pending';

-- Usage alert thresholds (prevents duplicate alerts per threshold per month)
CREATE TABLE IF NOT EXISTS usage_alerts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    threshold_pct   INTEGER NOT NULL CHECK (threshold_pct IN (50, 80, 100)),
    month           TEXT NOT NULL,
    alerted_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, threshold_pct, month)
);

CREATE INDEX IF NOT EXISTS idx_usage_alerts_tenant
    ON usage_alerts (tenant_id, month);

-- Monthly usage history (aggregated snapshots)
CREATE TABLE IF NOT EXISTS usage_history (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    month           TEXT NOT NULL,
    ops_count       INTEGER NOT NULL DEFAULT 0,
    plan            TEXT NOT NULL,
    usage_limit     INTEGER,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, month)
);

CREATE INDEX IF NOT EXISTS idx_usage_history_tenant
    ON usage_history (tenant_id, month DESC);
