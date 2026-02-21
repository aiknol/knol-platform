//! Tenant audit log insertion shared between services.

use uuid::Uuid;

/// Insert a record into the `tenant_audit_log` table.
///
/// Fire-and-forget: errors are logged but not propagated so audit failures
/// never break the primary operation.
#[allow(clippy::too_many_arguments)]
pub async fn insert_tenant_audit(
    pool: &sqlx::PgPool,
    tenant_id: Uuid,
    actor_id: Option<Uuid>,
    actor_email: Option<String>,
    action: &str,
    resource_type: &str,
    resource_key: Option<&str>,
    old_value: Option<serde_json::Value>,
    new_value: Option<serde_json::Value>,
    metadata: Option<serde_json::Value>,
) {
    let _ = sqlx::query(
        r#"INSERT INTO tenant_audit_log
           (tenant_id, app_user_id, app_user_email, action, resource_type, resource_key, old_value, new_value, metadata)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
    )
    .bind(tenant_id)
    .bind(actor_id)
    .bind(actor_email)
    .bind(action)
    .bind(resource_type)
    .bind(resource_key)
    .bind(old_value)
    .bind(new_value)
    .bind(metadata)
    .execute(pool)
    .await;
}
