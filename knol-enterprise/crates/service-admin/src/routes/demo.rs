//! Public demo configuration and extraction endpoints.
//!
//! These routes are intentionally unauthenticated for live demos, so they
//! must never return decrypted credentials.

use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;
use std::sync::Arc;

use crate::routes::credentials;
use crate::AdminAppState;

const DEFAULT_GITHUB_URL: &str = "https://github.com/aiknol/knol";
const DEFAULT_TAGLINE: &str = "Context engineering for AI applications";
const DEFAULT_GEMINI_API_URL: &str = "https://generativelanguage.googleapis.com/v1beta";
const DEFAULT_OPENAI_API_URL: &str = "https://api.openai.com/v1";
const DEFAULT_ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";

type DemoHttpError = (StatusCode, Json<serde_json::Value>);

#[derive(Debug, Clone)]
struct DemoRuntimeConfig {
    provider: String,
    model: String,
    api_url: String,
    credential_name: String,
}

#[derive(Debug, Deserialize)]
pub struct DemoExtractRequest {
    pub user_message: String,
    #[serde(default)]
    pub memory_context: String,
}

/// `GET /demo/config` — public demo metadata only.
///
/// No plaintext key material is returned.
pub async fn demo_config(State(state): State<Arc<AdminAppState>>) -> Json<serde_json::Value> {
    let pool = &state.db_pool;
    let enabled = load_bool_config(pool, "demo.enabled", true).await;

    if !enabled {
        return Json(serde_json::json!({
            "enabled": false,
            "error": "Demo is disabled"
        }));
    }

    let runtime = resolve_runtime_config(pool).await;
    let github_url = load_config_value(pool, "demo.github_url", DEFAULT_GITHUB_URL).await;
    let tagline = load_config_value(pool, "demo.tagline", DEFAULT_TAGLINE).await;
    let llm_ready =
        credentials::load_credential(pool, &state.encryption_key, &runtime.credential_name)
            .await
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);

    Json(serde_json::json!({
        "enabled": true,
        "llm_provider": runtime.provider,
        "llm_model": runtime.model,
        "llm_api_url": runtime.api_url,
        "llm_ready": llm_ready,
        "github_url": github_url,
        "tagline": tagline,
    }))
}

/// `POST /demo/extract` — run the configured LLM server-side.
///
/// Expected payload:
/// ```json
/// {
///   "user_message": "text from demo user",
///   "memory_context": "[0] (fact) existing memory"
/// }
/// ```
pub async fn demo_extract(
    State(state): State<Arc<AdminAppState>>,
    Json(body): Json<DemoExtractRequest>,
) -> Result<Json<serde_json::Value>, DemoHttpError> {
    if body.user_message.trim().is_empty() {
        return Err(error_json(
            StatusCode::BAD_REQUEST,
            "user_message must not be empty",
        ));
    }
    if body.user_message.len() > 16_000 {
        return Err(error_json(
            StatusCode::PAYLOAD_TOO_LARGE,
            "user_message is too large",
        ));
    }

    let pool = &state.db_pool;
    if !load_bool_config(pool, "demo.enabled", true).await {
        return Err(error_json(StatusCode::FORBIDDEN, "Demo is disabled"));
    }

    let runtime = resolve_runtime_config(pool).await;
    let api_key =
        credentials::load_credential(pool, &state.encryption_key, &runtime.credential_name)
            .await
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                error_json(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "No LLM credential configured for demo provider",
                )
            })?;

    let raw = request_provider_json(
        &state.http_client,
        &runtime,
        &api_key,
        &body.user_message,
        &body.memory_context,
    )
    .await
    .map_err(|message| error_json(StatusCode::BAD_GATEWAY, &message))?;

    let parsed: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|_| error_json(StatusCode::BAD_GATEWAY, "LLM returned invalid JSON"))?;

    let response = parsed
        .get("response")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| error_json(StatusCode::BAD_GATEWAY, "LLM response missing 'response'"))?;

    let new_memories = parsed
        .get("new_memories")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let referenced = parsed
        .get("referenced_memory_indices")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();

    Ok(Json(serde_json::json!({
        "response": response,
        "new_memories": new_memories,
        "referenced_memory_indices": referenced
    })))
}

