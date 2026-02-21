//! Core tenant app endpoints: signup, login, logout, profile, API keys, users, audit.

use axum::{
    extract::{Path, State},
    http::{header, HeaderMap},
    response::{IntoResponse, Json, Response},
};
use chrono::{Duration, Utc};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::auth::{
    app_cookie, audit, clear_app_cookie, generate_api_key, hash_api_key, issue_session_token,
    normalize_email, AppClaims, AppError, AppUserRow,
};
use crate::TenantAppState;

// ── Request / response types ────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct SignupRequest {
    /// Company or workspace name.
    pub company_name: String,
    /// Full name of the owner.
    pub full_name: String,
    /// Email address.
    pub email: String,
    /// Password (min 12 chars, must include uppercase, lowercase, digit, special).
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateApiKeyRequest {
    /// Human-readable name for the key.
    pub name: Option<String>,
    /// Role: admin, developer, or read_only.
    pub role: Option<String>,
    /// Expiry in days from now (optional).
    pub expires_in_days: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAppUserRequest {
    pub full_name: String,
    pub email: String,
    pub password: String,
    /// Role: admin, developer, or read_only.
    pub role: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateAppUserRequest {
    pub full_name: Option<String>,
    pub role: Option<String>,
    pub enabled: Option<bool>,
}

// ── DB row types ────────────────────────────────────────────────────────

#[derive(Debug, sqlx::FromRow)]
struct TenantRow {
    id: Uuid,
    name: String,
    slug: String,
    plan: String,
    usage_ops_month: i32,
    usage_limit: Option<i32>,
}

#[derive(Debug, sqlx::FromRow)]
struct ApiKeyRow {
    id: Uuid,
    name: String,
    role: String,
    active: bool,
    last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct TenantAppUserRow {
    id: Uuid,
    email: String,
    full_name: String,
    role: String,
    enabled: bool,
    last_login_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct TenantAuditRow {
    id: Uuid,
    app_user_email: Option<String>,
    action: String,
    resource_type: String,
    resource_key: Option<String>,
    old_value: Option<serde_json::Value>,
    new_value: Option<serde_json::Value>,
    metadata: Option<serde_json::Value>,
    created_at: chrono::DateTime<chrono::Utc>,
}

// ── Helpers ─────────────────────────────────────────────────────────────

fn validate_password(password: &str) -> Result<(), AppError> {
    enterprise_common::password::validate_password(password)
        .map_err(|msg| AppError::BadRequest(msg))
}

fn slugify_company(name: &str) -> String {
    let mut slug = String::new();
    let mut prev_dash = false;
    for ch in name.trim().to_ascii_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            prev_dash = false;
        } else if !prev_dash {
            slug.push('-');
            prev_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

async fn unique_tenant_slug(pool: &sqlx::PgPool, name: &str) -> Result<String, AppError> {
    let base = {
        let raw = slugify_company(name);
        if raw.is_empty() {
            "company".to_string()
        } else {
            raw
        }
    };

    for i in 0..5000 {
        let candidate = if i == 0 {
            base.clone()
        } else {
            format!("{}-{}", base, i + 1)
        };
        let exists = sqlx::query_scalar::<_, i32>("SELECT 1 FROM tenants WHERE slug = $1")
            .bind(&candidate)
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .is_some();
        if !exists {
            return Ok(candidate);
        }
    }
    Err(AppError::Internal("Failed to allocate tenant slug".into()))
}

// ── Handlers ────────────────────────────────────────────────────────────

/// Register a new tenant workspace and owner account.
#[utoipa::path(
    post,
    path = "/app/auth/signup",
    tag = "Auth",
    request_body = SignupRequest,
    responses(
        (status = 200, description = "Signup successful"),
        (status = 400, description = "Bad request"),
        (status = 409, description = "Email already registered"),
        (status = 429, description = "Rate limited"),
    )
)]
pub async fn signup(
    State(state): State<Arc<TenantAppState>>,
    headers: HeaderMap,
    Json(body): Json<SignupRequest>,
) -> Result<Response, AppError> {
    let client_ip = enterprise_common::client_ip::extract_client_ip(&headers);
    let rate_key = format!("app:signup:{}", client_ip);
    enterprise_common::rate_limit::enforce_rate_limit(&state.rate_limiter, &rate_key, "app:")
        .map_err(AppError::RateLimited)?;

    let email = normalize_email(&body.email);
    let company_name = body.company_name.trim();
    let full_name = body.full_name.trim();

    if company_name.len() < 2 {
        return Err(AppError::BadRequest("Company name is required".into()));
    }
    if full_name.len() < 2 {
        return Err(AppError::BadRequest("Full name is required".into()));
    }
    validate_password(&body.password)?;

    let existing = sqlx::query_scalar::<_, i32>("SELECT 1 FROM app_users WHERE email = $1")
        .bind(&email)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if existing.is_some() {
        enterprise_common::rate_limit::record_failure(&state.rate_limiter, &rate_key);
        return Err(AppError::Conflict("Email already registered".into()));
    }

    let tenant_slug = unique_tenant_slug(&state.db_pool, company_name).await?;
    let raw_api_key = generate_api_key();
    let api_key_hash = hash_api_key(&raw_api_key);
    let password_hash =
        bcrypt::hash(&body.password, 12).map_err(|e| AppError::Internal(e.to_string()))?;

    let mut tx = state
        .db_pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let tenant: (Uuid,) = sqlx::query_as(
        "INSERT INTO tenants (name, slug, plan, api_key_hash) VALUES ($1, $2, 'free', $3) RETURNING id",
    )
    .bind(company_name)
    .bind(&tenant_slug)
    .bind(&api_key_hash)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    sqlx::query(
        "INSERT INTO tenant_api_keys (tenant_id, name, key_hash, role, active) VALUES ($1, 'primary', $2, 'admin', true)",
    )
    .bind(tenant.0)
    .bind(&api_key_hash)
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let user: AppUserRow = sqlx::query_as(
        r#"INSERT INTO app_users (tenant_id, email, password_hash, full_name, role, enabled)
           VALUES ($1, $2, $3, $4, 'owner', true)
           RETURNING id, tenant_id, email, password_hash, full_name, role, enabled"#,
    )
    .bind(tenant.0)
    .bind(&email)
    .bind(&password_hash)
    .bind(full_name)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let (token, expires) = issue_session_token(&state, &user).await?;
    enterprise_common::rate_limit::clear_limit(&state.rate_limiter, &rate_key);
    audit(
        &state,
        user.tenant_id,
        None,
        "signup",
        "session",
        Some(&user.id.to_string()),
        None,
        Some(serde_json::json!({"email": user.email, "role": user.role})),
        Some(serde_json::json!({"ip": client_ip})),
    )
    .await;

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
            "id": tenant.0,
            "name": company_name,
            "slug": tenant_slug,
            "plan": "free",
        },
        "initial_api_key": raw_api_key,
    }))
    .into_response();

    response
        .headers_mut()
        .insert(header::SET_COOKIE, app_cookie(&token)?);
    Ok(response)
}

