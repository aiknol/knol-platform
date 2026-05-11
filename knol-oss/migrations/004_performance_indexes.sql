-- Migration 004: Performance indexes
-- Composite indexes for hot query paths identified during performance audit.

-- Fast lookup for active memories by tenant + kind (used in search, export, counts).
CREATE INDEX IF NOT EXISTS idx_memories_tenant_kind_active
    ON memories (tenant_id, kind, created_at DESC)
    WHERE status = 'active';

-- Fast entity lookup by tenant + name + type (used by entity context pruning).
CREATE INDEX IF NOT EXISTS idx_entities_tenant_name_type
    ON entities (tenant_id, name, entity_type);

-- Fast edge traversal by source entity (used by N-hop graph queries).
CREATE INDEX IF NOT EXISTS idx_edges_source
    ON edges (source_entity_id, rel_type);

-- Fast edge traversal by target entity (reverse traversal).
CREATE INDEX IF NOT EXISTS idx_edges_target
    ON edges (target_entity_id, rel_type);

-- Optimise tenant API key auth (hot path on every gateway request).
CREATE INDEX IF NOT EXISTS idx_tenant_api_keys_hash_active
    ON tenant_api_keys (key_hash)
    WHERE active = true;
