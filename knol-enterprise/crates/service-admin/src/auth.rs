//! Admin authentication — JWT login, middleware, bcrypt password hashing.

use axum::{
    extract::{Request, State},
    http::{header, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::AdminAppState;

// ── JWT Claims ──────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminClaims {
    pub sub: Uuid, // admin_id
    pub email: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

// ── Login ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub admin: AdminInfo,
    pub expires_at: String,
}

#[derive(Debug, Serialize)]
pub struct AdminInfo {
    pub id: Uuid,
    pub email: String,
    pub role: String,
}

/// Maximum login attempts per IP within the rate limit window.
const LOGIN_MAX_ATTEMPTS: u32 = 5;
/// Rate limit window duration (15 minutes).
const LOGIN_WINDOW_SECS: u64 = 900;

pub async fn login(
    State(state): State<Arc<AdminAppState>>,
    headers: axum::http::HeaderMap,
    Json(body): Json<LoginRequest>,
) -> Result<Response, AdminError> {
    // SECURITY: Extract client IP for rate limiting and audit logging.
    let client_ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    // SECURITY: Per-IP login rate limiting to prevent brute-force attacks.
    {
        let mut limiter = state
            .login_rate_limiter
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let now = std::time::Instant::now();

        // Clean up expired entries (older than window)
        limiter.retain(|_, (_, first_attempt)| {
            now.duration_since(*first_attempt).as_secs() < LOGIN_WINDOW_SECS
        });

        if let Some((count, first_attempt)) = limiter.get(&client_ip) {
            if now.duration_since(*first_attempt).as_secs() < LOGIN_WINDOW_SECS
                && *count >= LOGIN_MAX_ATTEMPTS
            {
                let remaining_secs =
                    LOGIN_WINDOW_SECS - now.duration_since(*first_attempt).as_secs();
                warn!(ip = %client_ip, "Login rate limited — {} attempts exhausted", LOGIN_MAX_ATTEMPTS);
                return Err(AdminError::RateLimited(remaining_secs));
            }
        }
    }

    // Find admin user
    let normalized_email = body.email.trim().to_lowercase();
    let row = sqlx::query_as::<_, AdminUserRow>(
        "SELECT id, email, password_hash, role, enabled FROM admin_users WHERE email = $1",
    )
    .bind(&normalized_email)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error during login: {}", e);
        AdminError::Internal("Authentication service unavailable".into())
    })?
    .ok_or(AdminError::InvalidCredentials)?;

    if !row.enabled {
        return Err(AdminError::AccountDisabled);
    }

    // Verify password
    let valid = bcrypt::verify(&body.password, &row.password_hash)
        .map_err(|_| AdminError::InvalidCredentials)?;
    if !valid {
        // SECURITY: Log failed attempt with IP and email, increment rate limiter.
        warn!(ip = %client_ip, email = %normalized_email, "Failed login attempt");
        {
            let mut limiter = state
                .login_rate_limiter
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let now = std::time::Instant::now();
            let entry = limiter.entry(client_ip.clone()).or_insert((0, now));
            if now.duration_since(entry.1).as_secs() >= LOGIN_WINDOW_SECS {
                *entry = (1, now); // Reset window
            } else {
                entry.0 += 1;
            }
        }
        return Err(AdminError::InvalidCredentials);
    }

    // SECURITY: Clear rate limit on successful login.
    {
        let mut limiter = state
            .login_rate_limiter
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        limiter.remove(&client_ip);
    }

    // Generate JWT
    let now = Utc::now();
    let expires = now + Duration::hours(24);
    let claims = AdminClaims {
        sub: row.id,
        email: row.email.clone(),
        role: row.role.clone(),
        exp: expires.timestamp(),
        iat: now.timestamp(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|e| {
        tracing::error!("JWT encode error: {}", e);
        AdminError::Internal("Token generation failed".into())
    })?;

    // Store session
    let token_hash = hex::encode(Sha256::digest(token.as_bytes()));
    let _ = sqlx::query(
        "INSERT INTO admin_sessions (admin_id, token_hash, expires_at) VALUES ($1, $2, $3)",
    )
    .bind(row.id)
    .bind(&token_hash)
    .bind(expires)
    .execute(&state.db_pool)
    .await;

    // Clean up expired sessions for this admin (LOW #11 fix)
    let _ = sqlx::query("DELETE FROM admin_sessions WHERE admin_id = $1 AND expires_at < NOW()")
        .bind(row.id)
        .execute(&state.db_pool)
        .await;

    // Update last login
    let _ = sqlx::query("UPDATE admin_users SET last_login_at = NOW() WHERE id = $1")
        .bind(row.id)
        .execute(&state.db_pool)
        .await;

    // Audit log
    let _ = sqlx::query(
        "INSERT INTO admin_audit_log (admin_id, admin_email, action, resource_type) VALUES ($1, $2, 'login', 'session')",
    )
    .bind(row.id)
    .bind(&row.email)
    .execute(&state.db_pool)
    .await;

    info!(ip = %client_ip, "Admin login: {}", row.email);

    // SECURITY: Set JWT as HttpOnly, Secure, SameSite cookie so it cannot
    // be stolen by XSS attacks.  The JSON body still contains the token for
    // backward compatibility, but new frontend code should rely on the cookie.
    let is_secure = std::env::var("ADMIN_SECURE_COOKIES").unwrap_or_default() != "false";
    let cookie_value = format!(
        "admin_token={}; HttpOnly; SameSite=Strict; Path=/admin; Max-Age=86400{}",
        token,
        if is_secure { "; Secure" } else { "" },
    );

    let mut response = axum::response::IntoResponse::into_response(Json(LoginResponse {
        token: token.clone(),
        admin: AdminInfo {
            id: row.id,
            email: row.email,
            role: row.role,
        },
        expires_at: expires.to_rfc3339(),
    }));

    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie_value).unwrap_or_else(|_| HeaderValue::from_static("")),
    );

    Ok(response)
}