/// Authenticate an existing user and return a session token.
#[utoipa::path(
    post,
    path = "/app/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful"),
        (status = 401, description = "Invalid credentials"),
        (status = 403, description = "Account disabled"),
        (status = 429, description = "Rate limited"),
    )
)]
pub async fn login(
    State(state): State<Arc<TenantAppState>>,
    headers: HeaderMap,
    Json(body): Json<LoginRequest>,
) -> Result<Response, AppError> {
    let client_ip = enterprise_common::client_ip::extract_client_ip(&headers);
    let rate_key = format!("app:login:{}", client_ip);
    enterprise_common::rate_limit::enforce_rate_limit(&state.rate_limiter, &rate_key, "app:")
        .map_err(AppError::RateLimited)?;

    let email = normalize_email(&body.email);
    let user = sqlx::query_as::<_, AppUserRow>(
        "SELECT id, tenant_id, email, password_hash, full_name, role, enabled FROM app_users WHERE email = $1",
    )
    .bind(&email)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    let user = match user {
        Some(u) => u,
        None => {
            enterprise_common::rate_limit::record_failure(&state.rate_limiter, &rate_key);
            return Err(AppError::Unauthorized);
        }
    };

    if !user.enabled {
        return Err(AppError::Forbidden);
    }
    let valid =
        bcrypt::verify(&body.password, &user.password_hash).map_err(|_| AppError::Unauthorized)?;
    if !valid {
        enterprise_common::rate_limit::record_failure(&state.rate_limiter, &rate_key);
        return Err(AppError::Unauthorized);
    }
    enterprise_common::rate_limit::clear_limit(&state.rate_limiter, &rate_key);

    let tenant = sqlx::query_as::<_, TenantRow>(
        "SELECT id, name, slug, plan, usage_ops_month, usage_limit FROM tenants WHERE id = $1",
    )
    .bind(user.tenant_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("Tenant not found".into()))?;

    let (token, expires) = issue_session_token(&state, &user).await?;
    audit(
        &state,
        user.tenant_id,
        None,
        "login",
        "session",
        Some(&user.id.to_string()),
        None,
        None,
        Some(serde_json::json!({"ip": client_ip, "email": user.email})),
    )
    .await;

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
            "id": tenant.id,
            "name": tenant.name,
            "slug": tenant.slug,
            "plan": tenant.plan,
            "usage_ops_month": tenant.usage_ops_month,
            "usage_limit": tenant.usage_limit,
        }
    }))
    .into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, app_cookie(&token)?);
    Ok(response)
}

