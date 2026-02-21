//! Tenant settings and user profile management endpoints.

use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Json, Response};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::auth::{audit, AppClaims, AppError};
use crate::TenantAppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTenantRequest {
    /// New workspace name (min 2 chars).
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProfileRequest {
    /// New full name (min 2 chars).
    pub full_name: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ChangePasswordRequest {
    /// Current password for verification.
    pub current_password: String,
    /// New password (min 12 chars, must include uppercase, lowercase, digit, special).
    pub new_password: String,
}

/// Update workspace name (owner/admin only).
#[utoipa::path(
    put,
    path = "/app/settings/tenant",
    tag = "Settings",
    security(("bearer_auth" = [])),
    request_body = UpdateTenantRequest,
    responses(
        (status = 200, description = "Tenant settings updated"),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn update_tenant_settings(
    claims: AppClaims,
    State(state): State<Arc<TenantAppState>>,
    Json(body): Json<UpdateTenantRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !matches!(claims.role.as_str(), "owner" | "admin") {
        return Err(AppError::Forbidden);
    }

    let name = body
        .name
        .as_deref()
        .map(|n| n.trim())
        .filter(|n| !n.is_empty());

    let name = match name {
        Some(n) if n.len() >= 2 => n,
        Some(_) => {
            return Err(AppError::BadRequest(
                "Workspace name must be at least 2 characters".into(),
            ))
        }
        None => return Err(AppError::BadRequest("No fields to update".into())),
    };

    let old_name = sqlx::query_scalar::<_, String>("SELECT name FROM tenants WHERE id = $1")
        .bind(claims.tenant_id)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Tenant not found".into()))?;

    sqlx::query("UPDATE tenants SET name = $1, updated_at = NOW() WHERE id = $2")
        .bind(name)
        .bind(claims.tenant_id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    audit(
        &state,
        claims.tenant_id,
        Some(&claims),
        "update",
        "tenant",
        Some(&claims.tenant_id.to_string()),
        Some(serde_json::json!({"name": old_name})),
        Some(serde_json::json!({"name": name})),
        None,
    )
    .await;

    Ok(Json(serde_json::json!({"updated": true, "name": name})))
}

/// Update own profile (name).
#[utoipa::path(
    put,
    path = "/app/settings/profile",
    tag = "Settings",
    security(("bearer_auth" = [])),
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated"),
        (status = 400, description = "Bad request"),
    )
)]
pub async fn update_profile(
    claims: AppClaims,
    State(state): State<Arc<TenantAppState>>,
    Json(body): Json<UpdateProfileRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let full_name = body
        .full_name
        .as_deref()
        .map(|n| n.trim())
        .filter(|n| !n.is_empty());

    let full_name = match full_name {
        Some(n) if n.len() >= 2 => n,
        Some(_) => {
            return Err(AppError::BadRequest(
                "Full name must be at least 2 characters".into(),
            ))
        }
        None => return Err(AppError::BadRequest("No fields to update".into())),
    };

    sqlx::query(
        "UPDATE app_users SET full_name = $1, updated_at = NOW() WHERE id = $2 AND tenant_id = $3",
    )
    .bind(full_name)
    .bind(claims.sub)
    .bind(claims.tenant_id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    audit(
        &state,
        claims.tenant_id,
        Some(&claims),
        "update",
        "profile",
        Some(&claims.sub.to_string()),
        None,
        Some(serde_json::json!({"full_name": full_name})),
        None,
    )
    .await;

    Ok(Json(
        serde_json::json!({"updated": true, "full_name": full_name}),
    ))
}

/// Change own password. Invalidates all sessions and issues a fresh token.
#[utoipa::path(
    post,
    path = "/app/settings/change-password",
    tag = "Settings",
    security(("bearer_auth" = [])),
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed, new token issued"),
        (status = 400, description = "Bad request — wrong current password or weak new password"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn change_password(
    claims: AppClaims,
    State(state): State<Arc<TenantAppState>>,
    Json(body): Json<ChangePasswordRequest>,
) -> Result<Response, AppError> {
    let current_hash = sqlx::query_scalar::<_, String>(
        "SELECT password_hash FROM app_users WHERE id = $1 AND tenant_id = $2",
    )
    .bind(claims.sub)
    .bind(claims.tenant_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .ok_or(AppError::Unauthorized)?;

    let valid = bcrypt::verify(&body.current_password, &current_hash)
        .map_err(|_| AppError::Unauthorized)?;
    if !valid {
        return Err(AppError::BadRequest("Current password is incorrect".into()));
    }

    enterprise_common::password::validate_password(&body.new_password)
        .map_err(AppError::BadRequest)?;

    let new_hash =
        bcrypt::hash(&body.new_password, 12).map_err(|e| AppError::Internal(e.to_string()))?;

    sqlx::query(
        "UPDATE app_users SET password_hash = $1, updated_at = NOW() WHERE id = $2 AND tenant_id = $3",
    )
    .bind(&new_hash)
    .bind(claims.sub)
    .bind(claims.tenant_id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    // Invalidate all sessions
    let _ = sqlx::query("DELETE FROM app_sessions WHERE app_user_id = $1")
        .bind(claims.sub)
        .execute(&state.db_pool)
        .await;

    // Issue fresh session
    #[derive(sqlx::FromRow)]
    #[allow(dead_code)]
    struct UserRow {
        id: uuid::Uuid,
        tenant_id: uuid::Uuid,
        email: String,
        password_hash: String,
        full_name: String,
        role: String,
        enabled: bool,
    }
    let user = sqlx::query_as::<_, UserRow>(
        "SELECT id, tenant_id, email, password_hash, full_name, role, enabled FROM app_users WHERE id = $1",
    )
    .bind(claims.sub)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let now = chrono::Utc::now();
    let expires = now + chrono::Duration::hours(24);
    let new_claims = AppClaims {
        sub: user.id,
        tenant_id: user.tenant_id,
        email: user.email.clone(),
        role: user.role.clone(),
        exp: expires.timestamp(),
        iat: now.timestamp(),
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &new_claims,
        &jsonwebtoken::EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let token_hash = hex::encode(Sha256::digest(token.as_bytes()));
    sqlx::query(
        "INSERT INTO app_sessions (app_user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
    )
    .bind(user.id)
    .bind(&token_hash)
    .bind(expires)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    audit(
        &state,
        claims.tenant_id,
        Some(&claims),
        "change_password",
        "user",
        Some(&claims.sub.to_string()),
        None,
        None,
        None,
    )
    .await;

    let secure = crate::auth::cookie_secure_suffix();
    let cookie = format!(
        "app_token={}; HttpOnly; SameSite=Lax; Path=/; Max-Age=86400{}",
        token, secure
    );

    let mut response = Json(serde_json::json!({
        "password_changed": true,
        "token": token,
        "expires_at": expires.to_rfc3339(),
    }))
    .into_response();
    if let Ok(cookie_val) = axum::http::HeaderValue::from_str(&cookie) {
        response
            .headers_mut()
            .insert(header::SET_COOKIE, cookie_val);
    }
    Ok(response)
}
