//! Admin audit log — searchable history of all admin actions.

use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::{AdminClaims, AdminError};
use crate::AdminAppState;

#[derive(Deserialize)]
pub struct AuditFilter {
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_key: Option<String>,
    pub admin_id: Option<Uuid>,
    pub limit: Option<i64>,
}

pub async fn list_audit(
    State(state): State<Arc<AdminAppState>>,
    _claims: AdminClaims,
    Query(filter): Query<AuditFilter>,
) -> Result<Json<Vec<serde_json::Value>>, AdminError> {
    let limit = filter.limit.unwrap_or(100).min(500);

    let rows = sqlx::query_as::<_, AuditLogRow>(
        r#"
        SELECT id, admin_id, admin_email, action, resource_type, resource_key,
               old_value, new_value, created_at
        FROM admin_audit_log
        WHERE ($1::text IS NULL OR action = $1)
          AND ($2::text IS NULL OR resource_type = $2)
          AND ($3::text IS NULL OR resource_key = $3)
          AND ($4::uuid IS NULL OR admin_id = $4)
        ORDER BY created_at DESC
        LIMIT $5
        "#,
    )
    .bind(&filter.action)
    .bind(&filter.resource_type)
    .bind(&filter.resource_key)
    .bind(filter.admin_id)
    .bind(limit)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Internal error: {}", e);
        AdminError::Internal("Internal server error".into())
    })?;

    let json: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "admin_id": r.admin_id,
                "admin_email": r.admin_email,
                "action": r.action,
                "resource_type": r.resource_type,
                "resource_key": r.resource_key,
                "old_value": r.old_value,
                "new_value": r.new_value,
                "created_at": r.created_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(json))
}

#[derive(sqlx::FromRow)]
struct AuditLogRow {
    id: Uuid,
    admin_id: Option<Uuid>,
    admin_email: Option<String>,
    action: String,
    resource_type: String,
    resource_key: Option<String>,
    old_value: Option<serde_json::Value>,
    new_value: Option<serde_json::Value>,
    created_at: chrono::DateTime<chrono::Utc>,
}
