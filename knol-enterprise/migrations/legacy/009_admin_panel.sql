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
