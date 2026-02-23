//! Team invitation endpoints for the tenant app.
//! Allows owners/admins to invite users via email-based tokens.

use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap};
use axum::response::{IntoResponse, Json, Response};
use chrono::{Duration, Utc};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::auth::{audit, normalize_email, AppClaims, AppError, AppUserRow};
use crate::TenantAppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateInviteRequest {
    /// Email address to invite.
    pub email: String,
    /// Role for the invitee: admin, developer, or viewer.
    pub role: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AcceptInviteRequest {
    /// Invite token received via email.
    pub token: String,
    pub full_name: String,
    /// Password (min 12 chars, must include uppercase, lowercase, digit, special).
    pub password: String,
}

#[derive(Debug, sqlx::FromRow)]
struct InviteRow {
    id: Uuid,
    tenant_id: Uuid,
    email: String,
    role: String,
    #[allow(dead_code)]
    invited_by: Uuid,
    status: String,
    expires_at: chrono::DateTime<chrono::Utc>,
    created_at: chrono::DateTime<chrono::Utc>,
}

fn random_hex(bytes: usize) -> String {
    let mut out = vec![0u8; bytes];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut out);
    hex::encode(out)
}

fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

/// Create a team invite (owner/admin only).
#[utoipa::path(
    post,
    path = "/app/invites",
    tag = "Invites",
    security(("bearer_auth" = [])),
    request_body = CreateInviteRequest,
    responses(
        (status = 200, description = "Invite created with token"),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Conflict — user exists or invite pending"),
    )
)]
pub async fn create_invite(
    claims: AppClaims,
    State(state): State<Arc<TenantAppState>>,
    Json(body): Json<CreateInviteRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !matches!(claims.role.as_str(), "owner" | "admin") {
        return Err(AppError::Forbidden);
    }

    let email = normalize_email(&body.email);
    if email.is_empty() || !email.contains('@') {
        return Err(AppError::BadRequest("Invalid email address".into()));
    }

    let role = body.role.unwrap_or_else(|| "developer".to_string());
    if !matches!(role.as_str(), "admin" | "developer" | "viewer") {
        return Err(AppError::BadRequest(
            "Invalid role. Use admin, developer, or viewer.".into(),
        ));
    }

    let existing_user =
        sqlx::query_scalar::<_, i32>("SELECT 1 FROM app_users WHERE tenant_id = $1 AND email = $2")
            .bind(claims.tenant_id)
            .bind(&email)
            .fetch_optional(&state.db_pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    if existing_user.is_some() {
        return Err(AppError::Conflict(
            "User is already a member of this workspace".into(),
        ));
    }

    let pending_invite = sqlx::query_scalar::<_, i32>(
        "SELECT 1 FROM team_invites WHERE tenant_id = $1 AND email = $2 AND status = 'pending' AND expires_at > NOW()",
    )
    .bind(claims.tenant_id)
    .bind(&email)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    if pending_invite.is_some() {
        return Err(AppError::Conflict(
            "A pending invite already exists for this email".into(),
        ));
    }

    let raw_token = random_hex(32);
    let token_hash = hash_token(&raw_token);
    let expires_at = Utc::now() + Duration::days(3);

    let invite_id = sqlx::query_scalar::<_, Uuid>(
        r#"INSERT INTO team_invites (tenant_id, email, role, invited_by, token_hash, status, expires_at)
           VALUES ($1, $2, $3, $4, $5, 'pending', $6)
           RETURNING id"#,
    )
    .bind(claims.tenant_id)
    .bind(&email)
    .bind(&role)
    .bind(claims.sub)
    .bind(&token_hash)
    .bind(expires_at)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    audit(
        &state,
        claims.tenant_id,
        Some(&claims),
        "create",
        "invite",
        Some(&invite_id.to_string()),
        None,
        Some(serde_json::json!({"email": email, "role": role})),
        None,
    )
    .await;

    Ok(Json(serde_json::json!({
        "id": invite_id,
        "email": email,
        "role": role,
        "token": raw_token,
        "expires_at": expires_at.to_rfc3339(),
    })))
}

/// List invites for the tenant (owner/admin only).
#[utoipa::path(
    get,
    path = "/app/invites",
    tag = "Invites",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Invites list"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn list_invites(
    claims: AppClaims,
    State(state): State<Arc<TenantAppState>>,
    Query(pagination): Query<crate::routes::app::PaginationParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !matches!(claims.role.as_str(), "owner" | "admin") {
        return Err(AppError::Forbidden);
    }

    let _ = sqlx::query(
        "UPDATE team_invites SET status = 'expired' WHERE tenant_id = $1 AND status = 'pending' AND expires_at <= NOW()",
    )
    .bind(claims.tenant_id)
    .execute(&state.db_pool)
    .await;

    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM team_invites WHERE tenant_id = $1",
    )
    .bind(claims.tenant_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let rows = sqlx::query_as::<_, InviteRow>(
        r#"SELECT id, tenant_id, email, role, invited_by, status, expires_at, created_at
           FROM team_invites
           WHERE tenant_id = $1
           ORDER BY created_at DESC
           LIMIT $2 OFFSET $3"#,
    )
    .bind(claims.tenant_id)
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let items: Vec<_> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "email": r.email,
                "role": r.role,
                "status": r.status,
                "expires_at": r.expires_at.to_rfc3339(),
                "created_at": r.created_at.to_rfc3339(),
            })
        })
        .collect();
    Ok(Json(serde_json::json!({
        "data": items,
        "total": total,
        "page": pagination.page(),
        "per_page": pagination.per_page(),
    })))
}

