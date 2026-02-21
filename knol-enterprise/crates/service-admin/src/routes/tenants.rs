//! Tenant management — view and update tenant configs and plans.

use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::{AdminClaims, AdminError};
use crate::AdminAppState;

pub async fn list_tenants(
    State(state): State<Arc<AdminAppState>>,
    _claims: AdminClaims,
) -> Result<Json<Vec<serde_json::Value>>, AdminError> {
    let rows = sqlx::query_as::<_, TenantRow>(
        r#"SELECT id, name, plan, config, api_key_hash, usage_ops_month, usage_limit,
                  created_at, updated_at
           FROM tenants ORDER BY created_at DESC"#,
    )
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
                "name": r.name,
                "plan": r.plan,
                "config": r.config,
                "usage_ops_month": r.usage_ops_month,
                "usage_limit": r.usage_limit,
                "has_api_key": r.api_key_hash.is_some(),
                "created_at": r.created_at.to_rfc3339(),
                "updated_at": r.updated_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(json))
}

pub async fn get_tenant(
    State(state): State<Arc<AdminAppState>>,
    _claims: AdminClaims,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AdminError> {
    let row = sqlx::query_as::<_, TenantRow>(
        r#"SELECT id, name, plan, config, api_key_hash, usage_ops_month, usage_limit,
                  created_at, updated_at
           FROM tenants WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Internal error: {}", e);
        AdminError::Internal("Internal server error".into())
    })?
    .ok_or_else(|| AdminError::NotFound(format!("Tenant {} not found", id)))?;

    Ok(Json(serde_json::json!({
        "id": row.id,
        "name": row.name,
        "plan": row.plan,
        "config": row.config,
        "usage_ops_month": row.usage_ops_month,
        "usage_limit": row.usage_limit,
        "has_api_key": row.api_key_hash.is_some(),
        "created_at": row.created_at.to_rfc3339(),
        "updated_at": row.updated_at.to_rfc3339(),
    })))
}

#[derive(Deserialize)]
pub struct UpdateTenant {
    pub plan: Option<String>,
    pub config: Option<serde_json::Value>,
    pub usage_limit: Option<i32>,
    pub name: Option<String>,
}

pub async fn update_tenant(
    State(state): State<Arc<AdminAppState>>,
    claims: AdminClaims,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateTenant>,
) -> Result<Json<serde_json::Value>, AdminError> {
    if claims.role == "read_only" || claims.role == "marketing_admin" {
        return Err(AdminError::Forbidden);
    }

    let old = sqlx::query_as::<_, TenantRow>(
        "SELECT id, name, plan, config, api_key_hash, usage_ops_month, usage_limit, created_at, updated_at FROM tenants WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Internal error: {}", e);
        AdminError::Internal("Internal server error".into())
    })?
    .ok_or_else(|| AdminError::NotFound(format!("Tenant {} not found", id)))?;

    if let Some(plan) = &body.plan {
        sqlx::query("UPDATE tenants SET plan = $1, updated_at = NOW() WHERE id = $2")
            .bind(plan)
            .bind(id)
            .execute(&state.db_pool)
            .await
            .map_err(|e| {
                tracing::error!("Internal error: {}", e);
                AdminError::Internal("Internal server error".into())
            })?;
    }

    if let Some(config) = &body.config {
        sqlx::query("UPDATE tenants SET config = $1, updated_at = NOW() WHERE id = $2")
            .bind(config)
            .bind(id)
            .execute(&state.db_pool)
            .await
            .map_err(|e| {
                tracing::error!("Internal error: {}", e);
                AdminError::Internal("Internal server error".into())
            })?;
    }

    if let Some(limit) = body.usage_limit {
        sqlx::query("UPDATE tenants SET usage_limit = $1, updated_at = NOW() WHERE id = $2")
            .bind(limit)
            .bind(id)
            .execute(&state.db_pool)
            .await
            .map_err(|e| {
                tracing::error!("Internal error: {}", e);
                AdminError::Internal("Internal server error".into())
            })?;
    }

    if let Some(name) = &body.name {
        sqlx::query("UPDATE tenants SET name = $1, updated_at = NOW() WHERE id = $2")
            .bind(name)
            .bind(id)
            .execute(&state.db_pool)
            .await
            .map_err(|e| {
                tracing::error!("Internal error: {}", e);
                AdminError::Internal("Internal server error".into())
            })?;
    }

    // Audit
    let _ = sqlx::query(
        "INSERT INTO admin_audit_log (admin_id, admin_email, action, resource_type, resource_key, old_value, new_value) VALUES ($1, $2, 'update', 'tenant', $3, $4, $5)",
    )
    .bind(claims.sub)
    .bind(&claims.email)
    .bind(id.to_string())
    .bind(serde_json::json!({"plan": old.plan, "config": old.config, "usage_limit": old.usage_limit}))
    .bind(serde_json::json!({"plan": body.plan, "config": body.config, "usage_limit": body.usage_limit}))
    .execute(&state.db_pool)
    .await;

    Ok(Json(serde_json::json!({"id": id, "updated": true})))
}

#[derive(sqlx::FromRow)]
struct TenantRow {
    id: Uuid,
    name: Option<String>,
    plan: String,
    config: serde_json::Value,
    api_key_hash: Option<String>,
    usage_ops_month: i32,
    usage_limit: Option<i32>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}
