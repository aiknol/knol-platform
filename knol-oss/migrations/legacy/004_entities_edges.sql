CREATE TABLE IF NOT EXISTS entities (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL REFERENCES tenants(id),
  name            TEXT NOT NULL,
  entity_type     TEXT NOT NULL,
  summary         TEXT,
  attributes      JSONB DEFAULT '{}',
  embedding       vector(1024),
  valid_from      TIMESTAMPTZ NOT NULL DEFAULT now(),
  valid_to        TIMESTAMPTZ,
  status          TEXT NOT NULL DEFAULT 'active'
                  CHECK (status IN ('active','merged','deleted')),
  merged_into     UUID REFERENCES entities(id),
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_entities_tenant ON entities(tenant_id);
CREATE INDEX IF NOT EXISTS idx_entities_type ON entities(tenant_id, entity_type);
CREATE INDEX IF NOT EXISTS idx_entities_name ON entities(tenant_id, name);
CREATE INDEX IF NOT EXISTS idx_entities_embedding ON entities USING hnsw (embedding vector_cosine_ops);
CREATE UNIQUE INDEX IF NOT EXISTS idx_entities_dedup ON entities(tenant_id, name, entity_type) WHERE status = 'active';
-- Full-text search on entity names
CREATE INDEX IF NOT EXISTS idx_entities_fts ON entities USING GIN(to_tsvector('english', name || ' ' || COALESCE(summary, '')));

ALTER TABLE entities ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_entities ON entities
  USING (tenant_id = current_setting('app.tenant_id', true)::uuid);

CREATE TABLE IF NOT EXISTS edges (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL REFERENCES tenants(id),
  source_entity_id UUID NOT NULL REFERENCES entities(id),
  target_entity_id UUID NOT NULL REFERENCES entities(id),
  rel_type        TEXT NOT NULL,
  properties      JSONB DEFAULT '{}',
  weight          REAL NOT NULL DEFAULT 1.0,
  source_episode_id UUID REFERENCES episodes(id),
  valid_from      TIMESTAMPTZ NOT NULL DEFAULT now(),
  valid_to        TIMESTAMPTZ,
  status          TEXT NOT NULL DEFAULT 'active',
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(tenant_id, source_entity_id) WHERE status = 'active';
CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(tenant_id, target_entity_id) WHERE status = 'active';
CREATE INDEX IF NOT EXISTS idx_edges_rel ON edges(tenant_id, rel_type);
CREATE INDEX IF NOT EXISTS idx_edges_temporal ON edges(tenant_id, valid_from, valid_to);

ALTER TABLE edges ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_edges ON edges
  USING (tenant_id = current_setting('app.tenant_id', true)::uuid);
