-- 018_security_hardening.sql
-- Per-account lockout, password reset tokens, session metadata

-- Per-account lockout columns
ALTER TABLE app_users ADD COLUMN IF NOT EXISTS failed_login_attempts INTEGER NOT NULL DEFAULT 0;
ALTER TABLE app_users ADD COLUMN IF NOT EXISTS locked_until TIMESTAMPTZ;

-- Password reset tokens (admin-initiated, no email infrastructure)
CREATE TABLE IF NOT EXISTS password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    app_user_id UUID NOT NULL REFERENCES app_users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_password_reset_token_hash ON password_reset_tokens(token_hash);

-- Session metadata for listing/revocation
ALTER TABLE app_sessions ADD COLUMN IF NOT EXISTS ip_address TEXT;
ALTER TABLE app_sessions ADD COLUMN IF NOT EXISTS user_agent TEXT;
