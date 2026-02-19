//! Admin user management — CRUD for admin accounts (super_admin only).

use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::{AdminClaims, AdminError};
use crate::AdminAppState;

pub async fn list_users(
    State(state): State<Arc<AdminAppState>>,
    claims: AdminClaims,
) -> Result<Json<Vec<serde_json::Value>>, AdminError> {
    if claims.role != "super_admin" {
        return Err(AdminError::Forbidden);
    }

    let rows = sqlx::query_as::<_, UserRow>(
        "SELECT id, email, role, enabled, last_login_at, created_at FROM admin_users ORDER BY created_at",
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AdminError::Internal(e.to_string()))?;

    let json: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "email": r.email,
                "role": r.role,
                "enabled": r.enabled,
                "last_login_at": r.last_login_at.map(|t| t.to_rfc3339()),
                "created_at": r.created_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(json))
}

#[derive(Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
    #[serde(default = "default_role")]
    pub role: String,
}

fn default_role() -> String {
    "read_only".into()
}

pub async fn create_user(
    State(state): State<Arc<AdminAppState>>,
    claims: AdminClaims,
    Json(body): Json<CreateUser>,
) -> Result<Json<serde_json::Value>, AdminError> {
    if claims.role != "super_admin" {
        return Err(AdminError::Forbidden);
    }

    if body.password.len() < 8 {
        return Err(AdminError::BadRequest(
            "Password must be at least 8 characters".into(),
        ));
    }

    let valid_roles = [
        "super_admin",
        "config_admin",
        "marketing_admin",
        "read_only",
    ];
    if !valid_roles.contains(&body.role.as_str()) {
        return Err(AdminError::BadRequest(format!(
            "Invalid role: {}. Must be one of: {:?}",
            body.role, valid_roles
        )));
    }

    let hash = bcrypt::hash(&body.password, 12)
        .map_err(|e| AdminError::Internal(format!("bcrypt: {}", e)))?;

    let id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO admin_users (email, password_hash, role) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(&body.email)
    .bind(&hash)
    .bind(&body.role)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AdminError::Internal(e.to_string()))?;

    let _ = sqlx::query(
        "INSERT INTO admin_audit_log (admin_id, admin_email, action, resource_type, resource_key, new_value) VALUES ($1, $2, 'create', 'user', $3, $4)",
    )
    .bind(claims.sub)
    .bind(&claims.email)
    .bind(id.to_string())
    .bind(serde_json::json!({"email": body.email, "role": body.role}))
    .execute(&state.db_pool)
    .await;

    Ok(Json(
        serde_json::json!({"id": id, "email": body.email, "role": body.role}),
    ))
}

#[derive(Deserialize)]
pub struct UpdateUser {
    pub role: Option<String>,
    pub enabled: Option<bool>,
}

pub async fn update_user(
    State(state): State<Arc<AdminAppState>>,
    claims: AdminClaims,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateUser>,
) -> Result<Json<serde_json::Value>, AdminError> {
    if claims.role != "super_admin" {
        return Err(AdminError::Forbidden);
    }

    // Prevent self-disable
    if id == claims.sub && body.enabled == Some(false) {
        return Err(AdminError::BadRequest(
            "Cannot disable your own account".into(),
        ));
    }

    if let Some(role) = &body.role {
        sqlx::query("UPDATE admin_users SET role = $1, updated_at = NOW() WHERE id = $2")
            .bind(role)
            .bind(id)
            .execute(&state.db_pool)
            .await
            .map_err(|e| AdminError::Internal(e.to_string()))?;
    }

    if let Some(enabled) = body.enabled {
        sqlx::query("UPDATE admin_users SET enabled = $1, updated_at = NOW() WHERE id = $2")
            .bind(enabled)
            .bind(id)
            .execute(&state.db_pool)
            .await
            .map_err(|e| AdminError::Internal(e.to_string()))?;
    }

    let _ = sqlx::query(
        "INSERT INTO admin_audit_log (admin_id, admin_email, action, resource_type, resource_key, new_value) VALUES ($1, $2, 'update', 'user', $3, $4)",
    )
    .bind(claims.sub)
    .bind(&claims.email)
    .bind(id.to_string())
    .bind(serde_json::json!({"role": body.role, "enabled": body.enabled}))
    .execute(&state.db_pool)
    .await;

    Ok(Json(serde_json::json!({"id": id, "updated": true})))
}

pub async fn delete_user(
    State(state): State<Arc<AdminAppState>>,
    claims: AdminClaims,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AdminError> {
    if claims.role != "super_admin" {
        return Err(AdminError::Forbidden);
    }

    if id == claims.sub {
        return Err(AdminError::BadRequest(
            "Cannot delete your own account".into(),
        ));
    }

    // Disable rather than hard delete
    sqlx::query("UPDATE admin_users SET enabled = false, updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| AdminError::Internal(e.to_string()))?;

    // Invalidate sessions
    sqlx::query("DELETE FROM admin_sessions WHERE admin_id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| AdminError::Internal(e.to_string()))?;

    let _ = sqlx::query(
        "INSERT INTO admin_audit_log (admin_id, admin_email, action, resource_type, resource_key) VALUES ($1, $2, 'delete', 'user', $3)",
    )
    .bind(claims.sub)
    .bind(&claims.email)
    .bind(id.to_string())
    .execute(&state.db_pool)
    .await;

    Ok(Json(serde_json::json!({"id": id, "disabled": true})))
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    role: String,
    enabled: bool,
    last_login_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
}
