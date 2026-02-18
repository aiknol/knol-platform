-- Consolidated enterprise baseline migration
-- Generated during development to reduce migration fan-out.

-- >>> BEGIN 006_audit_policies.sql
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

-- <<< END 006_audit_policies.sql

-- >>> BEGIN 007_consolidation.sql
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

-- <<< END 007_consolidation.sql

-- >>> BEGIN 008_marketing.sql
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

-- <<< END 008_marketing.sql

-- >>> BEGIN 009_admin_panel.sql
-- Admin panel: config store, encrypted credentials, admin users, audit log
-- Enables runtime configuration without redeployment

-- ─── System Configuration ────────────────────────────────────────────
-- Key-value store for runtime settings (replaces env vars)
CREATE TABLE IF NOT EXISTS system_config (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key          TEXT NOT NULL UNIQUE,
    value        JSONB NOT NULL DEFAULT '{}',
    value_type   TEXT NOT NULL DEFAULT 'string'
                     CHECK (value_type IN ('string','number','boolean','json','string_array')),
    category     TEXT NOT NULL DEFAULT 'general',
    description  TEXT NOT NULL DEFAULT '',
    env_override TEXT,                -- fallback env var name
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_system_config_category ON system_config (category);

-- ─── Encrypted Credentials ───────────────────────────────────────────
-- API keys and secrets encrypted with AES-256-GCM at rest
CREATE TABLE IF NOT EXISTS system_credentials (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT NOT NULL UNIQUE,
    encrypted_value BYTEA NOT NULL,
    service         TEXT NOT NULL DEFAULT 'general',
    description     TEXT NOT NULL DEFAULT '',
    last_rotated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_system_credentials_service ON system_credentials (service);

-- ─── Admin Users ─────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS admin_users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email         TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role          TEXT NOT NULL DEFAULT 'read_only'
                      CHECK (role IN ('super_admin','config_admin','marketing_admin','read_only')),
    enabled       BOOLEAN NOT NULL DEFAULT true,
    last_login_at TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ─── Admin Sessions ──────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS admin_sessions (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    admin_id   UUID NOT NULL REFERENCES admin_users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_admin_sessions_admin  ON admin_sessions (admin_id);
CREATE INDEX idx_admin_sessions_expiry ON admin_sessions (expires_at);

-- ─── Admin Audit Log ─────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS admin_audit_log (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    admin_id      UUID REFERENCES admin_users(id),
    admin_email   TEXT,
    action        TEXT NOT NULL CHECK (action IN ('create','update','delete','login','logout','test')),
    resource_type TEXT NOT NULL,        -- 'config', 'credential', 'campaign', 'tenant', 'user'
    resource_key  TEXT,
    old_value     JSONB,
    new_value     JSONB,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_admin_audit_time     ON admin_audit_log (created_at DESC);
CREATE INDEX idx_admin_audit_admin    ON admin_audit_log (admin_id);
CREATE INDEX idx_admin_audit_resource ON admin_audit_log (resource_type, resource_key);

-- ─── Seed Default Configs ────────────────────────────────────────────
-- These replace the env vars that used to configure these values

INSERT INTO system_config (key, value, value_type, category, description, env_override) VALUES
    -- Consolidation
    ('consolidation.min_age_hours',       '24',     'number', 'consolidation', 'Min hours before memory eligible for consolidation', 'CONSOLIDATION_MIN_AGE_HOURS'),
    ('consolidation.min_cluster_size',    '3',      'number', 'consolidation', 'Min memories to form a cluster',                     'CONSOLIDATION_MIN_CLUSTER_SIZE'),
    ('consolidation.max_per_run',         '100',    'number', 'consolidation', 'Max memories consolidated per job run',               'CONSOLIDATION_MAX_PER_RUN'),
    -- Conflict resolution
    ('conflict.supersede_days',           '7',      'number', 'conflict',      'Days before conflicting memory is superseded',        'CONFLICT_SUPERSEDE_DAYS'),
    ('conflict.duplicate_threshold',      '0.80',   'number', 'conflict',      'Similarity threshold for duplicate detection',        'CONFLICT_DUPLICATE_THRESHOLD'),
    ('conflict.contradiction_threshold',  '0.50',   'number', 'conflict',      'Similarity threshold for contradiction detection',    'CONFLICT_CONTRADICTION_THRESHOLD'),
    -- Decay
    ('decay.lambda',                      '0.01',   'number', 'retention',     'Exponential decay rate for memory importance',        'DECAY_LAMBDA'),
    ('decay.min_importance',              '0.05',   'number', 'retention',     'Floor importance — memories below this are archived',  'DECAY_MIN_IMPORTANCE'),
    ('retention.default_days',            '365',    'number', 'retention',     'Default retention period in days',                    'DEFAULT_RETENTION_DAYS'),
    -- Gateway
    ('gateway.default_plan_tier',    '"starter"',                   'string', 'gateway', 'Default plan tier for new tenants',             'DEFAULT_PLAN_TIER'),
    ('gateway.api_key_header',       '"X-API-Key"',                 'string', 'gateway', 'Header name for API key authentication',        'API_KEY_HEADER'),
    ('gateway.write_service_url',    '"http://write:8081"',         'string', 'gateway', 'Write service internal URL',                    'WRITE_SERVICE_URL'),
    ('gateway.retrieve_service_url', '"http://retrieve:8082"',      'string', 'gateway', 'Retrieve service internal URL',                 'RETRIEVE_SERVICE_URL'),
    ('gateway.admin_service_url',    '"http://admin:8084"',         'string', 'gateway', 'Admin service internal URL',                    'ADMIN_SERVICE_URL'),
    -- LLM
    ('llm.provider',          '"anthropic"',                 'string', 'llm', 'Active LLM provider (anthropic, openai, gemini)', 'LLM_PROVIDER'),
    ('llm.anthropic_model',   '"claude-3-haiku-20240307"',   'string', 'llm', 'Default Anthropic model for general use', 'ANTHROPIC_MODEL'),
    ('llm.extraction_model',  '"claude-haiku-4-5-20251001"', 'string', 'llm', 'Model for entity extraction (Anthropic)', 'EXTRACTION_MODEL'),
    ('llm.openai_model',      '"gpt-4o-mini"',               'string', 'llm', 'Model for extraction (OpenAI)',           'OPENAI_MODEL'),
    ('llm.openai_api_url',    '""',                           'string', 'llm', 'Custom OpenAI-compatible endpoint URL',  'OPENAI_API_URL'),
    ('llm.gemini_model',      '"gemini-2.0-flash"',           'string', 'llm', 'Model for extraction (Gemini)',           'GEMINI_MODEL'),
    ('llm.gemini_api_url',    '""',                           'string', 'llm', 'Custom Gemini/Vertex AI endpoint URL',   'GEMINI_API_URL'),
    ('llm.embedding_model',   '"voyage-3-lite"',             'string', 'llm', 'Model for embedding generation',          NULL),
    ('llm.embedding_dim',     '1024',                        'number', 'llm', 'Embedding vector dimensions',             NULL),
    -- Marketing rate limits (90% safety margins)
    ('marketing.twitter.daily_limit',    '45',   'number', 'marketing', 'Twitter posts per day (90% of 50)',     NULL),
    ('marketing.twitter.monthly_limit',  '1350', 'number', 'marketing', 'Twitter posts per month (90% of 1500)', NULL),
    ('marketing.linkedin.daily_limit',   '22',   'number', 'marketing', 'LinkedIn posts per day (90% of 25)',    NULL),
    ('marketing.reddit.daily_limit',     '9',    'number', 'marketing', 'Reddit posts per day (90% of 10)',      NULL),
    ('marketing.devto.daily_limit',      '27',   'number', 'marketing', 'Dev.to posts per day (90% of 30)',      NULL),
    ('marketing.email.daily_limit',      '400',  'number', 'marketing', 'Emails per day (90% of Gmail 500)',     NULL),
    -- Demo
    ('demo.enabled',          'true',                         'boolean', 'demo', 'Enable the public interactive demo',                    NULL),
    ('demo.llm_provider',     '"gemini"',                     'string',  'demo', 'LLM provider for the demo (gemini, openai, anthropic)', 'DEMO_LLM_PROVIDER'),
    ('demo.llm_model',        '""',                           'string',  'demo', 'Model override for demo (empty = use provider default)', 'DEMO_LLM_MODEL'),
    ('demo.admin_api_url',    '"http://localhost:8084"',      'string',  'demo', 'Admin API URL the demo fetches config from',             'DEMO_ADMIN_API_URL'),
    ('demo.github_url',       '"https://github.com/pankajb64/memorylayer"', 'string', 'demo', 'CTA link URL in the demo',                NULL),
    ('demo.tagline',          '"Give your AI persistent memory"', 'string', 'demo', 'Headline on demo welcome screen',                    NULL),
    -- Guardrails
    ('guardrails.redact_pii',                'true',    'boolean',      'guardrails', 'Enable PII detection and redaction in extracted memories',     NULL),
    ('guardrails.pii_mode',                  '"redact"','string',       'guardrails', 'PII handling mode: redact, mask, hash, or allow',              NULL),
    ('guardrails.strict_memory_types',       'true',    'boolean',      'guardrails', 'Normalize unrecognized memory types to valid enum values',     NULL),
    ('guardrails.strict_entity_types',       'true',    'boolean',      'guardrails', 'Normalize unrecognized entity types to valid enum values',     NULL),
    ('guardrails.max_memory_content_len',    '2000',    'number',       'guardrails', 'Max characters per memory content string',                     NULL),
    ('guardrails.max_entity_name_len',       '200',     'number',       'guardrails', 'Max characters for entity names',                              NULL),
    ('guardrails.max_memories_per_extraction','50',      'number',       'guardrails', 'Max memories allowed per single extraction',                   NULL),
    ('guardrails.max_entities_per_extraction','100',     'number',       'guardrails', 'Max entities allowed per single extraction',                   NULL),
    ('guardrails.min_confidence',            '0.0',     'number',       'guardrails', 'Drop memories with confidence below this threshold (0.0-1.0)', NULL),
    ('guardrails.detect_prompt_injection',   'true',    'boolean',      'guardrails', 'Block inputs that look like prompt injection attempts',         NULL),
    ('guardrails.max_input_content_len',     '50000',   'number',       'guardrails', 'Max bytes for input content before it reaches the LLM',        NULL),
    ('guardrails.blocked_keywords',          '[]',      'string_array', 'guardrails', 'Drop memories containing any of these keywords',               NULL)
ON CONFLICT (key) DO NOTHING;

-- Note: Initial admin user is created at first startup via ADMIN_INITIAL_PASSWORD env var
-- The service-admin app handles hashing and inserting the first super_admin user

-- <<< END 009_admin_panel.sql

-- >>> BEGIN 010_grounding.sql
-- Migration 010: Grounding System
-- Adds citation grounding (source quotes), factual grounding (verification),
-- and the memory_citations table for multi-source citation linking.

-- ── Add grounding columns to memories ──────────────────────────────────────

ALTER TABLE memories
  ADD COLUMN IF NOT EXISTS source_quote       TEXT,
  ADD COLUMN IF NOT EXISTS source_offset_start INTEGER,
  ADD COLUMN IF NOT EXISTS source_offset_end   INTEGER,
  ADD COLUMN IF NOT EXISTS verification_status TEXT NOT NULL DEFAULT 'unverified'
    CHECK (verification_status IN ('unverified', 'verified', 'contested', 'failed')),
  ADD COLUMN IF NOT EXISTS verification_score  REAL;

CREATE INDEX IF NOT EXISTS idx_memories_verification_status
  ON memories (verification_status) WHERE verification_status != 'unverified';

-- ── Memory citations table ─────────────────────────────────────────────────
-- Links a memory to one or more source episodes with exact quotes and offsets.

CREATE TABLE IF NOT EXISTS memory_citations (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  memory_id         UUID NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
  episode_id        UUID NOT NULL REFERENCES episodes(id) ON DELETE CASCADE,
  source_quote      TEXT NOT NULL,
  offset_start      INTEGER,
  offset_end        INTEGER,
  confidence        REAL DEFAULT 1.0 CHECK (confidence >= 0 AND confidence <= 1),
  created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_memory_citations_memory
  ON memory_citations (memory_id);
CREATE INDEX IF NOT EXISTS idx_memory_citations_episode
  ON memory_citations (episode_id);

-- ── Seed grounding config keys into system_config ──────────────────────────

INSERT INTO system_config (key, value, value_type, category, description, env_override)
VALUES
  ('grounding.enable_citations',      'true',        'boolean', 'grounding',
   'Enable citation grounding — extract source quotes for each memory', ''),
  ('grounding.enable_verification',   'false',       'boolean', 'grounding',
   'Enable factual verification — second LLM pass to verify extracted memories', ''),
  ('grounding.verification_model',    '"same"',      'string',  'grounding',
   'Model to use for verification (\"same\" = use extraction model)', ''),
  ('grounding.min_verification_score', '0.5',        'number',  'grounding',
   'Minimum verification score to mark a memory as verified (0.0-1.0)', '')
ON CONFLICT (key) DO NOTHING;

-- <<< END 010_grounding.sql

-- >>> BEGIN 011_env_consolidation.sql
-- Migration 011: Environment Consolidation
-- Moves service ports, MINIO settings, CORS config, and other env-only vars
-- into admin-configurable system_config with env var fallback.

-- ── Service Ports ───────────────────────────────────────────────────────────
INSERT INTO system_config (key, value, value_type, category, description, env_override)
VALUES
  ('services.gateway_port',       '8080', 'number', 'services',
   'Gateway service listen port',                          'GATEWAY_PORT'),
  ('services.write_port',         '8081', 'number', 'services',
   'Write service listen port',                            'WRITE_SERVICE_PORT'),
  ('services.retrieve_port',      '8082', 'number', 'services',
   'Retrieve service listen port',                         'RETRIEVE_SERVICE_PORT'),
  ('services.graph_port',         '8083', 'number', 'services',
   'Graph service listen port',                            'GRAPH_SERVICE_PORT'),
  ('services.admin_port',         '8084', 'number', 'services',
   'Admin service listen port',                            'ADMIN_SERVICE_PORT'),
  ('services.admin_panel_port',   '8084', 'number', 'services',
   'Admin panel proxy service listen port',                'ADMIN_PANEL_SERVICE_PORT'),
  ('services.billing_port',       '8086', 'number', 'services',
   'Billing service listen port',                          'BILLING_SERVICE_PORT'),
  ('services.ingest_port',        '8087', 'number', 'services',
   'Ingest service listen port',                           'INGEST_SERVICE_PORT'),

  -- CORS
  ('services.admin_cors_origin',  '"http://localhost:3006"', 'string', 'services',
   'Allowed CORS origin for admin panel',                  'ADMIN_CORS_ORIGIN'),

  -- MinIO / S3
  ('storage.minio_endpoint',      '"http://localhost:9000"', 'string', 'storage',
   'MinIO/S3 endpoint URL',                                'MINIO_ENDPOINT'),
  ('storage.minio_bucket',        '"memorylayer"',           'string', 'storage',
   'MinIO/S3 bucket name',                                 'MINIO_BUCKET'),

  -- Database pool
  ('database.max_connections',    '20',   'number', 'database',
   'Maximum database pool connections',                    'DATABASE_MAX_CONNECTIONS'),
  ('database.min_connections',    '2',    'number', 'database',
   'Minimum database pool connections',                    'DATABASE_MIN_CONNECTIONS')
ON CONFLICT (key) DO NOTHING;

-- <<< END 011_env_consolidation.sql

-- >>> BEGIN 012_llm_optimization.sql
-- Migration 012: LLM Optimization
-- Adds triage, caching, inline verification, and token usage tracking.

-- ── New config keys ──

INSERT INTO system_config (key, value, description, category, env_override)
VALUES
    -- Content triage: skip trivial content before LLM calls
    ('llm.enable_triage', 'true', 'Enable content triage to skip trivial messages (greetings, acks) before LLM extraction', 'llm', 'LLM_ENABLE_TRIAGE'),
    ('llm.triage_min_words', '3', 'Minimum word count for content to be sent to LLM', 'llm', 'LLM_TRIAGE_MIN_WORDS'),
    ('llm.triage_light_threshold', '15', 'Word count below which extraction uses reduced output budget', 'llm', 'LLM_TRIAGE_LIGHT_THRESHOLD'),

    -- Entity context pruning
    ('llm.max_entity_context', '20', 'Maximum entity names in extraction prompt (pruned by relevance)', 'llm', 'LLM_MAX_ENTITY_CONTEXT'),

    -- Dynamic output tokens
    ('llm.dynamic_output_tokens', 'true', 'Scale max_output_tokens by content length (1024/2048/4096)', 'llm', 'LLM_DYNAMIC_OUTPUT_TOKENS'),

    -- Redis LLM response cache
    ('llm.cache_enabled', 'true', 'Cache LLM extraction results in Redis to avoid duplicate calls', 'llm', 'LLM_CACHE_ENABLED'),
    ('llm.cache_ttl_secs', '3600', 'TTL in seconds for cached LLM extraction results', 'llm', 'LLM_CACHE_TTL_SECS'),

    -- Inline verification (merge grounding into extraction prompt)
    ('grounding.inline_verification', 'true', 'Embed grounding fields in extraction prompt to eliminate separate verification call', 'grounding', 'GROUNDING_INLINE_VERIFICATION')

ON CONFLICT (key) DO NOTHING;

-- ── Token usage log table ──

CREATE TABLE IF NOT EXISTS llm_usage_log (
    id            BIGSERIAL PRIMARY KEY,
    tenant_id     UUID        NOT NULL,
    provider      TEXT        NOT NULL,  -- 'anthropic', 'openai', 'gemini'
    model         TEXT        NOT NULL,
    call_type     TEXT        NOT NULL,  -- 'extraction' or 'verification'
    input_tokens  INTEGER     NOT NULL DEFAULT 0,
    output_tokens INTEGER     NOT NULL DEFAULT 0,
    total_tokens  INTEGER     NOT NULL DEFAULT 0,
    cache_hit     BOOLEAN     NOT NULL DEFAULT false,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Index for per-tenant cost queries
CREATE INDEX IF NOT EXISTS idx_llm_usage_log_tenant
    ON llm_usage_log (tenant_id, created_at DESC);

-- Index for aggregate monitoring dashboards
CREATE INDEX IF NOT EXISTS idx_llm_usage_log_created
    ON llm_usage_log (created_at DESC);

-- <<< END 012_llm_optimization.sql

-- >>> BEGIN 013_competitive_features.sql
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

-- <<< END 013_competitive_features.sql

-- >>> BEGIN 014_remove_plaintext_sensitive_config.sql
-- Remove sensitive plaintext config keys from system_config.
-- Secrets must be provided via environment variables or encrypted system_credentials.

DELETE FROM system_config
WHERE key IN (
  'gateway.jwt_secret',
  'storage.minio_access_key',
  'storage.minio_secret_key'
);

-- <<< END 014_remove_plaintext_sensitive_config.sql