/// Log out the current user (revokes all sessions).
#[utoipa::path(
    post,
    path = "/app/auth/logout",
    tag = "Auth",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Logged out"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn logout(
    State(state): State<Arc<TenantAppState>>,
    claims: AppClaims,
) -> Result<Response, AppError> {
    let _ = sqlx::query("DELETE FROM app_sessions WHERE app_user_id = $1")
        .bind(claims.sub)
        .execute(&state.db_pool)
        .await;
    audit(
        &state,
        claims.tenant_id,
        Some(&claims),
        "logout",
        "session",
        Some(&claims.sub.to_string()),
        None,
        None,
        None,
    )
    .await;

    let mut response = Json(serde_json::json!({"logged_out": true})).into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, clear_app_cookie());
    Ok(response)
}

/// Get the current user's profile and tenant information.
#[utoipa::path(
    get,
    path = "/app/auth/me",
    tag = "Auth",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "User profile"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn me(
    State(state): State<Arc<TenantAppState>>,
    claims: AppClaims,
) -> Result<Json<serde_json::Value>, AppError> {
    let profile = sqlx::query_as::<_, TenantAppUserRow>(
        "SELECT id, email, full_name, role, enabled, last_login_at, created_at, updated_at FROM app_users WHERE id = $1 AND tenant_id = $2",
    )
    .bind(claims.sub)
    .bind(claims.tenant_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .ok_or(AppError::Unauthorized)?;

    let tenant = sqlx::query_as::<_, TenantRow>(
        "SELECT id, name, slug, plan, usage_ops_month, usage_limit FROM tenants WHERE id = $1",
    )
    .bind(claims.tenant_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("Tenant not found".into()))?;

    Ok(Json(serde_json::json!({
        "user": {
            "id": claims.sub,
            "email": claims.email,
            "full_name": profile.full_name,
            "role": claims.role,
            "tenant_id": claims.tenant_id,
        },
        "tenant": {
            "id": tenant.id,
            "name": tenant.name,
            "slug": tenant.slug,
            "plan": tenant.plan,
            "usage_ops_month": tenant.usage_ops_month,
            "usage_limit": tenant.usage_limit,
        },
        "gateway_base_url": std::env::var("GATEWAY_PUBLIC_URL").unwrap_or_else(|_| "https://api.aiknol.com".to_string()),
    })))
}

/// Get the current tenant's information.
#[utoipa::path(
    get,
    path = "/app/tenant",
    tag = "Users",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Tenant details"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn tenant(
    State(state): State<Arc<TenantAppState>>,
    claims: AppClaims,
) -> Result<Json<serde_json::Value>, AppError> {
    let tenant = sqlx::query_as::<_, TenantRow>(
        "SELECT id, name, slug, plan, usage_ops_month, usage_limit FROM tenants WHERE id = $1",
    )
    .bind(claims.tenant_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("Tenant not found".into()))?;

    Ok(Json(serde_json::json!({
        "id": tenant.id,
        "name": tenant.name,
        "slug": tenant.slug,
        "plan": tenant.plan,
        "usage_ops_month": tenant.usage_ops_month,
        "usage_limit": tenant.usage_limit,
    })))
}

