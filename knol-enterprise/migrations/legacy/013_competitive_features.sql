-- Migration 013: Competitive features (embedding, decay, conflict, webhooks, export)
-- These features give Knol an edge over Mem0 and Zep.

-- ── Embedding Configuration ──
INSERT INTO system_config (key, value, description) VALUES
    ('embedding.provider', '"openai"', 'Embedding provider: openai, voyage, gemini, local'),
    ('embedding.model', '"text-embedding-3-small"', 'Embedding model name'),
    ('embedding.dimensions', '1024', 'Embedding vector dimensions'),
    ('embedding.cache_enabled', 'true', 'Enable in-memory embedding cache'),
    ('embedding.cache_max_entries', '10000', 'Max cached embeddings')
ON CONFLICT (key) DO NOTHING;

-- ── Memory Decay Configuration ──
INSERT INTO system_config (key, value, description) VALUES
    ('memory.decay_enabled', 'true', 'Enable time-based importance decay'),
    ('memory.decay_function', '"exponential"', 'Decay function: exponential, linear, step'),
    ('memory.decay_half_life_hours', '168', 'Half-life in hours (7 days default)'),
    ('memory.decay_min_score', '0.05', 'Minimum importance score floor'),
    ('memory.access_boost', '0.05', 'Importance boost when memory is retrieved')
ON CONFLICT (key) DO NOTHING;

-- Add last_accessed_at column for decay tracking
ALTER TABLE memories ADD COLUMN IF NOT EXISTS last_accessed_at TIMESTAMPTZ;
CREATE INDEX IF NOT EXISTS idx_memories_last_accessed ON memories (last_accessed_at)
    WHERE last_accessed_at IS NOT NULL;

-- ── Conflict Detection Configuration ──
INSERT INTO system_config (key, value, description) VALUES
    ('memory.conflict_detection_enabled', 'true', 'Enable conflict detection between memories'),
    ('memory.conflict_similarity_threshold', '0.80', 'Cosine similarity threshold for conflicts'),
    ('memory.conflict_entity_overlap', '0.70', 'Entity overlap ratio threshold'),
    ('memory.conflict_resolution', '"newest_wins"', 'Resolution: newest_wins, highest_confidence, manual_review')
ON CONFLICT (key) DO NOTHING;

-- Conflict log table
CREATE TABLE IF NOT EXISTS memory_conflicts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    existing_memory_id UUID NOT NULL,
    new_memory_id UUID,
    conflict_type TEXT NOT NULL, -- contradiction, duplicate, refinement
    similarity REAL NOT NULL,
    shared_entities TEXT[] DEFAULT '{}',
    resolution TEXT NOT NULL, -- supersede, skip_new, merge, review
    resolved BOOLEAN DEFAULT false,
    resolved_by TEXT,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_memory_conflicts_tenant ON memory_conflicts (tenant_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_memory_conflicts_unresolved ON memory_conflicts (tenant_id) WHERE NOT resolved;

-- ── Webhook System ──
CREATE TABLE IF NOT EXISTS webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    url TEXT NOT NULL,
    secret TEXT, -- HMAC-SHA256 secret
    event_types TEXT[] NOT NULL DEFAULT '{"*"}', -- Array of event type strings
    active BOOLEAN NOT NULL DEFAULT true,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_webhooks_tenant ON webhooks (tenant_id) WHERE active = true;

CREATE TABLE IF NOT EXISTS webhook_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    webhook_id UUID NOT NULL REFERENCES webhooks(id) ON DELETE CASCADE,
    event_id UUID NOT NULL,
    event_type TEXT NOT NULL,
    status_code SMALLINT,
    success BOOLEAN NOT NULL,
    attempt INTEGER NOT NULL DEFAULT 0,
    error TEXT,
    delivered_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_webhook ON webhook_deliveries (webhook_id, delivered_at DESC);

INSERT INTO system_config (key, value, description) VALUES
    ('webhook.enabled', 'true', 'Enable webhook event notifications'),
    ('webhook.max_retries', '3', 'Maximum delivery retry attempts'),
    ('webhook.timeout_secs', '10', 'Webhook delivery timeout')
ON CONFLICT (key) DO NOTHING;

-- ── Export/Import Tracking ──
CREATE TABLE IF NOT EXISTS memory_exports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    format TEXT NOT NULL DEFAULT 'json',
    total_memories INTEGER NOT NULL DEFAULT 0,
    total_entities INTEGER NOT NULL DEFAULT 0,
    total_edges INTEGER NOT NULL DEFAULT 0,
    duration_ms INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_memory_exports_tenant ON memory_exports (tenant_id, created_at DESC);
