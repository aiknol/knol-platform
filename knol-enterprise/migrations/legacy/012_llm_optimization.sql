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