pub async fn logout(
    State(state): State<Arc<AdminAppState>>,
    claims: AdminClaims,
) -> Result<Response, AdminError> {
    // Invalidate all sessions for this admin
    let _ = sqlx::query("DELETE FROM admin_sessions WHERE admin_id = $1")
        .bind(claims.sub)
        .execute(&state.db_pool)
        .await;

    let _ = sqlx::query(
        "INSERT INTO admin_audit_log (admin_id, admin_email, action, resource_type) VALUES ($1, $2, 'logout', 'session')",
    )
    .bind(claims.sub)
    .bind(&claims.email)
    .execute(&state.db_pool)
    .await;

    // SECURITY: Clear the HttpOnly cookie on logout.
    let mut response = Json(serde_json::json!({"logged_out": true})).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_static("admin_token=; HttpOnly; SameSite=Strict; Path=/admin; Max-Age=0"),
    );
    Ok(response)
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub async fn change_password(
    State(state): State<Arc<AdminAppState>>,
    claims: AdminClaims,
    Json(body): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, AdminError> {
    let row = sqlx::query_as::<_, AdminUserRow>(
        "SELECT id, email, password_hash, role, enabled FROM admin_users WHERE id = $1",
    )
    .bind(claims.sub)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Internal error: {}", e);
        AdminError::Internal("Internal server error".into())
    })?;

    let valid = bcrypt::verify(&body.current_password, &row.password_hash)
        .map_err(|_| AdminError::InvalidCredentials)?;
    if !valid {
        return Err(AdminError::InvalidCredentials);
    }

    validate_password_strength(&body.new_password)?;

    let new_hash = bcrypt::hash(&body.new_password, 12).map_err(|e| {
        tracing::error!("bcrypt error: {}", e);
        AdminError::Internal("Internal server error".into())
    })?;

    sqlx::query("UPDATE admin_users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind(&new_hash)
        .bind(claims.sub)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Internal error: {}", e);
            AdminError::Internal("Internal server error".into())
        })?;

    Ok(Json(serde_json::json!({"password_changed": true})))
}

// ── Auth Middleware ──────────────────────────────────────────────────

pub async fn admin_auth_middleware(
    State(state): State<Arc<AdminAppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    // SECURITY: Accept token from Authorization header OR HttpOnly cookie.
    // Prefer Authorization header (explicit), fall back to cookie (implicit).
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .or_else(|| {
            // Extract from HttpOnly cookie
            request
                .headers()
                .get(header::COOKIE)
                .and_then(|v| v.to_str().ok())
                .and_then(|cookies| {
                    cookies.split(';').find_map(|c| {
                        let c = c.trim();
                        c.strip_prefix("admin_token=").map(|t| t.to_string())
                    })
                })
        });

    let token = match token {
        Some(ref t) if !t.is_empty() => t.as_str(),
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Missing authentication"})),
            )
                .into_response();
        }
    };

    let claims = match decode::<AdminClaims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => data.claims,
        Err(_e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid token"})),
            )
                .into_response();
        }
    };

    // Check token is not expired (jsonwebtoken does this, but double-check)
    if claims.exp < Utc::now().timestamp() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Token expired"})),
        )
            .into_response();
    }

    // Enforce server-side session revocation:
    // token must still exist in admin_sessions and must not be expired.
    let token_hash = hex::encode(Sha256::digest(token.as_bytes()));
    let session_valid = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM admin_sessions WHERE admin_id = $1 AND token_hash = $2 AND expires_at > NOW())",
    )
    .bind(claims.sub)
    .bind(&token_hash)
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or(false);
    if !session_valid {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid or revoked session"})),
        )
            .into_response();
    }

    // Insert claims as request extension
    request.extensions_mut().insert(claims);
    next.run(request).await
}

/// Extract AdminClaims from request extensions.
/// Used by route handlers.
#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AdminClaims
where
    S: Send + Sync,
{
    type Rejection = AdminError;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AdminClaims>()
            .cloned()
            .ok_or(AdminError::Unauthorized)
    }
}

