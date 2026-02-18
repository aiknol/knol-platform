//! Public demo configuration endpoint.
//!
//! Returns demo settings + a decrypted LLM API key so the
//! browser-based demo can call the LLM directly.
//! This route does **not** require authentication.

use axum::{extract::State, Json};
use std::sync::Arc;

use crate::crypto;
use crate::AdminAppState;

/// `GET /demo/config` — public, no auth required.
///
/// Returns:
/// ```json
/// {
///   "enabled": true,
///   "llm_provider": "gemini",
///   "llm_model": "gemini-2.0-flash",
///   "llm_api_key": "AIza...",
///   "llm_api_url": "",
///   "github_url": "https://...",
///   "tagline": "Give your AI persistent memory"
/// }
/// ```
pub async fn demo_config(
    State(state): State<Arc<AdminAppState>>,
) -> Json<serde_json::Value> {
    let pool = &state.db_pool;

    // Helper: load a config value, stripping JSON quotes for string types.
    let load = |key: &str, default: &str| {
        let pool = pool.clone();
        let key = key.to_string();
        let default = default.to_string();
        async move {
            let row: Option<(serde_json::Value,)> =
                sqlx::query_as("SELECT value FROM system_config WHERE key = $1")
                    .bind(&key)
                    .fetch_optional(&pool)
                    .await
                    .ok()
                    .flatten();

            match row {
                Some((v,)) => match v {
                    serde_json::Value::String(s) => s,
                    other => other.to_string().trim_matches('"').to_string(),
                },
                None => default,
            }
        }
    };

    let enabled_str = load("demo.enabled", "true").await;
    let enabled = enabled_str == "true";

    if !enabled {
        return Json(serde_json::json!({
            "enabled": false,
            "error": "Demo is disabled"
        }));
    }

    let provider = load("demo.llm_provider", "gemini").await;
    let model_override = load("demo.llm_model", "").await;
    let github_url = load("demo.github_url", "https://github.com/pankajb64/memorylayer").await;
    let tagline = load("demo.tagline", "Give your AI persistent memory").await;

    // Determine model and credential name based on provider
    let (default_model, cred_name, default_api_url_key) = match provider.as_str() {
        "openai" | "open_ai" => ("gpt-4o-mini", "openai_api_key", "llm.openai_api_url"),
        "anthropic" => ("claude-haiku-4-5-20251001", "anthropic_api_key", ""),
        _ => ("gemini-2.0-flash", "gemini_api_key", "llm.gemini_api_url"),
    };

    let model = if model_override.is_empty() {
        // Try provider-specific model config, else use default
        let model_key = match provider.as_str() {
            "openai" | "open_ai" => "llm.openai_model",
            "anthropic" => "llm.extraction_model",
            _ => "llm.gemini_model",
        };
        let m = load(model_key, default_model).await;
        if m.is_empty() { default_model.to_string() } else { m }
    } else {
        model_override
    };

    // Load the API URL (for OpenAI/Gemini custom endpoints)
    let api_url = if !default_api_url_key.is_empty() {
        load(default_api_url_key, "").await
    } else {
        String::new()
    };

    // Security default: do not expose raw provider API keys on a public route
    // unless explicitly enabled for local demo scenarios.
    let expose_public_key = load("demo.expose_public_llm_key", "false").await == "true";
    let api_key = if expose_public_key {
        load_decrypted_credential(pool, &state.encryption_key, cred_name)
            .await
            .unwrap_or_default()
    } else {
        String::new()
    };

    Json(serde_json::json!({
        "enabled": true,
        "llm_provider": provider,
        "llm_model": model,
        "llm_api_key": api_key,
        "llm_api_key_exposed": expose_public_key,
        "llm_api_url": api_url,
        "github_url": github_url,
        "tagline": tagline,
    }))
}

/// Decrypt a credential by name.
async fn load_decrypted_credential(
    pool: &sqlx::PgPool,
    key: &[u8; 32],
    name: &str,
) -> Option<String> {
    let row: Option<(Vec<u8>,)> =
        sqlx::query_as("SELECT encrypted_value FROM system_credentials WHERE name = $1")
            .bind(name)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    let encrypted = row?.0;
    let decrypted = crypto::decrypt(&encrypted, key).ok()?;
    String::from_utf8(decrypted).ok()
}
