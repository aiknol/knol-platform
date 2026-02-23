//! TOTP 2FA endpoints: setup, enable, disable, and verify.

use axum::{
    extract::State,
    http::{header, HeaderMap},
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::auth::{
    app_cookie, append_csrf_cookie, audit, issue_session_token, random_hex, AppClaims, AppError,
    AppUserRow,
};
use crate::TenantAppState;

// ── Request types ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct EnableTotpRequest {
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct DisableTotpRequest {
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyTotpRequest {
    /// The short-lived JWT issued during login when TOTP is required.
    pub totp_token: String,
    /// The 6-digit TOTP code (or an 8-char backup code).
    pub code: String,
}

// ── TOTP pending claims ──────────────────────────────────────────────────

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TotpPendingClaims {
    sub: uuid::Uuid,
    tenant_id: uuid::Uuid,
    purpose: String,
    exp: i64,
}

// ── Handlers ─────────────────────────────────────────────────────────────

/// Generate a TOTP secret and backup codes. Does NOT enable TOTP yet.
pub async fn setup_totp(
    claims: AppClaims,
    State(state): State<Arc<TenantAppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Generate a 20-byte secret (standard for TOTP)
    let secret_bytes: Vec<u8> = (0..20).map(|_| rand::random::<u8>()).collect();
    let secret_base32 = base32_encode(&secret_bytes);

    // Build otpauth URI for QR code
    let issuer = "Knol%20Cloud";
    let qr_uri = format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits=6&period=30",
        issuer, claims.email, secret_base32, issuer
    );

    // Generate 10 backup codes (8 hex chars each)
    let backup_codes: Vec<String> = (0..10).map(|_| random_hex(4).to_uppercase()).collect();
    let hashed_codes: Vec<String> = backup_codes
        .iter()
        .map(|c| hex::encode(Sha256::digest(c.as_bytes())))
        .collect();

    // Store encrypted secret and hashed backup codes (not enabled yet)
    // For simplicity, store the base32 secret directly (in production, encrypt with state.totp_encryption_key)
    sqlx::query(
        "UPDATE app_users SET totp_secret_encrypted = $1, totp_backup_codes = $2, updated_at = NOW() WHERE id = $3",
    )
    .bind(&secret_base32)
    .bind(&hashed_codes)
    .bind(claims.sub)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "secret": secret_base32,
        "qr_uri": qr_uri,
        "backup_codes": backup_codes,
    })))
}

