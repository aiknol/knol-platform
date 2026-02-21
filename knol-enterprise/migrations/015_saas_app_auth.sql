-- SaaS app authentication and tenant user management
-- Supports cloud.aiknol.com signup/login and per-tenant API key management.

CREATE TABLE IF NOT EXISTS app_users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email           TEXT NOT NULL,
    password_hash   TEXT NOT NULL,
    full_name       TEXT NOT NULL DEFAULT '',
    role            TEXT NOT NULL DEFAULT 'owner'
                    CHECK (role IN ('owner','admin','developer','viewer')),
    enabled         BOOLEAN NOT NULL DEFAULT true,
    last_login_at   TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (email)
);

CREATE INDEX IF NOT EXISTS idx_app_users_tenant ON app_users (tenant_id);
CREATE INDEX IF NOT EXISTS idx_app_users_email ON app_users (email);

CREATE TABLE IF NOT EXISTS app_sessions (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    app_user_id   UUID NOT NULL REFERENCES app_users(id) ON DELETE CASCADE,
    token_hash    TEXT NOT NULL UNIQUE,
    expires_at    TIMESTAMPTZ NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_app_sessions_user ON app_sessions (app_user_id);
CREATE INDEX IF NOT EXISTS idx_app_sessions_expiry ON app_sessions (expires_at);
