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
    /// Unique token ID to prevent hash collisions when multiple tokens are
    /// issued within the same second for the same user.
    #[serde(default = "Uuid::new_v4")]
    pub jti: Uuid,
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
        match self {
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Authentication required"})),
            )
                .into_response(),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "Insufficient permissions"})),
            )
                .into_response(),
            AppError::RateLimited(secs) => {
                let msg = format!("Too many attempts. Retry in {} seconds.", secs);
                let mut resp = (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(serde_json::json!({"error": msg})),
                )
                    .into_response();
                if let Ok(val) = axum::http::HeaderValue::from_str(&secs.to_string()) {
                    resp.headers_mut().insert("retry-after", val);
                }
                resp
            }
            AppError::Conflict(m) => {
                (StatusCode::CONFLICT, Json(serde_json::json!({"error": m}))).into_response()
            }
            AppError::BadRequest(m) => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": m})),
            )
                .into_response(),
            AppError::NotFound(m) => {
                (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": m}))).into_response()
            }
            AppError::Internal(m) => {
                tracing::error!("Tenant API internal error: {}", m);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Internal server error"})),
                )
                    .into_response()
            }
        }
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
        &Validation::new(jsonwebtoken::Algorithm::HS256),
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

    // Fetch session with last_activity_at for idle timeout check
    let session_row = sqlx::query_as::<_, (bool, Option<chrono::DateTime<chrono::Utc>>)>(
        "SELECT true, last_activity_at FROM app_sessions WHERE app_user_id = $1 AND token_hash = $2 AND expires_at > NOW()",
    )
    .bind(claims.sub)
    .bind(&token_hash)
    .fetch_optional(&state.db_pool)
    .await
    .unwrap_or(None);

    let last_activity = match session_row {
        Some((_, la)) => la,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error":"Invalid or revoked session"})),
            )
                .into_response();
        }
    };

    // Idle session timeout check
    if state.idle_timeout_mins > 0 {
        if let Some(la) = last_activity {
            let idle_duration = Utc::now() - la;
            if idle_duration > chrono::Duration::minutes(state.idle_timeout_mins) {
                // Delete the expired session
                let _ = sqlx::query(
                    "DELETE FROM app_sessions WHERE app_user_id = $1 AND token_hash = $2",
                )
                .bind(claims.sub)
                .bind(&token_hash)
                .execute(&state.db_pool)
                .await;
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({"error":"Session expired due to inactivity"})),
                )
                    .into_response();
            }
        }
    }

    // Throttled activity update (at most once per minute to reduce DB writes)
    let _ = sqlx::query(
        "UPDATE app_sessions SET last_activity_at = NOW() WHERE app_user_id = $1 AND token_hash = $2 AND last_activity_at < NOW() - INTERVAL '1 minute'",
    )
    .bind(claims.sub)
    .bind(&token_hash)
    .execute(&state.db_pool)
    .await;

    // CSRF check for mutating requests
    let method = request.method().clone();
    if matches!(
        method,
        axum::http::Method::POST | axum::http::Method::PUT | axum::http::Method::DELETE
    ) {
        if !enterprise_common::csrf::verify_csrf(request.headers()) {
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error":"Invalid or missing CSRF token"})),
            )
                .into_response();
        }
    }

    // Per-tenant API rate limiting
    let plan = sqlx::query_scalar::<_, String>("SELECT plan FROM tenants WHERE id = $1")
        .bind(claims.tenant_id)
        .fetch_optional(&state.db_pool)
        .await
        .unwrap_or(None)
        .unwrap_or_else(|| "free".to_string());

    let limit = match plan.as_str() {
        "growth" => 2000,
        "builder" => 500,
        _ => 100,
    };

    if let Err(retry_after) = state.api_rate_limiter.check(claims.tenant_id, limit) {
        let mut resp = (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({"error": format!("API rate limit exceeded. Retry in {} seconds.", retry_after)})),
        )
            .into_response();
        if let Ok(val) = axum::http::HeaderValue::from_str(&retry_after.to_string()) {
            resp.headers_mut().insert("retry-after", val);
        }
        return resp;
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
    pub failed_login_attempts: i32,
    pub locked_until: Option<chrono::DateTime<chrono::Utc>>,
    pub email_verified: bool,
    pub totp_enabled: bool,
}