/// Revoke a pending invite (owner/admin only).
#[utoipa::path(
    delete,
    path = "/app/invites/{id}",
    tag = "Invites",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Invite ID")
    ),
    responses(
        (status = 200, description = "Invite revoked"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Invite not found or already used"),
    )
)]
pub async fn revoke_invite(
    claims: AppClaims,
    State(state): State<Arc<TenantAppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !matches!(claims.role.as_str(), "owner" | "admin") {
        return Err(AppError::Forbidden);
    }

    let updated = sqlx::query(
        "UPDATE team_invites SET status = 'revoked' WHERE id = $1 AND tenant_id = $2 AND status = 'pending'",
    )
    .bind(id)
    .bind(claims.tenant_id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if updated.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Invite not found or already used".into(),
        ));
    }

    audit(
        &state,
        claims.tenant_id,
        Some(&claims),
        "revoke",
        "invite",
        Some(&id.to_string()),
        None,
        Some(serde_json::json!({"status": "revoked"})),
        None,
    )
    .await;

    Ok(Json(serde_json::json!({"id": id, "revoked": true})))
}

/// Accept a team invite (public, rate-limited).
#[utoipa::path(
    post,
    path = "/app/auth/accept-invite",
    tag = "Invites",
    request_body = AcceptInviteRequest,
    responses(
        (status = 200, description = "Invite accepted, session created"),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Invite not found or expired"),
        (status = 409, description = "User already exists"),
        (status = 429, description = "Rate limited"),
    )
)]
pub async fn accept_invite(
    State(state): State<Arc<TenantAppState>>,
    headers: HeaderMap,
    Json(body): Json<AcceptInviteRequest>,
) -> Result<Response, AppError> {
    let client_ip = enterprise_common::client_ip::extract_client_ip(&headers);
    let rate_key = format!("app:invite:{}", client_ip);
    enterprise_common::rate_limit::enforce_rate_limit(&state.rate_limiter, &rate_key, "app:")
        .map_err(AppError::RateLimited)?;

    let full_name = body.full_name.trim();
    if full_name.len() < 2 {
        return Err(AppError::BadRequest("Full name is required".into()));
    }
    enterprise_common::password::validate_password(&body.password).map_err(AppError::BadRequest)?;

    let token_hash = hash_token(&body.token);

    let invite = sqlx::query_as::<_, InviteRow>(
        r#"SELECT id, tenant_id, email, role, invited_by, status, expires_at, created_at
           FROM team_invites
           WHERE token_hash = $1 AND status = 'pending' AND expires_at > NOW()"#,
    )
    .bind(&token_hash)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .ok_or_else(|| {
        enterprise_common::rate_limit::record_failure(&state.rate_limiter, &rate_key);
        AppError::NotFound("Invite not found, expired, or already used".into())
    })?;

    let existing =
        sqlx::query_scalar::<_, i32>("SELECT 1 FROM app_users WHERE tenant_id = $1 AND email = $2")
            .bind(invite.tenant_id)
            .bind(&invite.email)
            .fetch_optional(&state.db_pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    if existing.is_some() {
        return Err(AppError::Conflict(
            "User already exists in this workspace".into(),
        ));
    }

    let global_existing = sqlx::query_scalar::<_, i32>("SELECT 1 FROM app_users WHERE email = $1")
        .bind(&invite.email)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if global_existing.is_some() {
        return Err(AppError::Conflict(
            "Email already registered with another workspace".into(),
        ));
    }

    let password_hash =
        bcrypt::hash(&body.password, 12).map_err(|e| AppError::Internal(e.to_string()))?;

    let mut tx = state
        .db_pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let user_role = match invite.role.as_str() {
        "viewer" => "read_only",
        other => other,
    };

    #[derive(sqlx::FromRow)]
    #[allow(dead_code)]
    struct NewUser {
        id: Uuid,
        tenant_id: Uuid,
        email: String,
        password_hash: String,
        full_name: String,
        role: String,
        enabled: bool,
        failed_login_attempts: i32,
        locked_until: Option<chrono::DateTime<chrono::Utc>>,
        email_verified: bool,
        totp_enabled: bool,
    }

    let user = sqlx::query_as::<_, NewUser>(
        r#"INSERT INTO app_users (tenant_id, email, password_hash, full_name, role, enabled)
           VALUES ($1, $2, $3, $4, $5, true)
           RETURNING id, tenant_id, email, password_hash, full_name, role, enabled, failed_login_attempts, locked_until, email_verified, totp_enabled"#,
    )
    .bind(invite.tenant_id)
    .bind(&invite.email)
    .bind(&password_hash)
    .bind(full_name)
    .bind(user_role)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    sqlx::query("UPDATE team_invites SET status = 'accepted', accepted_at = NOW() WHERE id = $1")
        .bind(invite.id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    enterprise_common::rate_limit::clear_limit(&state.rate_limiter, &rate_key);

    // Build AppUserRow for issue_session_token
    let app_user = AppUserRow {
        id: user.id,
        tenant_id: user.tenant_id,
        email: user.email.clone(),
        password_hash: user.password_hash.clone(),
        full_name: user.full_name.clone(),
        role: user.role.clone(),
        enabled: user.enabled,
        failed_login_attempts: user.failed_login_attempts,
        locked_until: user.locked_until,
        email_verified: user.email_verified,
        totp_enabled: user.totp_enabled,
    };
    let user_agent_str = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let (token, expires) =
        crate::auth::issue_session_token(&state, &app_user, Some(&client_ip), user_agent_str.as_deref())
            .await?;

    audit(
        &state,
        user.tenant_id,
        None,
        "accept_invite",
        "user",
        Some(&user.id.to_string()),
        None,
        Some(serde_json::json!({"email": user.email, "role": user.role, "invite_id": invite.id})),
        Some(serde_json::json!({"ip": client_ip})),
    )
    .await;

    let tenant_name = sqlx::query_scalar::<_, String>("SELECT name FROM tenants WHERE id = $1")
        .bind(user.tenant_id)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .unwrap_or_default();

    let mut response = Json(serde_json::json!({
        "token": token,
        "expires_at": expires.to_rfc3339(),
        "user": {
            "id": user.id,
            "email": user.email,
            "full_name": user.full_name,
            "role": user.role,
            "tenant_id": user.tenant_id,
        },
        "tenant": {
            "name": tenant_name,
        },
    }))
    .into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, crate::auth::app_cookie(&token)?);
    Ok(response)
}
