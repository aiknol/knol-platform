//! System config CRUD — runtime settings backed by database.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::{AdminClaims, AdminError};
use crate::AdminAppState;

#[derive(Deserialize)]
pub struct ConfigFilter {
    pub category: Option<String>,
}

fn is_sensitive_config_key(key: &str, env_override: Option<&str>) -> bool {
    let key_l = key.to_ascii_lowercase();
    let env_l = env_override.unwrap_or_default().to_ascii_lowercase();

    // Known non-secret keys that include "key" in the name.
    if key_l == "gateway.api_key_header" {
        return false;
    }

    fn has_sensitive_marker(haystack: &str, marker: &str) -> bool {
        if haystack == marker {
            return true;
        }
        let delimiters = [".", "_", "-"];
        delimiters.iter().any(|d| {
            haystack.contains(&format!("{d}{marker}"))
                || haystack.contains(&format!("{marker}{d}"))
        })
    }

    // High-signal secret markers in keys and env aliases.
    let markers = [
        "secret",
        "password",
        "token",
        "api_key",
        "private_key",
        "access_key",
        "encryption_key",
        "signing_key",
        "jwt",
    ];
    markers.iter().any(|m| has_sensitive_marker(&key_l, m) || has_sensitive_marker(&env_l, m))
}

fn redact_config_value(
    key: &str,
    env_override: Option<&str>,
    value: &serde_json::Value,
) -> serde_json::Value {
    if is_sensitive_config_key(key, env_override) {
        serde_json::json!("••••••••")
    } else {
        value.clone()
    }
}

pub async fn list_configs(
    State(state): State<Arc<AdminAppState>>,
    _claims: AdminClaims,
    Query(filter): Query<ConfigFilter>,
) -> Result<Json<Vec<serde_json::Value>>, AdminError> {
    let rows = if let Some(cat) = &filter.category {
        sqlx::query_as::<_, ConfigRow>(
            "SELECT id, key, value, value_type, category, description, env_override, updated_at FROM system_config WHERE category = $1 ORDER BY key",
        )
        .bind(cat)
        .fetch_all(&state.db_pool)
        .await
    } else {
        sqlx::query_as::<_, ConfigRow>(
            "SELECT id, key, value, value_type, category, description, env_override, updated_at FROM system_config ORDER BY category, key",
        )
        .fetch_all(&state.db_pool)
        .await
    }
    .map_err(|e| AdminError::Internal(e.to_string()))?;

    let json: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "key": r.key,
                "value": redact_config_value(&r.key, r.env_override.as_deref(), &r.value),
                "value_type": r.value_type,
                "category": r.category,
                "description": r.description,
                "env_override": r.env_override,
                "updated_at": r.updated_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(json))
}

pub async fn get_config(
    State(state): State<Arc<AdminAppState>>,
    _claims: AdminClaims,
    Path(key): Path<String>,
) -> Result<Json<serde_json::Value>, AdminError> {
    let row = sqlx::query_as::<_, ConfigRow>(
        "SELECT id, key, value, value_type, category, description, env_override, updated_at FROM system_config WHERE key = $1",
    )
    .bind(&key)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AdminError::Internal(e.to_string()))?
    .ok_or_else(|| AdminError::NotFound(format!("Config key '{}' not found", key)))?;

    Ok(Json(serde_json::json!({
        "id": row.id,
        "key": row.key,
        "value": redact_config_value(&row.key, row.env_override.as_deref(), &row.value),
        "value_type": row.value_type,
        "category": row.category,
        "description": row.description,
        "env_override": row.env_override,
        "updated_at": row.updated_at.to_rfc3339(),
    })))
}

#[derive(Deserialize)]
pub struct UpsertConfig {
    pub value: serde_json::Value,
    #[serde(default = "default_string")]
    pub value_type: String,
    #[serde(default = "default_general")]
    pub category: String,
    #[serde(default)]
    pub description: String,
    pub env_override: Option<String>,
}

fn default_string() -> String { "string".into() }
fn default_general() -> String { "general".into() }