// ── Password validation ─────────────────────────────────────────────

/// SECURITY: Enforce password complexity beyond just length.
fn validate_password_strength(password: &str) -> Result<(), AdminError> {
    if password.len() < 12 {
        return Err(AdminError::BadRequest(
            "Password must be at least 12 characters".into(),
        ));
    }
    if password.len() > 128 {
        return Err(AdminError::BadRequest(
            "Password must not exceed 128 characters".into(),
        ));
    }
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());
    if !has_upper || !has_lower || !has_digit || !has_special {
        return Err(AdminError::BadRequest(
            "Password must include uppercase, lowercase, digit, and special character".into(),
        ));
    }
    // Block common weak passwords
    let lower = password.to_lowercase();
    let weak = [
        "password1234",
        "admin1234567",
        "123456789012",
        "qwerty123456",
    ];
    if weak.iter().any(|w| lower.contains(w)) {
        return Err(AdminError::BadRequest(
            "Password is too common. Choose a stronger password.".into(),
        ));
    }
    Ok(())
}

// ── Helper: seed initial admin ──────────────────────────────────────

pub async fn seed_initial_admin(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    // Check if any admin exists
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM admin_users")
        .fetch_one(pool)
        .await?;

    if count.0 > 0 {
        return Ok(());
    }

    // Require explicit bootstrap password for the first admin user.
    let password = std::env::var("ADMIN_INITIAL_PASSWORD").map_err(|_| {
        anyhow::anyhow!("ADMIN_INITIAL_PASSWORD must be set to bootstrap the initial admin user")
    })?;

    if password.len() < 12 {
        return Err(anyhow::anyhow!(
            "ADMIN_INITIAL_PASSWORD must be at least 12 characters"
        ));
    }

    let hash = bcrypt::hash(&password, 12)?;

    // SECURITY: Allow configurable initial admin email via env var.
    let admin_email =
        std::env::var("ADMIN_INITIAL_EMAIL").unwrap_or_else(|_| "admin@aiknol.com".into());

    sqlx::query(
        "INSERT INTO admin_users (email, password_hash, role, enabled) VALUES ($1, $2, 'super_admin', true)",
    )
    .bind(&admin_email)
    .bind(&hash)
    .execute(pool)
    .await?;

    info!("Created initial super_admin: {}", admin_email);
    Ok(())
}

// ── Types ───────────────────────────────────────────────────────────

#[derive(Debug, sqlx::FromRow)]
pub struct AdminUserRow {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub enabled: bool,
}

// ── Error ───────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum AdminError {
    InvalidCredentials,
    AccountDisabled,
    Unauthorized,
    Forbidden,
    BadRequest(String),
    NotFound(String),
    Internal(String),
    /// Too many login attempts — includes seconds until window resets.
    RateLimited(u64),
}

impl IntoResponse for AdminError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AdminError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string())
            }
            AdminError::AccountDisabled => (StatusCode::FORBIDDEN, "Account disabled".to_string()),
            AdminError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "Authentication required".to_string(),
            ),
            AdminError::Forbidden => (
                StatusCode::FORBIDDEN,
                "Insufficient permissions".to_string(),
            ),
            AdminError::BadRequest(ref m) => (StatusCode::BAD_REQUEST, m.clone()),
            AdminError::NotFound(ref m) => (StatusCode::NOT_FOUND, m.clone()),
            AdminError::RateLimited(secs) => (
                StatusCode::TOO_MANY_REQUESTS,
                format!("Too many login attempts. Try again in {} seconds", secs),
            ),
            // SECURITY: Never expose internal error details to the client.
            // The actual error is logged server-side before this point.
            AdminError::Internal(internal_msg) => {
                tracing::error!("Internal admin error: {}", internal_msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };
        (status, Json(serde_json::json!({"error": msg}))).into_response()
    }
}

// hex encoding helper (avoid pulling in the `hex` crate)
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bcrypt_hash_and_verify() {
        let password = "test-password-123";
        let hash = bcrypt::hash(password, 4).unwrap();
        assert!(bcrypt::verify(password, &hash).unwrap());
        assert!(!bcrypt::verify("wrong", &hash).unwrap());
    }

    #[test]
    fn test_jwt_roundtrip() {
        let secret = "test-secret-key";
        let claims = AdminClaims {
            sub: Uuid::new_v4(),
            email: "test@test.com".to_string(),
            role: "super_admin".to_string(),
            exp: (Utc::now() + Duration::hours(1)).timestamp(),
            iat: Utc::now().timestamp(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        let decoded = decode::<AdminClaims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .unwrap();

        assert_eq!(decoded.claims.sub, claims.sub);
        assert_eq!(decoded.claims.role, "super_admin");
    }

    #[test]
    fn test_hex_encode() {
        assert_eq!(hex::encode([0xde, 0xad, 0xbe, 0xef]), "deadbeef");
    }
}
