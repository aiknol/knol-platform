CREATE TABLE IF NOT EXISTS memory_audit (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL REFERENCES tenants(id),
  memory_id       UUID NOT NULL,
  target_table    TEXT NOT NULL,
  action          TEXT NOT NULL
                  CHECK (action IN ('create','update','delete','merge','supersede','restore','archive','decay')),
  actor_type      TEXT NOT NULL
                  CHECK (actor_type IN ('system','user','admin','connector')),
  actor_id        TEXT,
  diff            JSONB,
  reason          TEXT,
  timestamp       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_audit_memory ON memory_audit(tenant_id, memory_id);
CREATE INDEX IF NOT EXISTS idx_audit_time ON memory_audit(tenant_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_actor ON memory_audit(tenant_id, actor_type, actor_id);

ALTER TABLE memory_audit ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_audit ON memory_audit
  USING (tenant_id = current_setting('app.tenant_id', true)::uuid);

CREATE TABLE IF NOT EXISTS memory_policies (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL REFERENCES tenants(id),
  name            TEXT NOT NULL,
  rule_type       TEXT NOT NULL
                  CHECK (rule_type IN ('retention','redaction','scope_access','auto_classify','auto_expire','pii_filter')),
  config          JSONB NOT NULL,
  enabled         BOOLEAN NOT NULL DEFAULT true,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE memory_policies ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_policies ON memory_policies
  USING (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- Usage events table for billing
CREATE TABLE IF NOT EXISTS usage_events (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL REFERENCES tenants(id),
  event_type      TEXT NOT NULL CHECK (event_type IN ('write','search','extract','admin')),
  count           INTEGER NOT NULL DEFAULT 1,
  metadata        JSONB DEFAULT '{}',
  timestamp       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_usage_tenant ON usage_events(tenant_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_usage_type ON usage_events(tenant_id, event_type);