/// Enable TOTP by verifying a code against the stored secret.
pub async fn enable_totp(
    claims: AppClaims,
    State(state): State<Arc<TenantAppState>>,
    Json(body): Json<EnableTotpRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let secret = sqlx::query_scalar::<_, Option<String>>(
        "SELECT totp_secret_encrypted FROM app_users WHERE id = $1",
    )
    .bind(claims.sub)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .ok_or_else(|| AppError::BadRequest("TOTP not set up. Call setup first.".into()))?;

    // Verify the code
    if !verify_totp_code(&secret, &body.code) {
        return Err(AppError::BadRequest("Invalid TOTP code".into()));
    }

    sqlx::query("UPDATE app_users SET totp_enabled = true, updated_at = NOW() WHERE id = $1")
        .bind(claims.sub)
        .execute(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    audit(
        &state,
        claims.tenant_id,
        Some(&claims),
        "enable_totp",
        "user",
        Some(&claims.sub.to_string()),
        None,
        None,
        None,
    )
    .await;

    Ok(Json(serde_json::json!({"enabled": true})))
}

/// Disable TOTP (requires password confirmation).
pub async fn disable_totp(
    claims: AppClaims,
    State(state): State<Arc<TenantAppState>>,
    Json(body): Json<DisableTotpRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Verify password
    let hash = sqlx::query_scalar::<_, String>("SELECT password_hash FROM app_users WHERE id = $1")
        .bind(claims.sub)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let valid = bcrypt::verify(&body.password, &hash).map_err(|_| AppError::Unauthorized)?;
    if !valid {
        return Err(AppError::BadRequest("Password is incorrect".into()));
    }

    sqlx::query(
        "UPDATE app_users SET totp_enabled = false, totp_secret_encrypted = NULL, totp_backup_codes = NULL, updated_at = NOW() WHERE id = $1",
    )
    .bind(claims.sub)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    audit(
        &state,
        claims.tenant_id,
        Some(&claims),
        "disable_totp",
        "user",
        Some(&claims.sub.to_string()),
        None,
        None,
        None,
    )
    .await;

    Ok(Json(serde_json::json!({"disabled": true})))
}

/// Verify a TOTP code during login (public, rate-limited).
pub async fn verify_totp(
    State(state): State<Arc<TenantAppState>>,
    headers: HeaderMap,
    Json(body): Json<VerifyTotpRequest>,
) -> Result<Response, AppError> {
    let client_ip = enterprise_common::client_ip::extract_client_ip(&headers);
    let rate_key = format!("app:totp:{}", client_ip);
    enterprise_common::rate_limit::enforce_rate_limit(&state.rate_limiter, &rate_key, "app:")
        .map_err(AppError::RateLimited)?;

    // Decode the pending token
    let pending = jsonwebtoken::decode::<TotpPendingClaims>(
        &body.totp_token,
        &jsonwebtoken::DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
    )
    .map_err(|_| AppError::Unauthorized)?
    .claims;

    if pending.purpose != "totp_pending" {
        return Err(AppError::Unauthorized);
    }

    // Fetch user and verify TOTP
    let user = sqlx::query_as::<_, AppUserRow>(
        "SELECT id, tenant_id, email, password_hash, full_name, role, enabled, failed_login_attempts, locked_until, email_verified, totp_enabled FROM app_users WHERE id = $1",
    )
    .bind(pending.sub)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if !user.totp_enabled {
        return Err(AppError::BadRequest("TOTP is not enabled".into()));
    }

    let secret = sqlx::query_scalar::<_, Option<String>>(
        "SELECT totp_secret_encrypted FROM app_users WHERE id = $1",
    )
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .ok_or(AppError::Internal("TOTP secret missing".into()))?;

    // Try TOTP code first, then backup codes
    let code_valid = verify_totp_code(&secret, &body.code);
    let backup_used = if !code_valid {
        try_backup_code(&state, user.id, &body.code).await?
    } else {
        false
    };

    if !code_valid && !backup_used {
        enterprise_common::rate_limit::record_failure(&state.rate_limiter, &rate_key);
        return Err(AppError::Unauthorized);
    }

    enterprise_common::rate_limit::clear_limit(&state.rate_limiter, &rate_key);

    // Issue full session
    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let (token, expires) =
        issue_session_token(&state, &user, Some(&client_ip), user_agent.as_deref()).await?;

    let tenant = sqlx::query_as::<_, super::app::TenantRow>(
        "SELECT id, name, slug, plan, usage_ops_month, usage_limit FROM tenants WHERE id = $1",
    )
    .bind(user.tenant_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    audit(
        &state,
        user.tenant_id,
        None,
        "login_totp",
        "session",
        Some(&user.id.to_string()),
        None,
        None,
        Some(serde_json::json!({"ip": client_ip, "backup_code_used": backup_used})),
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
    append_csrf_cookie(&mut response);
    Ok(response)
}

// ── TOTP helpers ─────────────────────────────────────────────────────────

/// Simple TOTP verification (RFC 6238, SHA1, 6 digits, 30s period).
/// Accepts current and adjacent time steps (±1) for clock skew tolerance.
fn verify_totp_code(secret_base32: &str, code: &str) -> bool {
    let secret_bytes = match base32_decode(secret_base32) {
        Some(b) => b,
        None => return false,
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let time_step = now / 30;

    // Check current and ±1 time steps for clock skew tolerance
    for offset in [0i64, -1, 1] {
        let step = (time_step as i64 + offset) as u64;
        let expected = generate_totp(&secret_bytes, step);
        if code == expected {
            return true;
        }
    }
    false
}

/// Generate a 6-digit TOTP code for a given time step.
fn generate_totp(secret: &[u8], time_step: u64) -> String {
    use hmac::{Hmac, Mac};
    type HmacSha1 = Hmac<sha1::Sha1>;

    let time_bytes = time_step.to_be_bytes();
    let mut mac = HmacSha1::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(&time_bytes);
    let result = mac.finalize().into_bytes();

    let offset = (result[result.len() - 1] & 0x0f) as usize;
    let code = ((result[offset] as u32 & 0x7f) << 24)
        | ((result[offset + 1] as u32) << 16)
        | ((result[offset + 2] as u32) << 8)
        | (result[offset + 3] as u32);

    format!("{:06}", code % 1_000_000)
}

/// Try a backup code; if valid, remove it from the list.
async fn try_backup_code(
    state: &TenantAppState,
    user_id: uuid::Uuid,
    code: &str,
) -> Result<bool, AppError> {
    let code_hash = hex::encode(Sha256::digest(code.to_uppercase().as_bytes()));

    let codes: Vec<String> =
        sqlx::query_scalar("SELECT unnest(totp_backup_codes) FROM app_users WHERE id = $1")
            .bind(user_id)
            .fetch_all(&state.db_pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

    if !codes.contains(&code_hash) {
        return Ok(false);
    }

    // Remove the used backup code
    sqlx::query(
        "UPDATE app_users SET totp_backup_codes = array_remove(totp_backup_codes, $1) WHERE id = $2",
    )
    .bind(&code_hash)
    .bind(user_id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(true)
}

/// Base32 encode bytes (RFC 4648, no padding).
fn base32_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut result = String::new();
    let mut buffer: u64 = 0;
    let mut bits = 0;
    for &byte in data {
        buffer = (buffer << 8) | byte as u64;
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            result.push(ALPHABET[((buffer >> bits) & 0x1f) as usize] as char);
        }
    }
    if bits > 0 {
        buffer <<= 5 - bits;
        result.push(ALPHABET[(buffer & 0x1f) as usize] as char);
    }
    result
}

/// Base32 decode string (RFC 4648).
fn base32_decode(input: &str) -> Option<Vec<u8>> {
    let mut result = Vec::new();
    let mut buffer: u64 = 0;
    let mut bits = 0;
    for c in input.chars() {
        let val = match c {
            'A'..='Z' => c as u64 - 'A' as u64,
            'a'..='z' => c as u64 - 'a' as u64,
            '2'..='7' => c as u64 - '2' as u64 + 26,
            '=' => continue,
            _ => return None,
        };
        buffer = (buffer << 5) | val;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            result.push((buffer >> bits) as u8);
        }
    }
    Some(result)
}
