-- 019_features_hardening.sql
-- Session timeout, email verification, TOTP 2FA, GDPR account deletion

-- Idle session timeout tracking
ALTER TABLE app_sessions ADD COLUMN IF NOT EXISTS last_activity_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
CREATE INDEX IF NOT EXISTS idx_app_sessions_last_activity ON app_sessions(last_activity_at);

-- Email verification
ALTER TABLE app_users ADD COLUMN IF NOT EXISTS email_verified BOOLEAN NOT NULL DEFAULT false;

-- TOTP 2FA
ALTER TABLE app_users ADD COLUMN IF NOT EXISTS totp_secret_encrypted TEXT;
ALTER TABLE app_users ADD COLUMN IF NOT EXISTS totp_enabled BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE app_users ADD COLUMN IF NOT EXISTS totp_backup_codes TEXT[];

-- GDPR account deletion
ALTER TABLE app_users ADD COLUMN IF NOT EXISTS deletion_requested_at TIMESTAMPTZ;
ALTER TABLE app_users ADD COLUMN IF NOT EXISTS deletion_scheduled_for TIMESTAMPTZ;
