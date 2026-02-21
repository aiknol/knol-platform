-- Tenant-facing audit logs for SaaS app actions.

CREATE TABLE IF NOT EXISTS tenant_audit_log (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    app_user_id     UUID REFERENCES app_users(id) ON DELETE SET NULL,
    app_user_email  TEXT,
    action          TEXT NOT NULL,
    resource_type   TEXT NOT NULL,
    resource_key    TEXT,
    old_value       JSONB,
    new_value       JSONB,
    metadata        JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_tenant_audit_tenant_created
    ON tenant_audit_log (tenant_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_tenant_audit_user
    ON tenant_audit_log (app_user_id)
    WHERE app_user_id IS NOT NULL;