pub async fn issue_session_token(
    state: &TenantAppState,
    user: &AppUserRow,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
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
        jti: Uuid::new_v4(),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let token_hash = hex::encode(Sha256::digest(token.as_bytes()));
    sqlx::query(
        "INSERT INTO app_sessions (app_user_id, token_hash, expires_at, ip_address, user_agent) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user.id)
    .bind(&token_hash)
    .bind(expires)
    .bind(ip_address)
    .bind(user_agent)
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

/// Append a CSRF cookie to the response (non-HttpOnly so JS can read it).
pub fn append_csrf_cookie(response: &mut Response) {
    let csrf_token = enterprise_common::csrf::generate_csrf_token();
    let secure = std::env::var("ADMIN_SECURE_COOKIES").unwrap_or_default() != "false";
    let cookie_val = enterprise_common::csrf::csrf_cookie(&csrf_token, secure);
    if let Ok(val) = axum::http::HeaderValue::from_str(&cookie_val) {
        response.headers_mut().append(header::SET_COOKIE, val);
    }
}

pub fn cookie_secure_suffix() -> &'static str {
    if std::env::var("ADMIN_SECURE_COOKIES").unwrap_or_default() == "false" {
        ""
    } else {
        "; Secure"
    }
}

/// Convenience wrapper: insert tenant audit using AppClaims for actor info.
#[allow(clippy::too_many_arguments)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_api_key_deterministic() {
        let key = "knol_live_abc123";
        assert_eq!(hash_api_key(key), hash_api_key(key));
    }

    #[test]
    fn test_hash_api_key_different_inputs() {
        assert_ne!(hash_api_key("key_a"), hash_api_key("key_b"));
    }

    #[test]
    fn test_hash_api_key_uses_salt() {
        // Plain SHA256 without salt should differ from our salted hash
        let key = "test_key";
        let plain_sha256 = hex::encode(sha2::Sha256::digest(key.as_bytes()));
        assert_ne!(hash_api_key(key), plain_sha256);
    }

    #[test]
    fn test_random_hex_length() {
        let hex = random_hex(24);
        assert_eq!(hex.len(), 48); // 24 bytes = 48 hex chars
    }

    #[test]
    fn test_random_hex_uniqueness() {
        let a = random_hex(24);
        let b = random_hex(24);
        assert_ne!(a, b);
    }

    #[test]
    fn test_generate_api_key_format() {
        let key = generate_api_key();
        assert!(key.starts_with("knol_live_"));
        assert_eq!(key.len(), 10 + 48); // "knol_live_" + 48 hex chars
    }

    #[test]
    fn test_normalize_email_lowercase() {
        assert_eq!(normalize_email("User@DOMAIN.COM"), "user@domain.com");
    }

    #[test]
    fn test_normalize_email_trim() {
        assert_eq!(normalize_email("  foo@bar.com  "), "foo@bar.com");
    }

    #[test]
    fn test_app_cookie_format() {
        // Set env var for test to get predictable output
        std::env::set_var("ADMIN_SECURE_COOKIES", "false");
        let cookie = app_cookie("test_token_123").unwrap();
        let cookie_str = cookie.to_str().unwrap();
        assert!(cookie_str.contains("app_token=test_token_123"));
        assert!(cookie_str.contains("HttpOnly"));
        assert!(cookie_str.contains("SameSite=Lax"));
        assert!(cookie_str.contains("Max-Age=86400"));
    }

    #[test]
    fn test_clear_app_cookie() {
        let cookie = clear_app_cookie();
        let cookie_str = cookie.to_str().unwrap();
        assert!(cookie_str.contains("app_token="));
        assert!(cookie_str.contains("Max-Age=0"));
        assert!(cookie_str.contains("HttpOnly"));
    }
}
