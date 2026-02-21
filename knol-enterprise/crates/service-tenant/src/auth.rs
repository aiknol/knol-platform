//! Tenant app authentication: JWT claims, error types, and auth middleware.

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::TenantAppState;

// ── Claims ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct AppClaims {
    /// App user ID
    pub sub: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

// ── Error ───────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum AppError {
    Unauthorized,
    Forbidden,
    RateLimited(u64),
    Conflict(String),
    BadRequest(String),
    NotFound(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "Authentication required".to_string(),
            ),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                "Insufficient permissions".to_string(),
            ),
            AppError::RateLimited(secs) => (
                StatusCode::TOO_MANY_REQUESTS,
                format!("Too many attempts. Retry in {} seconds.", secs),
            ),
            AppError::Conflict(m) => (StatusCode::CONFLICT, m),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, m),
            AppError::Internal(m) => {
                tracing::error!("Tenant API internal error: {}", m);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };
        (status, Json(serde_json::json!({"error": msg}))).into_response()
    }
}

// ── Auth middleware ─────────────────────────────────────────────────────

pub async fn app_auth_middleware(
    State(state): State<Arc<TenantAppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .or_else(|| {
            request
                .headers()
                .get(header::COOKIE)
                .and_then(|v| v.to_str().ok())
                .and_then(|cookies| {
                    cookies.split(';').find_map(|c| {
                        let c = c.trim();
                        c.strip_prefix("app_token=").map(|t| t.to_string())
                    })
                })
        });

    let token = match token {
        Some(t) if !t.is_empty() => t,
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error":"Missing authentication"})),
            )
                .into_response();
        }
    };

    let claims = match decode::<AppClaims>(
        &token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => data.claims,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error":"Invalid token"})),
            )
                .into_response();
        }
    };

    if claims.exp < Utc::now().timestamp() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"Token expired"})),
        )
            .into_response();
    }

    let token_hash = hex::encode(Sha256::digest(token.as_bytes()));
    let valid_session = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM app_sessions WHERE app_user_id = $1 AND token_hash = $2 AND expires_at > NOW())",
    )
    .bind(claims.sub)
    .bind(token_hash)
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or(false);
    if !valid_session {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"Invalid or revoked session"})),
        )
            .into_response();
    }

    request.extensions_mut().insert(claims);
    next.run(request).await
}

// ── Extractor ───────────────────────────────────────────────────────────

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AppClaims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AppClaims>()
            .cloned()
            .ok_or(AppError::Unauthorized)
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────

#[derive(Debug, sqlx::FromRow)]
pub struct AppUserRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub full_name: String,
    pub role: String,
    pub enabled: bool,
}

pub async fn issue_session_token(
    state: &TenantAppState,
    user: &AppUserRow,
) -> Result<(String, chrono::DateTime<chrono::Utc>), AppError> {
    let now = Utc::now();
    let expires = now + Duration::hours(24);
    let claims = AppClaims {
        sub: user.id,
        tenant_id: user.tenant_id,
        email: user.email.clone(),
        role: user.role.clone(),
        exp: expires.timestamp(),
        iat: now.timestamp(),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
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

    let _ =
        sqlx::query("UPDATE app_users SET last_login_at = NOW(), updated_at = NOW() WHERE id = $1")
            .bind(user.id)
            .execute(&state.db_pool)
            .await;

    Ok((token, expires))
}

pub fn app_cookie(token: &str) -> Result<axum::http::HeaderValue, AppError> {
    let cookie = format!(
        "app_token={}; HttpOnly; SameSite=Lax; Path=/; Max-Age=86400{}",
        token,
        cookie_secure_suffix()
    );
    axum::http::HeaderValue::from_str(&cookie).map_err(|e| AppError::Internal(e.to_string()))
}

pub fn clear_app_cookie() -> axum::http::HeaderValue {
    axum::http::HeaderValue::from_static("app_token=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0")
}

pub fn cookie_secure_suffix() -> &'static str {
    if std::env::var("ADMIN_SECURE_COOKIES").unwrap_or_default() == "false" {
        ""
    } else {
        "; Secure"
    }
}

/// Convenience wrapper: insert tenant audit using AppClaims for actor info.
pub async fn audit(
    state: &TenantAppState,
    tenant_id: Uuid,
    actor: Option<&AppClaims>,
    action: &str,
    resource_type: &str,
    resource_key: Option<&str>,
    old_value: Option<serde_json::Value>,
    new_value: Option<serde_json::Value>,
    metadata: Option<serde_json::Value>,
) {
    let actor_id = actor.map(|c| c.sub);
    let actor_email = actor.map(|c| c.email.clone());
    enterprise_common::audit::insert_tenant_audit(
        &state.db_pool,
        tenant_id,
        actor_id,
        actor_email,
        action,
        resource_type,
        resource_key,
        old_value,
        new_value,
        metadata,
    )
    .await;
}

pub fn hash_api_key(key: &str) -> String {
    const API_KEY_SALT: &[u8] = b"knol-memory-platform-v1";
    let mut hasher = sha2::Sha256::new();
    hasher.update(API_KEY_SALT);
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn random_hex(bytes: usize) -> String {
    let mut out = vec![0u8; bytes];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut out);
    hex::encode(out)
}

pub fn generate_api_key() -> String {
    format!("knol_live_{}", random_hex(24))
}

pub fn normalize_email(email: &str) -> String {
    email.trim().to_ascii_lowercase()
}
