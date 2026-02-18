//! Encrypted credential management — store/retrieve/rotate API keys.

use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::{AdminClaims, AdminError};
use crate::crypto;
use crate::AdminAppState;

pub async fn list_credentials(
    State(state): State<Arc<AdminAppState>>,
    _claims: AdminClaims,
) -> Result<Json<Vec<serde_json::Value>>, AdminError> {
    let rows = sqlx::query_as::<_, CredentialListRow>(
        "SELECT id, name, service, description, last_rotated_at, updated_at FROM system_credentials ORDER BY service, name",
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AdminError::Internal(e.to_string()))?;

    let json: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "name": r.name,
                "service": r.service,
                "description": r.description,
                "last_rotated_at": r.last_rotated_at.to_rfc3339(),
                "updated_at": r.updated_at.to_rfc3339(),
                "value": "••••••••",  // Never expose actual values in list
            })
        })
        .collect();

    Ok(Json(json))
}

#[derive(Deserialize)]
pub struct UpsertCredential {
    pub value: String,
    #[serde(default = "default_general")]
    pub service: String,
    #[serde(default)]
    pub description: String,
}

fn default_general() -> String { "general".into() }

pub async fn upsert_credential(
    State(state): State<Arc<AdminAppState>>,
    claims: AdminClaims,
    Path(name): Path<String>,
    Json(body): Json<UpsertCredential>,
) -> Result<Json<serde_json::Value>, AdminError> {
    if claims.role == "read_only" {
        return Err(AdminError::Forbidden);
    }

    // Encrypt the value
    let encrypted = crypto::encrypt(body.value.as_bytes(), &state.encryption_key)
        .map_err(|e| AdminError::Internal(format!("Encryption failed: {}", e)))?;

    // Check if exists (for audit)
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM system_credentials WHERE name = $1)",
    )
    .bind(&name)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AdminError::Internal(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO system_credentials (name, encrypted_value, service, description, last_rotated_at, updated_at)
        VALUES ($1, $2, $3, $4, NOW(), NOW())
        ON CONFLICT (name) DO UPDATE SET
            encrypted_value = EXCLUDED.encrypted_value,
            service = EXCLUDED.service,
            description = EXCLUDED.description,
            last_rotated_at = NOW(),
            updated_at = NOW()
        "#,
    )
    .bind(&name)
    .bind(&encrypted)
    .bind(&body.service)
    .bind(&body.description)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AdminError::Internal(e.to_string()))?;

    // Audit (don't log actual credential value)
    let _ = sqlx::query(
        "INSERT INTO admin_audit_log (admin_id, admin_email, action, resource_type, resource_key, new_value) VALUES ($1, $2, $3, 'credential', $4, $5)",
    )
    .bind(claims.sub)
    .bind(&claims.email)
    .bind(if exists { "update" } else { "create" })
    .bind(&name)
    .bind(serde_json::json!({"service": body.service, "rotated": true}))
    .execute(&state.db_pool)
    .await;

    Ok(Json(serde_json::json!({
        "name": name,
        "action": if exists { "rotated" } else { "created" },
    })))
}

pub async fn delete_credential(
    State(state): State<Arc<AdminAppState>>,
    claims: AdminClaims,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, AdminError> {
    if claims.role != "super_admin" {
        return Err(AdminError::Forbidden);
    }

    sqlx::query("DELETE FROM system_credentials WHERE name = $1")
        .bind(&name)
        .execute(&state.db_pool)
        .await
        .map_err(|e| AdminError::Internal(e.to_string()))?;

    let _ = sqlx::query(
        "INSERT INTO admin_audit_log (admin_id, admin_email, action, resource_type, resource_key) VALUES ($1, $2, 'delete', 'credential', $3)",
    )
    .bind(claims.sub)
    .bind(&claims.email)
    .bind(&name)
    .execute(&state.db_pool)
    .await;

    Ok(Json(serde_json::json!({"name": name, "deleted": true})))
}

/// Test a credential by making a lightweight API call.
pub async fn test_credential(
    State(state): State<Arc<AdminAppState>>,
    _claims: AdminClaims,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, AdminError> {
    // Fetch and decrypt
    let row = sqlx::query_as::<_, CredentialRow>(
        "SELECT encrypted_value, service FROM system_credentials WHERE name = $1",
    )
    .bind(&name)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AdminError::Internal(e.to_string()))?
    .ok_or_else(|| AdminError::NotFound(format!("Credential '{}' not found", name)))?;

    let decrypted = crypto::decrypt(&row.encrypted_value, &state.encryption_key)
        .map_err(|e| AdminError::Internal(format!("Decryption failed: {}", e)))?;
    let value = String::from_utf8(decrypted)
        .map_err(|_| AdminError::Internal("Invalid UTF-8 in credential".into()))?;

    // Test based on service type
    let (valid, message) = match row.service.as_str() {
        "twitter" => test_twitter_token(&value).await,
        "github" => test_github_token(&state, &value).await,
        "devto" => test_devto_token(&state, &value).await,
        "anthropic" => test_anthropic_key(&state, &value).await,
        "openai" => test_openai_key(&state, &value).await,
        "gemini" | "google" => test_gemini_key(&state, &value).await,
        _ => (true, format!("Credential '{}' decrypted successfully (no specific test for '{}')", name, row.service)),
    };

    Ok(Json(serde_json::json!({
        "name": name,
        "service": row.service,
        "valid": valid,
        "message": message,
    })))
}

async fn test_twitter_token(_value: &str) -> (bool, String) {
    // Twitter OAuth requires multi-key auth; just verify it's non-empty
    if _value.len() > 10 {
        (true, "Token appears valid (format check only)".into())
    } else {
        (false, "Token too short".into())
    }
}

async fn test_github_token(state: &AdminAppState, token: &str) -> (bool, String) {
    let resp = state
        .http_client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "knol-admin/0.1.0")
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => (true, "GitHub token is valid".into()),
        Ok(r) => (false, format!("GitHub API returned {}", r.status())),
        Err(e) => (false, format!("Request failed: {}", e)),
    }
}