/// List all API keys for the current tenant.
#[utoipa::path(
    get,
    path = "/app/api-keys",
    tag = "API Keys",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "API keys list"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list_api_keys(
    State(state): State<Arc<TenantAppState>>,
    claims: AppClaims,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let rows = sqlx::query_as::<_, ApiKeyRow>(
        "SELECT id, name, role, active, last_used_at, expires_at, created_at FROM tenant_api_keys WHERE tenant_id = $1 ORDER BY created_at DESC",
    )
    .bind(claims.tenant_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let items = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "name": r.name,
                "role": r.role,
                "active": r.active,
                "last_used_at": r.last_used_at.map(|t| t.to_rfc3339()),
                "expires_at": r.expires_at.map(|t| t.to_rfc3339()),
                "created_at": r.created_at.to_rfc3339(),
            })
        })
        .collect();
    Ok(Json(items))
}

/// Create a new API key for the tenant.
#[utoipa::path(
    post,
    path = "/app/api-keys",
    tag = "API Keys",
    security(("bearer_auth" = [])),
    request_body = CreateApiKeyRequest,
    responses(
        (status = 200, description = "API key created (key shown once)"),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn create_api_key(
    State(state): State<Arc<TenantAppState>>,
    claims: AppClaims,
    Json(body): Json<CreateApiKeyRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !matches!(claims.role.as_str(), "owner" | "admin") {
        return Err(AppError::Forbidden);
    }

    let name = body.name.unwrap_or_else(|| "key".to_string());
    if name.trim().is_empty() {
        return Err(AppError::BadRequest("Key name cannot be empty".into()));
    }
    let role = body.role.unwrap_or_else(|| "developer".to_string());
    if !matches!(role.as_str(), "admin" | "developer" | "read_only") {
        return Err(AppError::BadRequest(
            "Invalid role. Use admin, developer, or read_only.".into(),
        ));
    }

    let expires_at = body.expires_in_days.map(|d| Utc::now() + Duration::days(d));
    let raw_key = generate_api_key();
    let key_hash = hash_api_key(&raw_key);

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO tenant_api_keys (tenant_id, name, key_hash, role, expires_at, active) VALUES ($1, $2, $3, $4, $5, true) RETURNING id",
    )
    .bind(claims.tenant_id)
    .bind(name.trim())
    .bind(key_hash)
    .bind(&role)
    .bind(expires_at)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    audit(
        &state,
        claims.tenant_id,
        Some(&claims),
        "create",
        "api_key",
        Some(&row.0.to_string()),
        None,
        Some(serde_json::json!({"name": name.trim(), "role": role, "expires_at": expires_at.map(|t| t.to_rfc3339())})),
        None,
    )
    .await;

    Ok(Json(serde_json::json!({
        "id": row.0,
        "name": name.trim(),
        "role": role,
        "expires_at": expires_at.map(|t| t.to_rfc3339()),
        "api_key": raw_key,
    })))
}

