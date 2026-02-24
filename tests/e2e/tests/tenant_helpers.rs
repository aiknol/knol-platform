// =============================================================================
// Shared helpers for tenant-service e2e tests
// =============================================================================
//
// Provides cookie-based HTTP client, CSRF extraction, signup/login helpers,
// and authenticated request builders that other test modules can reuse.
// =============================================================================

use crate::harness::*;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::env;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

pub fn tenant_url() -> String {
    env::var("TENANT_URL").unwrap_or_else(|_| "http://localhost:3002".into())
}

/// Strong password that satisfies the signup validator
/// (≥12 chars, uppercase, lowercase, digit, special).
pub const TEST_PASSWORD: &str = "E2eTest!2026xyz";

// ---------------------------------------------------------------------------
// HTTP client with cookie jar
// ---------------------------------------------------------------------------

pub fn http() -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .cookie_store(true)
        .build()
        .expect("Failed to create HTTP client")
}

/// Generate a unique email for test isolation.
pub fn unique_email(prefix: &str) -> String {
    let suffix = Uuid::new_v4().to_string()[..8].to_string();
    format!("e2e-{}+{}@test.local", prefix, suffix)
}

// ---------------------------------------------------------------------------
// CSRF extraction
// ---------------------------------------------------------------------------

/// Extract the `csrf_token` value from Set-Cookie response headers.
pub fn extract_csrf_from_response(resp: &reqwest::Response) -> Option<String> {
    for value in resp.headers().get_all(reqwest::header::SET_COOKIE) {
        if let Ok(s) = value.to_str() {
            if let Some(rest) = s.strip_prefix("csrf_token=") {
                if let Some(token) = rest.split(';').next() {
                    if !token.is_empty() {
                        return Some(token.to_string());
                    }
                }
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Request helpers
// ---------------------------------------------------------------------------

/// POST to tenant service (unauthenticated). Returns (status, body, csrf_token).
pub async fn tenant_post(client: &Client, path: &str, body: &Value) -> (StatusCode, Value, Option<String>) {
    let resp = client
        .post(format!("{}{}", tenant_url(), path))
        .json(body)
        .send()
        .await
        .expect("Tenant POST failed");
    let status = resp.status();
    let csrf = extract_csrf_from_response(&resp);
    let text = resp.text().await.unwrap_or_default();
    let json: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    (status, json, csrf)
}

/// Authenticated POST to tenant service (via cookie + CSRF token).
pub async fn tenant_post_auth(client: &Client, csrf_token: &str, path: &str, body: &Value) -> (StatusCode, Value) {
    let resp = client
        .post(format!("{}{}", tenant_url(), path))
        .header("x-csrf-token", csrf_token)
        .json(body)
        .send()
        .await
        .expect("Tenant auth POST failed");
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    let json: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    (status, json)
}

/// Authenticated GET to tenant service (via cookie set during login).
pub async fn tenant_get_auth(client: &Client, path: &str) -> (StatusCode, Value) {
    let resp = client
        .get(format!("{}{}", tenant_url(), path))
        .send()
        .await
        .expect("Tenant auth GET failed");
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    let json: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    (status, json)
}

/// Authenticated PUT to tenant service (via cookie + CSRF token).
pub async fn tenant_put_auth(client: &Client, csrf_token: &str, path: &str, body: &Value) -> (StatusCode, Value) {
    let resp = client
        .put(format!("{}{}", tenant_url(), path))
        .header("x-csrf-token", csrf_token)
        .json(body)
        .send()
        .await
        .expect("Tenant auth PUT failed");
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    let json: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    (status, json)
}

/// Authenticated DELETE to tenant service (via cookie + CSRF token).
pub async fn tenant_delete_auth(client: &Client, csrf_token: &str, path: &str) -> (StatusCode, Value) {
    let resp = client
        .delete(format!("{}{}", tenant_url(), path))
        .header("x-csrf-token", csrf_token)
        .send()
        .await
        .expect("Tenant auth DELETE failed");
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    let json: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    (status, json)
}

/// POST to gateway with a specific API key.
pub async fn gateway_post_with_key(api_key: &str, path: &str, body: &Value) -> (StatusCode, Value) {
    let resp = client()
        .post(format!("{}{}", gateway_url(), path))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(body)
        .send()
        .await
        .expect("Gateway POST failed");
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    let json: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    (status, json)
}

/// GET from gateway with a specific API key.
pub async fn gateway_get_with_key(api_key: &str, path: &str) -> (StatusCode, Value) {
    let resp = client()
        .get(format!("{}{}", gateway_url(), path))
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .expect("Gateway GET failed");
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    let json: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    (status, json)
}

/// PUT to gateway with a specific API key.
pub async fn gateway_put_with_key(api_key: &str, path: &str, body: &Value) -> (StatusCode, Value) {
    let resp = client()
        .put(format!("{}{}", gateway_url(), path))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(body)
        .send()
        .await
        .expect("Gateway PUT failed");
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    let json: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    (status, json)
}

/// DELETE from gateway with a specific API key.
pub async fn gateway_delete_with_key(api_key: &str, path: &str) -> (StatusCode, Value) {
    let resp = client()
        .delete(format!("{}{}", gateway_url(), path))
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .expect("Gateway DELETE failed");
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    let json: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    (status, json)
}

// ---------------------------------------------------------------------------
// High-level helpers
// ---------------------------------------------------------------------------

/// Signup a new tenant. Returns (client_with_cookies, api_key, csrf_token, response_json).
pub async fn signup_tenant(prefix: &str) -> (Client, String, String, Value) {
    let client = http();
    let email = unique_email(prefix);
    let payload = json!({
        "company_name": format!("E2E Test Co {}", prefix),
        "full_name": "E2E Tester",
        "email": email,
        "password": TEST_PASSWORD,
    });
    let (status, body, csrf) = tenant_post(&client, "/app/auth/signup", &payload).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "Signup failed: {}",
        serde_json::to_string_pretty(&body).unwrap_or_default()
    );
    let api_key = body["initial_api_key"]
        .as_str()
        .expect("Signup should return initial_api_key")
        .to_string();
    let csrf_token = csrf.expect("Signup should return csrf_token cookie");
    (client, api_key, csrf_token, body)
}

/// Login with email/password. Returns (client_with_cookies, csrf_token, response_json).
pub async fn login_tenant(email: &str, password: &str) -> (Client, String, Value) {
    let client = http();
    let payload = json!({ "email": email, "password": password });
    let (status, body, csrf) = tenant_post(&client, "/app/auth/login", &payload).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "Login failed: {}",
        serde_json::to_string_pretty(&body).unwrap_or_default()
    );
    let csrf_token = csrf.expect("Login should return csrf_token cookie");
    (client, csrf_token, body)
}
