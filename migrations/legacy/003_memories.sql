CREATE TABLE IF NOT EXISTS memories (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL REFERENCES tenants(id),
  user_id         UUID,
  scope           TEXT NOT NULL DEFAULT 'user'
                  CHECK (scope IN ('user','team','project','agent','org')),
  kind            TEXT NOT NULL
                  CHECK (kind IN ('preference','fact','task','event','relationship','summary','procedure')),
  content         TEXT NOT NULL,
  content_json    JSONB,
  confidence      REAL NOT NULL DEFAULT 0.8 CHECK (confidence >= 0 AND confidence <= 1),
  importance      REAL NOT NULL DEFAULT 0.5 CHECK (importance >= 0 AND importance <= 1),
  status          TEXT NOT NULL DEFAULT 'active'
                  CHECK (status IN ('active','superseded','archived','deleted')),
  valid_from      TIMESTAMPTZ NOT NULL DEFAULT now(),
  valid_to        TIMESTAMPTZ,
  event_time      TIMESTAMPTZ,
  ingested_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  source_episode_id UUID REFERENCES episodes(id),
  created_by      TEXT NOT NULL DEFAULT 'system'
                  CHECK (created_by IN ('system','user','admin','connector')),
  tags            TEXT[] DEFAULT '{}',
  metadata        JSONB DEFAULT '{}',
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_memories_tenant_user ON memories(tenant_id, user_id);
CREATE INDEX IF NOT EXISTS idx_memories_scope ON memories(tenant_id, scope);
CREATE INDEX IF NOT EXISTS idx_memories_kind ON memories(tenant_id, kind);
CREATE INDEX IF NOT EXISTS idx_memories_status ON memories(tenant_id, status) WHERE status = 'active';
CREATE INDEX IF NOT EXISTS idx_memories_valid ON memories(tenant_id, valid_from, valid_to);
CREATE INDEX IF NOT EXISTS idx_memories_tags ON memories USING GIN(tags);
CREATE INDEX IF NOT EXISTS idx_memories_metadata ON memories USING GIN(metadata);
-- Full-text search index for BM25
CREATE INDEX IF NOT EXISTS idx_memories_fts ON memories USING GIN(to_tsvector('english', content));

ALTER TABLE memories ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_memories ON memories
  USING (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- Memory vectors table (pgvector)
CREATE TABLE IF NOT EXISTS memory_vectors (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  memory_id       UUID NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
  tenant_id       UUID NOT NULL,
  user_id         UUID,
  scope           TEXT NOT NULL,
  kind            TEXT NOT NULL,
  status          TEXT NOT NULL DEFAULT 'active',
  valid_from      TIMESTAMPTZ NOT NULL,
  valid_to        TIMESTAMPTZ,
  embedding       vector(1024) NOT NULL,
  content_hash    TEXT NOT NULL,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_vectors_embedding ON memory_vectors
  USING hnsw (embedding vector_cosine_ops) WITH (m = 16, ef_construction = 200);
CREATE INDEX IF NOT EXISTS idx_vectors_tenant ON memory_vectors(tenant_id, status);
CREATE INDEX IF NOT EXISTS idx_vectors_dedup ON memory_vectors(tenant_id, content_hash);

ALTER TABLE memory_vectors ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_vectors ON memory_vectors
  USING (tenant_id = current_setting('app.tenant_id', true)::uuid);