fn validate_config_update(key: &str, value: &serde_json::Value, value_type: &str) -> Result<(), AdminError> {
    let valid_types = ["string", "number", "boolean", "json", "string_array"];
    if !valid_types.contains(&value_type) {
        return Err(AdminError::BadRequest(format!(
            "Invalid value_type '{}'. Must be one of: {:?}",
            value_type, valid_types
        )));
    }

    match value_type {
        "string" => {
            if !value.is_string() {
                return Err(AdminError::BadRequest("Value must be a string".into()));
            }
        }
        "number" => {
            if !value.is_number() {
                return Err(AdminError::BadRequest("Value must be a number".into()));
            }
        }
        "boolean" => {
            if !value.is_boolean() {
                return Err(AdminError::BadRequest("Value must be a boolean".into()));
            }
        }
        "string_array" => {
            let valid = value
                .as_array()
                .map(|arr| arr.iter().all(|v| v.is_string()))
                .unwrap_or(false);
            if !valid {
                return Err(AdminError::BadRequest(
                    "Value must be a JSON array of strings".into(),
                ));
            }
        }
        "json" => {}
        _ => {}
    }

    if key == "llm.provider" {
        let provider = value
            .as_str()
            .ok_or_else(|| AdminError::BadRequest("llm.provider must be a string".into()))?
            .to_lowercase();
        let valid_providers = ["anthropic", "openai", "gemini"];
        if !valid_providers.contains(&provider.as_str()) {
            return Err(AdminError::BadRequest(format!(
                "Invalid llm.provider '{}'. Must be one of: anthropic, openai, gemini",
                provider
            )));
        }
    }

    Ok(())
}

pub async fn upsert_config(
    State(state): State<Arc<AdminAppState>>,
    claims: AdminClaims,
    Path(key): Path<String>,
    Json(body): Json<UpsertConfig>,
) -> Result<Json<serde_json::Value>, AdminError> {
    if claims.role == "read_only" {
        return Err(AdminError::Forbidden);
    }

    if is_sensitive_config_key(&key, body.env_override.as_deref()) {
        return Err(AdminError::BadRequest(
            "Sensitive values are not allowed in system_config. Use encrypted /admin/credentials instead.".into(),
        ));
    }

    validate_config_update(&key, &body.value, &body.value_type)?;

    // Get old value for audit
    let old = sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT value FROM system_config WHERE key = $1",
    )
    .bind(&key)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AdminError::Internal(e.to_string()))?;

    // Upsert
    sqlx::query(
        r#"
        INSERT INTO system_config (key, value, value_type, category, description, env_override, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, NOW())
        ON CONFLICT (key) DO UPDATE SET
            value = EXCLUDED.value,
            value_type = EXCLUDED.value_type,
            category = EXCLUDED.category,
            description = EXCLUDED.description,
            env_override = EXCLUDED.env_override,
            updated_at = NOW()
        "#,
    )
    .bind(&key)
    .bind(&body.value)
    .bind(&body.value_type)
    .bind(&body.category)
    .bind(&body.description)
    .bind(&body.env_override)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AdminError::Internal(e.to_string()))?;

    // Audit log
    let old_audit = old
        .as_ref()
        .map(|v| redact_config_value(&key, body.env_override.as_deref(), v));
    let new_audit = redact_config_value(&key, body.env_override.as_deref(), &body.value);
    let _ = sqlx::query(
        "INSERT INTO admin_audit_log (admin_id, admin_email, action, resource_type, resource_key, old_value, new_value) VALUES ($1, $2, $3, 'config', $4, $5, $6)",
    )
    .bind(claims.sub)
    .bind(&claims.email)
    .bind(if old.is_some() { "update" } else { "create" })
    .bind(&key)
    .bind(&old_audit)
    .bind(&new_audit)
    .execute(&state.db_pool)
    .await;

    Ok(Json(serde_json::json!({"key": key, "updated": true})))
}

pub async fn delete_config(
    State(state): State<Arc<AdminAppState>>,
    claims: AdminClaims,
    Path(key): Path<String>,
) -> Result<Json<serde_json::Value>, AdminError> {
    if claims.role != "super_admin" {
        return Err(AdminError::Forbidden);
    }

    let env_override = sqlx::query_scalar::<_, Option<String>>(
        "SELECT env_override FROM system_config WHERE key = $1",
    )
    .bind(&key)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AdminError::Internal(e.to_string()))?
    .flatten();

    let old = sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT value FROM system_config WHERE key = $1",
    )
    .bind(&key)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AdminError::Internal(e.to_string()))?;

    sqlx::query("DELETE FROM system_config WHERE key = $1")
        .bind(&key)
        .execute(&state.db_pool)
        .await
        .map_err(|e| AdminError::Internal(e.to_string()))?;

    let old_audit = old
        .as_ref()
        .map(|v| redact_config_value(&key, env_override.as_deref(), v));
    let _ = sqlx::query(
        "INSERT INTO admin_audit_log (admin_id, admin_email, action, resource_type, resource_key, old_value) VALUES ($1, $2, 'delete', 'config', $3, $4)",
    )
    .bind(claims.sub)
    .bind(&claims.email)
    .bind(&key)
    .bind(&old_audit)
    .execute(&state.db_pool)
    .await;

    Ok(Json(serde_json::json!({"key": key, "deleted": true})))
}

#[derive(sqlx::FromRow)]
struct ConfigRow {
    id: Uuid,
    key: String,
    value: serde_json::Value,
    value_type: String,
    category: String,
    description: String,
    env_override: Option<String>,
    updated_at: chrono::DateTime<chrono::Utc>,
}
