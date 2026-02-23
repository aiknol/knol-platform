//! Tenant settings and user profile management endpoints.

use axum::extract::State;
use axum::http::{header, HeaderMap};
use axum::response::{IntoResponse, Json, Response};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;

use crate::auth::{
    app_cookie, audit, clear_app_cookie, issue_session_token, AppClaims, AppError, AppUserRow,
};
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
    headers: HeaderMap,
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

    // Issue fresh session using shared helper
    let user = sqlx::query_as::<_, AppUserRow>(
        "SELECT id, tenant_id, email, password_hash, full_name, role, enabled, failed_login_attempts, locked_until, email_verified, totp_enabled FROM app_users WHERE id = $1",
    )
    .bind(claims.sub)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let client_ip = enterprise_common::client_ip::extract_client_ip(&headers);
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let (token, expires) =
        issue_session_token(&state, &user, Some(&client_ip), user_agent.as_deref()).await?;

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

    let mut response = Json(serde_json::json!({
        "password_changed": true,
        "token": token,
        "expires_at": expires.to_rfc3339(),
    }))
    .into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, app_cookie(&token)?);
    Ok(response)
}

// ── GDPR Data Export ────────────────────────────────────────────────────

/// Export all data associated with the current user.
pub async fn data_export(
    claims: AppClaims,
    State(state): State<Arc<TenantAppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    // User profile
    let user = sqlx::query_as::<_, (String, String, String, bool, bool)>(
        "SELECT email, full_name, role, enabled, email_verified FROM app_users WHERE id = $1",
    )
    .bind(claims.sub)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    // Sessions
    let sessions: Vec<serde_json::Value> = sqlx::query_as::<_, (uuid::Uuid, Option<String>, Option<String>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
        "SELECT id, ip_address, user_agent, created_at, expires_at FROM app_sessions WHERE app_user_id = $1 ORDER BY created_at DESC",
    )
    .bind(claims.sub)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .into_iter()
    .map(|(id, ip, ua, created, expires)| serde_json::json!({
        "id": id,
        "ip_address": ip,
        "user_agent": ua,
        "created_at": created.to_rfc3339(),
        "expires_at": expires.to_rfc3339(),
    }))
    .collect();

    // Audit log entries where this user is the actor
    let audit_entries: Vec<serde_json::Value> = sqlx::query_as::<_, (String, String, Option<String>, chrono::DateTime<chrono::Utc>)>(
        "SELECT action, resource_type, resource_key, created_at FROM tenant_audit_log WHERE app_user_id = $1 ORDER BY created_at DESC LIMIT 1000",
    )
    .bind(claims.sub)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .into_iter()
    .map(|(action, rt, rk, created)| serde_json::json!({
        "action": action,
        "resource_type": rt,
        "resource_key": rk,
        "created_at": created.to_rfc3339(),
    }))
    .collect();

    Ok(Json(serde_json::json!({
        "export_date": chrono::Utc::now().to_rfc3339(),
        "user": {
            "id": claims.sub,
            "email": user.0,
            "full_name": user.1,
            "role": user.2,
            "enabled": user.3,
            "email_verified": user.4,
            "tenant_id": claims.tenant_id,
        },
        "sessions": sessions,
        "audit_log": audit_entries,
    })))
}

// ── GDPR Account Deletion ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct DeleteAccountRequest {
    pub password: String,
}

/// Schedule account deletion (30-day grace period).
pub async fn delete_account(
    claims: AppClaims,
    State(state): State<Arc<TenantAppState>>,
    Json(body): Json<DeleteAccountRequest>,
) -> Result<Response, AppError> {
    // Verify password
    let hash = sqlx::query_scalar::<_, String>(
        "SELECT password_hash FROM app_users WHERE id = $1 AND tenant_id = $2",
    )
    .bind(claims.sub)
    .bind(claims.tenant_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .ok_or(AppError::Unauthorized)?;

    let valid = bcrypt::verify(&body.password, &hash).map_err(|_| AppError::Unauthorized)?;
    if !valid {
        return Err(AppError::BadRequest("Password is incorrect".into()));
    }

    // Prevent owner from deleting if they're the only owner
    if claims.role == "owner" {
        let owner_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM app_users WHERE tenant_id = $1 AND role = 'owner' AND enabled = true",
        )
        .bind(claims.tenant_id)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        if owner_count <= 1 {
            return Err(AppError::BadRequest(
                "Cannot delete the only owner. Transfer ownership first.".into(),
            ));
        }
    }

    // Schedule deletion for 30 days from now
    let deletion_date = chrono::Utc::now() + chrono::Duration::days(30);
    sqlx::query(
        "UPDATE app_users SET deletion_requested_at = NOW(), deletion_scheduled_for = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(deletion_date)
    .bind(claims.sub)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    // Invalidate all sessions
    let _ = sqlx::query("DELETE FROM app_sessions WHERE app_user_id = $1")
        .bind(claims.sub)
        .execute(&state.db_pool)
        .await;

    audit(
        &state,
        claims.tenant_id,
        Some(&claims),
        "delete_account_requested",
        "user",
        Some(&claims.sub.to_string()),
        None,
        Some(serde_json::json!({"deletion_scheduled_for": deletion_date.to_rfc3339()})),
        None,
    )
    .await;

    let mut response = Json(serde_json::json!({
        "scheduled": true,
        "deletion_date": deletion_date.to_rfc3339(),
    }))
    .into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, clear_app_cookie());
    Ok(response)
}
