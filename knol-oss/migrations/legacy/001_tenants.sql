-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "vector";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- Tenants table
CREATE TABLE IF NOT EXISTS tenants (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name            TEXT NOT NULL,
  slug            TEXT NOT NULL UNIQUE,
  plan            TEXT NOT NULL DEFAULT 'free'
                  CHECK (plan IN ('free','developer','pro','team','enterprise')),
  config          JSONB NOT NULL DEFAULT '{
    "extraction_model": "claude-haiku-4-5-20251001",
    "embedding_model": "voyage-3-lite",
    "embedding_dim": 1024,
    "decay_lambda": 0.01,
    "importance_threshold": 0.1,
    "max_memories_per_user": 100000,
    "retention_days": null,
    "pii_redaction": false,
    "custom_ontology": null
  }'::jsonb,
  api_key_hash    TEXT NOT NULL,
  usage_ops_month INTEGER NOT NULL DEFAULT 0,
  usage_limit     INTEGER,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- RLS on tenants
ALTER TABLE tenants ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_self_access ON tenants
  USING (id = current_setting('app.tenant_id', true)::uuid);
