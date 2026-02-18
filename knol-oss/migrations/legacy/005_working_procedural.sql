CREATE TABLE IF NOT EXISTS working_memory (
  session_id      TEXT NOT NULL,
  tenant_id       UUID NOT NULL REFERENCES tenants(id),
  user_id         UUID,
  agent_id        TEXT,
  summary         TEXT,
  active_facts    UUID[] DEFAULT '{}',
  active_procs    UUID[] DEFAULT '{}',
  scratchpad      JSONB DEFAULT '{}',
  turn_count      INTEGER NOT NULL DEFAULT 0,
  last_updated    TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, session_id)
);

ALTER TABLE working_memory ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_working ON working_memory
  USING (tenant_id = current_setting('app.tenant_id', true)::uuid);

CREATE TABLE IF NOT EXISTS procedural_memories (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL REFERENCES tenants(id),
  scope           TEXT NOT NULL DEFAULT 'user',
  scope_id        TEXT,
  user_id         UUID,
  agent_id        TEXT,
  description     TEXT NOT NULL,
  trigger_condition TEXT,
  procedure_steps TEXT NOT NULL,
  success_count   INTEGER NOT NULL DEFAULT 0,
  fail_count      INTEGER NOT NULL DEFAULT 0,
  embedding       vector(1024),
  status          TEXT NOT NULL DEFAULT 'active',
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_used       TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_procs_tenant ON procedural_memories(tenant_id, status);
CREATE INDEX IF NOT EXISTS idx_procs_embedding ON procedural_memories USING hnsw (embedding vector_cosine_ops);

ALTER TABLE procedural_memories ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_procs ON procedural_memories
  USING (tenant_id = current_setting('app.tenant_id', true)::uuid);