async fn load_config_value(pool: &sqlx::PgPool, key: &str, default: &str) -> String {
    let row: Option<(serde_json::Value,)> =
        sqlx::query_as("SELECT value FROM system_config WHERE key = $1")
            .bind(key)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    match row {
        Some((value,)) => match value {
            serde_json::Value::String(s) => s,
            other => other.to_string().trim_matches('"').to_string(),
        },
        None => default.to_string(),
    }
}

async fn load_bool_config(pool: &sqlx::PgPool, key: &str, default: bool) -> bool {
    let fallback = if default { "true" } else { "false" };
    parse_bool(&load_config_value(pool, key, fallback).await).unwrap_or(default)
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn normalize_provider(provider: &str) -> String {
    match provider.trim().to_ascii_lowercase().as_str() {
        "open_ai" => "openai".to_string(),
        "google" => "gemini".to_string(),
        "openai" => "openai".to_string(),
        "anthropic" => "anthropic".to_string(),
        _ => "gemini".to_string(),
    }
}

async fn resolve_runtime_config(pool: &sqlx::PgPool) -> DemoRuntimeConfig {
    let demo_provider_override = load_config_value(pool, "demo.llm_provider", "").await;
    let provider_raw = if demo_provider_override.trim().is_empty() {
        load_config_value(pool, "llm.provider", "gemini").await
    } else {
        demo_provider_override
    };
    let provider = normalize_provider(&provider_raw);

    let (
        default_model,
        credential_name,
        provider_model_key,
        provider_api_url_key,
        provider_default_url,
    ) = match provider.as_str() {
        "openai" => (
            "gpt-4o-mini",
            "openai_api_key",
            "llm.openai_model",
            "llm.openai_api_url",
            DEFAULT_OPENAI_API_URL,
        ),
        "anthropic" => (
            "claude-3-haiku-20240307",
            "anthropic_api_key",
            "llm.anthropic_model",
            "llm.anthropic_api_url",
            DEFAULT_ANTHROPIC_API_URL,
        ),
        _ => (
            "gemini-2.0-flash",
            "gemini_api_key",
            "llm.gemini_model",
            "llm.gemini_api_url",
            DEFAULT_GEMINI_API_URL,
        ),
    };

    let demo_model_override = load_config_value(pool, "demo.llm_model", "").await;
    let model = if demo_model_override.trim().is_empty() {
        let configured = load_config_value(pool, provider_model_key, default_model).await;
        if configured.trim().is_empty() {
            default_model.to_string()
        } else {
            configured
        }
    } else {
        demo_model_override
    };

    let configured_api_url = load_config_value(pool, provider_api_url_key, "").await;
    let api_url = if configured_api_url.trim().is_empty() {
        provider_default_url.to_string()
    } else {
        configured_api_url
    };

    DemoRuntimeConfig {
        provider,
        model,
        api_url,
        credential_name: credential_name.to_string(),
    }
}

fn build_system_prompt(memory_context: &str) -> String {
    let context = if memory_context.trim().is_empty() {
        "(empty)"
    } else {
        memory_context
    };

    format!(
        r#"You are the AI inside "Knol", a context engineering platform for AI applications. You have access to a memory store of facts, preferences, events, and relationships about the user. Knol uses hybrid retrieval (vector + BM25 + knowledge graph) to surface the most relevant context.

CURRENT MEMORY STORE:
{context}

YOUR TASK:
1. Respond naturally and helpfully to the user's message
2. If they share new information, acknowledge you're storing it
3. ALWAYS reference specific stored memories when relevant (mention details from memory)
4. Extract any new memories from this message

Respond in this exact JSON format:
{{
  "response": "Your conversational response. Use **bold** for emphasis. Be warm, specific, and reference stored memories.",
  "new_memories": [
    {{
      "content": "A concise extracted memory",
      "type": "fact|preference|event|relationship|temporal_change|goal",
      "entities": [
        {{"name": "EntityName", "type": "person|organization|technology|concept|location"}}
      ]
    }}
  ],
  "referenced_memory_indices": [0, 2]
}}

RULES:
- Extract ALL meaningful facts, preferences, relationships, events from the user's message
- Entity names should be capitalized properly
- "referenced_memory_indices" should list indices of memories you referenced in your response
- If memory store is empty, welcome the user warmly and encourage them to share
- Be specific when referencing memories — quote details, don't be vague
- IMPORTANT: response must be valid JSON only, no markdown code fences"#,
    )
}

async fn request_provider_json(
    client: &reqwest::Client,
    runtime: &DemoRuntimeConfig,
    api_key: &str,
    user_message: &str,
    memory_context: &str,
) -> Result<String, String> {
    let system_prompt = build_system_prompt(memory_context);
    match runtime.provider.as_str() {
        "openai" => request_openai(client, runtime, api_key, &system_prompt, user_message).await,
        "anthropic" => {
            request_anthropic(client, runtime, api_key, &system_prompt, user_message).await
        }
        _ => request_gemini(client, runtime, api_key, &system_prompt, user_message).await,
    }
}

async fn request_gemini(
    client: &reqwest::Client,
    runtime: &DemoRuntimeConfig,
    api_key: &str,
    system_prompt: &str,
    user_message: &str,
) -> Result<String, String> {
    let url = format!(
        "{}/models/{}:generateContent?key={}",
        runtime.api_url.trim_end_matches('/'),
        runtime.model,
        api_key
    );

    let resp = client
        .post(url)
        .json(&serde_json::json!({
            "systemInstruction": { "parts": [{ "text": system_prompt }] },
            "contents": [{ "role": "user", "parts": [{ "text": user_message }] }],
            "generationConfig": {
                "temperature": 0.7,
                "maxOutputTokens": 1024,
                "responseMimeType": "application/json"
            }
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to call Gemini: {}", e))?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read Gemini response: {}", e))?;
    if !status.is_success() {
        return Err(format!("Gemini returned HTTP {}", status));
    }

    let payload: serde_json::Value =
        serde_json::from_str(&body).map_err(|_| "Gemini returned invalid JSON".to_string())?;
    payload
        .pointer("/candidates/0/content/parts/0/text")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| "Gemini response missing generated text".to_string())
}

async fn request_openai(
    client: &reqwest::Client,
    runtime: &DemoRuntimeConfig,
    api_key: &str,
    system_prompt: &str,
    user_message: &str,
) -> Result<String, String> {
    let url = format!("{}/chat/completions", runtime.api_url.trim_end_matches('/'));

    let resp = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "model": runtime.model,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_message }
            ],
            "temperature": 0.7,
            "max_tokens": 1024,
            "response_format": { "type": "json_object" }
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to call OpenAI: {}", e))?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read OpenAI response: {}", e))?;
    if !status.is_success() {
        return Err(format!("OpenAI returned HTTP {}", status));
    }

    let payload: serde_json::Value =
        serde_json::from_str(&body).map_err(|_| "OpenAI returned invalid JSON".to_string())?;
    payload
        .pointer("/choices/0/message/content")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| "OpenAI response missing message content".to_string())
}

async fn request_anthropic(
    client: &reqwest::Client,
    runtime: &DemoRuntimeConfig,
    api_key: &str,
    system_prompt: &str,
    user_message: &str,
) -> Result<String, String> {
    let url = if runtime.api_url.trim().is_empty() {
        DEFAULT_ANTHROPIC_API_URL.to_string()
    } else {
        runtime.api_url.clone()
    };

    let resp = client
        .post(url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&serde_json::json!({
            "model": runtime.model,
            "system": system_prompt,
            "messages": [{ "role": "user", "content": user_message }],
            "max_tokens": 1024,
            "temperature": 0.7
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to call Anthropic: {}", e))?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read Anthropic response: {}", e))?;
    if !status.is_success() {
        return Err(format!("Anthropic returned HTTP {}", status));
    }

    let payload: serde_json::Value =
        serde_json::from_str(&body).map_err(|_| "Anthropic returned invalid JSON".to_string())?;
    payload
        .pointer("/content/0/text")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| "Anthropic response missing text".to_string())
}

fn error_json(status: StatusCode, message: &str) -> DemoHttpError {
    (
        status,
        Json(serde_json::json!({
            "error": message
        })),
    )
}