async fn test_devto_token(state: &AdminAppState, token: &str) -> (bool, String) {
    let resp = state
        .http_client
        .get("https://dev.to/api/users/me")
        .header("api-key", token)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => (true, "Dev.to API key is valid".into()),
        Ok(r) => (false, format!("Dev.to API returned {}", r.status())),
        Err(e) => (false, format!("Request failed: {}", e)),
    }
}

async fn test_anthropic_key(state: &AdminAppState, key: &str) -> (bool, String) {
    let resp = state
        .http_client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .body(r#"{"model":"claude-haiku-4-5-20251001","max_tokens":1,"messages":[{"role":"user","content":"hi"}]}"#)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => (true, "Anthropic API key is valid".into()),
        Ok(r) if r.status().as_u16() == 401 => (false, "Anthropic API key is invalid (401 Unauthorized)".into()),
        Ok(r) => {
            let status = r.status();
            let body = r.text().await.unwrap_or_default();
            if status.as_u16() == 400 || status.as_u16() == 429 {
                // 400 (bad request) or 429 (rate limited) means the key itself is valid
                (true, format!("Anthropic API key is valid (got {})", status))
            } else {
                (false, format!("Anthropic API returned {}: {}", status, &body[..body.len().min(200)]))
            }
        }
        Err(e) => (false, format!("Request failed: {}", e)),
    }
}

async fn test_openai_key(state: &AdminAppState, key: &str) -> (bool, String) {
    let resp = state
        .http_client
        .get("https://api.openai.com/v1/models")
        .header("Authorization", format!("Bearer {}", key))
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => (true, "OpenAI API key is valid".into()),
        Ok(r) if r.status().as_u16() == 401 => (false, "OpenAI API key is invalid (401 Unauthorized)".into()),
        Ok(r) => (false, format!("OpenAI API returned {}", r.status())),
        Err(e) => (false, format!("Request failed: {}", e)),
    }
}

async fn test_gemini_key(state: &AdminAppState, key: &str) -> (bool, String) {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models?key={}",
        key
    );
    let resp = state.http_client.get(&url).send().await;

    match resp {
        Ok(r) if r.status().is_success() => (true, "Gemini API key is valid".into()),
        Ok(r) if r.status().as_u16() == 400 || r.status().as_u16() == 403 => {
            let body = r.text().await.unwrap_or_default();
            if body.contains("API_KEY_INVALID") || body.contains("expired") {
                (false, "Gemini API key is invalid or expired".into())
            } else {
                (false, format!("Gemini API error: {}", &body[..body.len().min(200)]))
            }
        }
        Ok(r) => (false, format!("Gemini API returned {}", r.status())),
        Err(e) => (false, format!("Request failed: {}", e)),
    }
}

/// Load a decrypted credential by name — used by other services via config_loader.
pub async fn load_credential(
    pool: &sqlx::PgPool,
    key: &[u8; 32],
    name: &str,
) -> Option<String> {
    let row = sqlx::query_as::<_, CredentialRow>(
        "SELECT encrypted_value, service FROM system_credentials WHERE name = $1",
    )
    .bind(name)
    .fetch_optional(pool)
    .await
    .ok()??;

    let decrypted = crypto::decrypt(&row.encrypted_value, key).ok()?;
    String::from_utf8(decrypted).ok()
}

#[derive(sqlx::FromRow)]
struct CredentialListRow {
    id: Uuid,
    name: String,
    service: String,
    description: String,
    last_rotated_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
struct CredentialRow {
    encrypted_value: Vec<u8>,
    service: String,
}
