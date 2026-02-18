//! Admin authentication — JWT login, middleware, bcrypt password hashing.

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
use tracing::{info, warn};
use uuid::Uuid;

use crate::AdminAppState;

// ── JWT Claims ──────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminClaims {
    pub sub: Uuid,       // admin_id
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

pub async fn login(
    State(state): State<Arc<AdminAppState>>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AdminError> {
    // Find admin user
    let normalized_email = body.email.trim().to_lowercase();
    let row = sqlx::query_as::<_, AdminUserRow>(
        "SELECT id, email, password_hash, role, enabled FROM admin_users WHERE email = $1",
    )
    .bind(&normalized_email)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AdminError::Internal(e.to_string()))?
    .ok_or(AdminError::InvalidCredentials)?;

    if !row.enabled {
        return Err(AdminError::AccountDisabled);
    }

    // Verify password
    let valid = bcrypt::verify(&body.password, &row.password_hash)
        .map_err(|_| AdminError::InvalidCredentials)?;
    if !valid {
        warn!("Failed login attempt");
        return Err(AdminError::InvalidCredentials);
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
    .map_err(|e| AdminError::Internal(format!("JWT encode: {}", e)))?;

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

    info!("Admin login: {}", row.email);

    Ok(Json(LoginResponse {
        token,
        admin: AdminInfo {
            id: row.id,
            email: row.email,
            role: row.role,
        },
        expires_at: expires.to_rfc3339(),
    }))
}

pub async fn logout(
    State(state): State<Arc<AdminAppState>>,
    claims: AdminClaims,
) -> Result<Json<serde_json::Value>, AdminError> {
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

    Ok(Json(serde_json::json!({"logged_out": true})))
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
    .map_err(|e| AdminError::Internal(e.to_string()))?;

    let valid = bcrypt::verify(&body.current_password, &row.password_hash)
        .map_err(|_| AdminError::InvalidCredentials)?;
    if !valid {
        return Err(AdminError::InvalidCredentials);
    }

    if body.new_password.len() < 8 {
        return Err(AdminError::BadRequest("Password must be at least 8 characters".into()));
    }

    let new_hash = bcrypt::hash(&body.new_password, 12)
        .map_err(|e| AdminError::Internal(format!("bcrypt: {}", e)))?;

    sqlx::query("UPDATE admin_users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind(&new_hash)
        .bind(claims.sub)
        .execute(&state.db_pool)
        .await
        .map_err(|e| AdminError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({"password_changed": true})))
}

// ── Auth Middleware ──────────────────────────────────────────────────

pub async fn admin_auth_middleware(
    State(state): State<Arc<AdminAppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));

    let token = match token {
        Some(t) => t,
        None => {
            return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Missing Authorization header"}))).into_response();
        }
    };

    let claims = match decode::<AdminClaims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => data.claims,
        Err(_e) => {
            return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Invalid token"}))).into_response();
        }
    };

    // Check token is not expired (jsonwebtoken does this, but double-check)
    if claims.exp < Utc::now().timestamp() {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Token expired"}))).into_response();
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
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Invalid or revoked session"}))).into_response();
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
        anyhow::anyhow!(
            "ADMIN_INITIAL_PASSWORD must be set to bootstrap the initial admin user"
        )
    })?;

    let hash = bcrypt::hash(&password, 12)?;

    sqlx::query(
        "INSERT INTO admin_users (email, password_hash, role, enabled) VALUES ('admin@aiknol.com', $1, 'super_admin', true)",
    )
    .bind(&hash)
    .execute(pool)
    .await?;

    info!("Created initial super_admin: admin@aiknol.com");
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
}

impl IntoResponse for AdminError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AdminError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            AdminError::AccountDisabled => (StatusCode::FORBIDDEN, "Account disabled"),
            AdminError::Unauthorized => (StatusCode::UNAUTHORIZED, "Authentication required"),
            AdminError::Forbidden => (StatusCode::FORBIDDEN, "Insufficient permissions"),
            AdminError::BadRequest(ref m) => (StatusCode::BAD_REQUEST, m.as_str()),
            AdminError::NotFound(ref m) => (StatusCode::NOT_FOUND, m.as_str()),
            AdminError::Internal(ref m) => (StatusCode::INTERNAL_SERVER_ERROR, m.as_str()),
        };
        (status, Json(serde_json::json!({"error": msg}))).into_response()
    }
}

// hex encoding helper (avoid pulling in the `hex` crate)
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
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
