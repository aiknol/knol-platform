CREATE TABLE IF NOT EXISTS episodes (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL REFERENCES tenants(id),
  user_id         UUID,
  session_id      TEXT,
  agent_id        TEXT,
  content         TEXT NOT NULL,
  role            TEXT NOT NULL CHECK (role IN ('user','assistant','system','tool')),
  event_time      TIMESTAMPTZ NOT NULL DEFAULT now(),
  ingested_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  storage_key     TEXT,
  content_hash    TEXT NOT NULL,
  metadata        JSONB DEFAULT '{}',
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_episodes_tenant_user ON episodes(tenant_id, user_id);
CREATE INDEX IF NOT EXISTS idx_episodes_session ON episodes(tenant_id, session_id);
CREATE INDEX IF NOT EXISTS idx_episodes_time ON episodes(tenant_id, event_time DESC);
CREATE INDEX IF NOT EXISTS idx_episodes_hash ON episodes(tenant_id, content_hash);

ALTER TABLE episodes ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_episodes ON episodes
  USING (tenant_id = current_setting('app.tenant_id', true)::uuid);
