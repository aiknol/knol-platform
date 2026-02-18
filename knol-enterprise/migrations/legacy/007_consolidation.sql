-- Memory Consolidation Tracking Table
-- Tracks the relationship between episodic and semantic memories after consolidation

CREATE TABLE IF NOT EXISTS memory_consolidations (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  episodic_memory_id UUID NOT NULL REFERENCES memories(id),
  semantic_memory_id UUID NOT NULL REFERENCES memories(id),
  tenant_id       UUID NOT NULL REFERENCES tenants(id),
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_consolidations_episodic ON memory_consolidations(episodic_memory_id);
CREATE INDEX IF NOT EXISTS idx_consolidations_semantic ON memory_consolidations(semantic_memory_id);
CREATE INDEX IF NOT EXISTS idx_consolidations_tenant ON memory_consolidations(tenant_id);

ALTER TABLE memory_consolidations ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_consolidations ON memory_consolidations
  USING (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- Extension to memory_audit to include 'consolidate' action
ALTER TABLE memory_audit ADD CONSTRAINT audit_action_check
  CHECK (action IN ('create','update','delete','merge','supersede','restore','archive','decay','consolidate'))
  NOT VALID;

-- Extension to memories table status to include 'consolidated'
ALTER TABLE memories DROP CONSTRAINT memories_status_check;
ALTER TABLE memories ADD CONSTRAINT memories_status_check
  CHECK (status IN ('active','superseded','archived','deleted','consolidated'));

-- Table to track memory entities relationships (if not already present)
CREATE TABLE IF NOT EXISTS memory_entities (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  memory_id       UUID NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
  entity_id       UUID NOT NULL REFERENCES entities(id),
  relation_type   TEXT DEFAULT 'mentioned',
  confidence      REAL DEFAULT 0.8,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_memory_entities_memory ON memory_entities(memory_id);
CREATE INDEX IF NOT EXISTS idx_memory_entities_entity ON memory_entities(entity_id);
CREATE INDEX IF NOT EXISTS idx_memory_entities_tenant ON memory_entities(memory_id);

ALTER TABLE memory_entities ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_memory_entities ON memory_entities
  USING (memory_id IN (SELECT id FROM memories WHERE tenant_id = current_setting('app.tenant_id', true)::uuid));