/// Revoke an existing API key.
#[utoipa::path(
    delete,
    path = "/app/api-keys/{id}",
    tag = "API Keys",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "API key ID")
    ),
    responses(
        (status = 200, description = "API key revoked"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn revoke_api_key(
    State(state): State<Arc<TenantAppState>>,
    claims: AppClaims,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !matches!(claims.role.as_str(), "owner" | "admin") {
        return Err(AppError::Forbidden);
    }
    let updated = sqlx::query(
        "UPDATE tenant_api_keys SET active = false, updated_at = NOW() WHERE id = $1 AND tenant_id = $2",
    )
    .bind(id)
    .bind(claims.tenant_id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    if updated.rows_affected() == 0 {
        return Err(AppError::NotFound("API key not found".into()));
    }
    audit(
        &state,
        claims.tenant_id,
        Some(&claims),
        "revoke",
        "api_key",
        Some(&id.to_string()),
        None,
        Some(serde_json::json!({"active": false})),
        None,
    )
    .await;
    Ok(Json(serde_json::json!({"id": id, "revoked": true})))
}

/// List all users in the tenant workspace.
#[utoipa::path(
    get,
    path = "/app/users",
    tag = "Users",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Users list"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn list_users(
    State(state): State<Arc<TenantAppState>>,
    claims: AppClaims,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    if !matches!(claims.role.as_str(), "owner" | "admin") {
        return Err(AppError::Forbidden);
    }
    let rows = sqlx::query_as::<_, TenantAppUserRow>(
        r#"SELECT id, email, full_name, role, enabled, last_login_at, created_at, updated_at
           FROM app_users
           WHERE tenant_id = $1
           ORDER BY created_at ASC"#,
    )
    .bind(claims.tenant_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(
        rows.iter()
            .map(|u| {
                serde_json::json!({
                    "id": u.id,
                    "email": u.email,
                    "full_name": u.full_name,
                    "role": u.role,
                    "enabled": u.enabled,
                    "last_login_at": u.last_login_at.map(|t| t.to_rfc3339()),
                    "created_at": u.created_at.to_rfc3339(),
                    "updated_at": u.updated_at.to_rfc3339(),
                })
            })
            .collect(),
    ))
}

/// Create a new user in the tenant workspace.
#[utoipa::path(
    post,
    path = "/app/users",
    tag = "Users",
    security(("bearer_auth" = [])),
    request_body = CreateAppUserRequest,
    responses(
        (status = 200, description = "User created"),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Conflict"),
    )
)]
pub async fn create_user(
    State(state): State<Arc<TenantAppState>>,
    claims: AppClaims,
    Json(body): Json<CreateAppUserRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !matches!(claims.role.as_str(), "owner" | "admin") {
        return Err(AppError::Forbidden);
    }
    validate_password(&body.password)?;

    let role = body.role.unwrap_or_else(|| "developer".to_string());
    if !matches!(role.as_str(), "admin" | "developer" | "read_only") {
        return Err(AppError::BadRequest(
            "Invalid role. Use admin, developer, or read_only.".into(),
        ));
    }

    let email = normalize_email(&body.email);
    let full_name = body.full_name.trim();
    if full_name.len() < 2 {
        return Err(AppError::BadRequest("Full name is required".into()));
    }

    let exists =
        sqlx::query_scalar::<_, i32>("SELECT 1 FROM app_users WHERE tenant_id = $1 AND email = $2")
            .bind(claims.tenant_id)
            .bind(&email)
            .fetch_optional(&state.db_pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    if exists.is_some() {
        return Err(AppError::Conflict(
            "User already exists in this tenant".into(),
        ));
    }

    let password_hash =
        bcrypt::hash(&body.password, 12).map_err(|e| AppError::Internal(e.to_string()))?;
    let created = sqlx::query_as::<_, TenantAppUserRow>(
        r#"INSERT INTO app_users (tenant_id, email, password_hash, full_name, role, enabled)
           VALUES ($1, $2, $3, $4, $5, true)
           RETURNING id, email, full_name, role, enabled, last_login_at, created_at, updated_at"#,
    )
    .bind(claims.tenant_id)
    .bind(&email)
    .bind(password_hash)
    .bind(full_name)
    .bind(&role)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    audit(
        &state,
        claims.tenant_id,
        Some(&claims),
        "create",
        "user",
        Some(&created.id.to_string()),
        None,
        Some(serde_json::json!({"email": created.email, "role": created.role, "full_name": created.full_name})),
        None,
    )
    .await;

    Ok(Json(serde_json::json!({
        "id": created.id,
        "email": created.email,
        "full_name": created.full_name,
        "role": created.role,
        "enabled": created.enabled,
    })))
}

/// Update a user's profile, role, or enabled status.
#[utoipa::path(
    put,
    path = "/app/users/{id}",
    tag = "Users",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateAppUserRequest,
    responses(
        (status = 200, description = "User updated"),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn update_user(
    State(state): State<Arc<TenantAppState>>,
    claims: AppClaims,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateAppUserRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !matches!(claims.role.as_str(), "owner" | "admin") {
        return Err(AppError::Forbidden);
    }

    let existing = sqlx::query_as::<_, TenantAppUserRow>(
        r#"SELECT id, email, full_name, role, enabled, last_login_at, created_at, updated_at
           FROM app_users WHERE id = $1 AND tenant_id = $2"#,
    )
    .bind(id)
    .bind(claims.tenant_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    if existing.id == claims.sub && body.enabled == Some(false) {
        return Err(AppError::BadRequest(
            "Cannot disable your own account".into(),
        ));
    }

    if let Some(name) = &body.full_name {
        if name.trim().len() < 2 {
            return Err(AppError::BadRequest("Full name is required".into()));
        }
    }
    if let Some(role) = &body.role {
        if !matches!(role.as_str(), "admin" | "developer" | "read_only") {
            return Err(AppError::BadRequest(
                "Invalid role. Use admin, developer, or read_only.".into(),
            ));
        }
    }

    let new_full_name = body
        .full_name
        .as_ref()
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|| existing.full_name.clone());
    let new_role = body.role.clone().unwrap_or_else(|| existing.role.clone());
    let new_enabled = body.enabled.unwrap_or(existing.enabled);

    sqlx::query(
        r#"UPDATE app_users
           SET full_name = $1, role = $2, enabled = $3, updated_at = NOW()
           WHERE id = $4 AND tenant_id = $5"#,
    )
    .bind(&new_full_name)
    .bind(&new_role)
    .bind(new_enabled)
    .bind(id)
    .bind(claims.tenant_id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if !new_enabled {
        let _ = sqlx::query("DELETE FROM app_sessions WHERE app_user_id = $1")
            .bind(id)
            .execute(&state.db_pool)
            .await;
    }

    audit(
        &state,
        claims.tenant_id,
        Some(&claims),
        "update",
        "user",
        Some(&id.to_string()),
        Some(serde_json::json!({"full_name": existing.full_name, "role": existing.role, "enabled": existing.enabled})),
        Some(serde_json::json!({"full_name": new_full_name, "role": new_role, "enabled": new_enabled})),
        None,
    )
    .await;

    Ok(Json(serde_json::json!({"id": id, "updated": true})))
}

/// List recent audit log entries for the tenant.
#[utoipa::path(
    get,
    path = "/app/audit",
    tag = "Usage",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Audit log entries"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn list_audit_logs(
    State(state): State<Arc<TenantAppState>>,
    claims: AppClaims,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    if !matches!(claims.role.as_str(), "owner" | "admin") {
        return Err(AppError::Forbidden);
    }

    let rows = sqlx::query_as::<_, TenantAuditRow>(
        r#"SELECT id, app_user_email, action, resource_type, resource_key, old_value, new_value, metadata, created_at
           FROM tenant_audit_log
           WHERE tenant_id = $1
           ORDER BY created_at DESC
           LIMIT 100"#,
    )
    .bind(claims.tenant_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(
        rows.iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "app_user_email": r.app_user_email,
                    "action": r.action,
                    "resource_type": r.resource_type,
                    "resource_key": r.resource_key,
                    "old_value": r.old_value,
                    "new_value": r.new_value,
                    "metadata": r.metadata,
                    "created_at": r.created_at.to_rfc3339(),
                })
            })
            .collect(),
    ))
}
