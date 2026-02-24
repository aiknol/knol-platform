// =============================================================================
// Signup Flow — End-to-End Tests
// =============================================================================
//
// Validates the complete lifecycle:
//   1. Signup (create tenant + owner user + initial API key)
//   2. Login with the created credentials
//   3. Use the initial API key to call the gateway
//   4. Create additional API keys via the tenant service
//   5. Use new API keys to write and search memories via the gateway
//   6. Verify role-based access (read_only key cannot write)
//   7. Revoke a key and confirm it stops working
//
// Requires:
//   - Tenant service running (TENANT_URL, default http://localhost:3002)
//   - Gateway service running (GATEWAY_URL, default http://localhost:8080)
//
// Run:
//   cargo test --manifest-path tests/e2e/Cargo.toml test_signup_flow
// =============================================================================

use crate::harness::*;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::env;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Tenant service URL
// ---------------------------------------------------------------------------

fn tenant_url() -> String {
    env::var("TENANT_URL").unwrap_or_else(|_| "http://localhost:3002".into())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn http() -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .cookie_store(true)
        .build()
        .expect("Failed to create HTTP client")
}

/// Generate a unique email for test isolation.
fn unique_email(prefix: &str) -> String {
    let suffix = Uuid::new_v4().to_string()[..8].to_string();
    format!("e2e-{}+{}@test.local", prefix, suffix)
}

/// Strong password that satisfies the signup validator
/// (≥12 chars, uppercase, lowercase, digit, special).
const TEST_PASSWORD: &str = "E2eTest!2026xyz";

/// Extract the `csrf_token` value from Set-Cookie response headers.
fn extract_csrf_from_response(resp: &reqwest::Response) -> Option<String> {
    for value in resp.headers().get_all(reqwest::header::SET_COOKIE) {
        if let Ok(s) = value.to_str() {
            // Format: csrf_token=<hex>; SameSite=Lax; Path=/; ...
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

/// POST to tenant service (unauthenticated). Returns (status, body, csrf_token).
async fn tenant_post(client: &Client, path: &str, body: &Value) -> (StatusCode, Value, Option<String>) {
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
async fn tenant_post_auth(client: &Client, csrf_token: &str, path: &str, body: &Value) -> (StatusCode, Value) {
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
async fn tenant_get_auth(client: &Client, path: &str) -> (StatusCode, Value) {
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

/// Authenticated DELETE to tenant service (via cookie + CSRF token).
async fn tenant_delete_auth(client: &Client, csrf_token: &str, path: &str) -> (StatusCode, Value) {
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
async fn gateway_post_with_key(api_key: &str, path: &str, body: &Value) -> (StatusCode, Value) {
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
async fn gateway_get_with_key(api_key: &str, path: &str) -> (StatusCode, Value) {
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

/// Signup a new tenant and return (client_with_cookies, api_key, csrf_token, response_json).
async fn signup_tenant(prefix: &str) -> (Client, String, String, Value) {
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

/// Login with email/password and return (client_with_cookies, csrf_token, response_json).
async fn login_tenant(email: &str, password: &str) -> (Client, String, Value) {
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

// ===========================================================================
// Tests
// ===========================================================================

// ── 1. Signup ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn signup_creates_tenant_and_returns_api_key() {
    let (_client, api_key, _csrf, body) = signup_tenant("signup").await;

    // Verify response structure
    assert!(body["token"].is_string(), "Should return session token");
    assert!(body["expires_at"].is_string(), "Should return expires_at");
    assert_eq!(body["user"]["role"].as_str().unwrap(), "owner");
    assert!(body["user"]["id"].is_string(), "User should have an id");
    assert!(body["user"]["tenant_id"].is_string(), "User should have tenant_id");
    assert!(body["tenant"]["id"].is_string(), "Tenant should have an id");
    assert_eq!(body["tenant"]["plan"].as_str().unwrap(), "free");

    // API key should have the knol_live_ prefix
    assert!(
        api_key.starts_with("knol_live_"),
        "API key should start with knol_live_, got: {}",
        &api_key[..20.min(api_key.len())]
    );
}

#[tokio::test]
async fn signup_rejects_duplicate_email() {
    let email = unique_email("dup");
    let payload = json!({
        "company_name": "Dup Co",
        "full_name": "Tester",
        "email": email,
        "password": TEST_PASSWORD,
    });

    let client = http();
    let (status, _, _) = tenant_post(&client, "/app/auth/signup", &payload).await;
    assert_eq!(status, StatusCode::OK, "First signup should succeed");

    // Second signup with same email should fail
    let (status, body, _) = tenant_post(&client, "/app/auth/signup", &payload).await;
    assert_eq!(status, StatusCode::CONFLICT, "Duplicate email should return 409");
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn signup_rejects_weak_password() {
    let payload = json!({
        "company_name": "Weak Co",
        "full_name": "Tester",
        "email": unique_email("weak"),
        "password": "short",
    });

    let client = http();
    let (status, body, _) = tenant_post(&client, "/app/auth/signup", &payload).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "Weak password should be rejected");
    assert!(body["error"].is_string());
}

// ── 2. Login ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn login_with_valid_credentials_succeeds() {
    let (_signup_client, _api_key, _csrf, signup_body) = signup_tenant("login-ok").await;
    let email = signup_body["user"]["email"].as_str().unwrap();

    let (_client, _csrf, login_body) = login_tenant(email, TEST_PASSWORD).await;
    assert!(login_body["token"].is_string());
    assert!(login_body["expires_at"].is_string());
    assert_eq!(login_body["user"]["email"].as_str().unwrap(), email);
}

#[tokio::test]
async fn login_with_wrong_password_fails() {
    let (_signup_client, _api_key, _csrf, signup_body) = signup_tenant("login-bad").await;
    let email = signup_body["user"]["email"].as_str().unwrap();

    let client = http();
    let payload = json!({ "email": email, "password": "WrongPassword!123" });
    let (status, _, _) = tenant_post(&client, "/app/auth/login", &payload).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_with_nonexistent_email_fails() {
    let client = http();
    let payload = json!({ "email": "nobody@nowhere.test", "password": TEST_PASSWORD });
    let (status, _, _) = tenant_post(&client, "/app/auth/login", &payload).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ── 3. /me endpoint ────────────────────────────────────────────────────────

#[tokio::test]
async fn me_returns_user_profile_after_signup() {
    let (client, _api_key, _csrf, signup_body) = signup_tenant("me").await;
    let email = signup_body["user"]["email"].as_str().unwrap();

    let (status, me_body) = tenant_get_auth(&client, "/app/auth/me").await;
    assert_eq!(status, StatusCode::OK, "GET /me failed: {:?}", me_body);
    assert_eq!(me_body["user"]["email"].as_str().unwrap(), email);
    assert!(me_body["tenant"]["id"].is_string());
    assert!(me_body["gateway_base_url"].is_string());
}

#[tokio::test]
async fn me_without_auth_returns_401() {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();
    let resp = client
        .get(format!("{}/app/auth/me", tenant_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ── 4. Initial API key works with gateway ──────────────────────────────────

#[tokio::test]
async fn initial_api_key_can_write_memory() {
    let (_client, api_key, _csrf, _body) = signup_tenant("gw-write").await;

    let memory = json!({
        "content": unique_content("e2e-initial-key"),
        "role": "user",
    });
    let (status, resp) = gateway_post_with_key(&api_key, "/v1/memory", &memory).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "Write via initial API key failed: {:?}",
        resp
    );
    assert_eq!(resp["status"].as_str().unwrap(), "accepted");
    assert!(resp["episode_id"].is_string(), "Should return episode_id");
}

#[tokio::test]
async fn initial_api_key_can_search_memory() {
    let (_client, api_key, _csrf, _body) = signup_tenant("gw-search").await;

    let search = json!({
        "query": "test search",
        "limit": 5,
    });
    let (status, resp) = gateway_post_with_key(&api_key, "/v1/memory/search", &search).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "Search via initial API key failed: {:?}",
        resp
    );
    assert!(resp["results"].is_array());
}

#[tokio::test]
async fn write_memory_with_uuid_user_id_succeeds() {
    let (_client, api_key, _csrf, _body) = signup_tenant("gw-uuid").await;
    let user_id = Uuid::new_v4();

    let memory = json!({
        "content": unique_content("e2e-uuid-user"),
        "user_id": user_id.to_string(),
    });
    let (status, resp) = gateway_post_with_key(&api_key, "/v1/memory", &memory).await;
    assert_eq!(status, StatusCode::OK, "Write with UUID user_id failed: {:?}", resp);
    assert_eq!(resp["status"].as_str().unwrap(), "accepted");
}

#[tokio::test]
async fn write_memory_with_invalid_user_id_fails() {
    let (_client, api_key, _csrf, _body) = signup_tenant("gw-bad-uid").await;

    let memory = json!({
        "content": "test content",
        "user_id": "not-a-uuid",
    });
    let (status, _resp) = gateway_post_with_key(&api_key, "/v1/memory", &memory).await;
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Non-UUID user_id should be rejected, got {}",
        status
    );
}

#[tokio::test]
async fn write_memory_with_metadata_succeeds() {
    let (_client, api_key, _csrf, _body) = signup_tenant("gw-meta").await;

    let memory = json!({
        "content": unique_content("e2e-metadata"),
        "role": "assistant",
        "session_id": "session-42",
        "agent_id": "agent-007",
        "metadata": {
            "source": "e2e-test",
            "tags": ["test", "integration"]
        }
    });
    let (status, resp) = gateway_post_with_key(&api_key, "/v1/memory", &memory).await;
    assert_eq!(status, StatusCode::OK, "Write with metadata failed: {:?}", resp);
}

// ── 5. API key management ──────────────────────────────────────────────────

#[tokio::test]
async fn create_developer_api_key_and_use_it() {
    let (client, _initial_key, csrf, _body) = signup_tenant("apikey-dev").await;

    // Create a new developer key
    let (status, created) = tenant_post_auth(
        &client,
        &csrf,
        "/app/api-keys",
        &json!({ "name": "dev-key", "role": "developer" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "Create API key failed: {:?}", created);

    let new_key = created["api_key"]
        .as_str()
        .expect("Should return api_key")
        .to_string();
    assert!(new_key.starts_with("knol_live_"));

    // Use the new key to write a memory via gateway
    let memory = json!({ "content": unique_content("e2e-dev-key") });
    let (status, resp) = gateway_post_with_key(&new_key, "/v1/memory", &memory).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "Write via new developer key failed: {:?}",
        resp
    );
}

#[tokio::test]
async fn create_read_only_api_key_cannot_write() {
    let (client, _initial_key, csrf, _body) = signup_tenant("apikey-ro").await;

    // Create a read_only key
    let (status, created) = tenant_post_auth(
        &client,
        &csrf,
        "/app/api-keys",
        &json!({ "name": "ro-key", "role": "read_only" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "Create API key failed: {:?}", created);

    let ro_key = created["api_key"].as_str().unwrap().to_string();

    // read_only key should NOT be able to write
    let memory = json!({ "content": "should not be written" });
    let (status, _resp) = gateway_post_with_key(&ro_key, "/v1/memory", &memory).await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "read_only key should not be able to write"
    );

    // But it SHOULD be able to search
    let search = json!({ "query": "test", "limit": 1 });
    let (status, _resp) = gateway_post_with_key(&ro_key, "/v1/memory/search", &search).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "read_only key should be able to search"
    );
}

#[tokio::test]
async fn list_api_keys_shows_created_keys() {
    let (client, _initial_key, csrf, _body) = signup_tenant("apikey-list").await;

    // Create two keys
    tenant_post_auth(
        &client,
        &csrf,
        "/app/api-keys",
        &json!({ "name": "key-alpha", "role": "developer" }),
    )
    .await;
    tenant_post_auth(
        &client,
        &csrf,
        "/app/api-keys",
        &json!({ "name": "key-beta", "role": "read_only" }),
    )
    .await;

    let (status, list) = tenant_get_auth(&client, "/app/api-keys").await;
    assert_eq!(status, StatusCode::OK, "List API keys failed: {:?}", list);

    let keys = list["data"].as_array().expect("Should return data array");
    // At least 3: initial key + 2 created keys
    assert!(
        keys.len() >= 3,
        "Expected at least 3 keys, got {}",
        keys.len()
    );

    let names: Vec<&str> = keys.iter().filter_map(|k| k["name"].as_str()).collect();
    assert!(names.contains(&"key-alpha"), "Should contain key-alpha");
    assert!(names.contains(&"key-beta"), "Should contain key-beta");
}

// ── 6. Key revocation ──────────────────────────────────────────────────────

#[tokio::test]
async fn revoked_api_key_stops_working() {
    let (client, _initial_key, csrf, _body) = signup_tenant("apikey-revoke").await;

    // Create a key
    let (status, created) = tenant_post_auth(
        &client,
        &csrf,
        "/app/api-keys",
        &json!({ "name": "to-revoke", "role": "developer" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let key_id = created["id"].as_str().unwrap();
    let api_key = created["api_key"].as_str().unwrap().to_string();

    // Verify the key works
    let memory = json!({ "content": unique_content("before-revoke") });
    let (status, _) = gateway_post_with_key(&api_key, "/v1/memory", &memory).await;
    assert_eq!(status, StatusCode::OK, "Key should work before revocation");

    // Revoke it
    let (status, _) = tenant_delete_auth(&client, &csrf, &format!("/app/api-keys/{}", key_id)).await;
    assert_eq!(status, StatusCode::OK, "Revoke should succeed");

    // Verify the key no longer works
    let memory = json!({ "content": "should be rejected" });
    let (status, _) = gateway_post_with_key(&api_key, "/v1/memory", &memory).await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "Revoked key should return 401"
    );
}

// ── 7. Invalid API key ─────────────────────────────────────────────────────

#[tokio::test]
async fn invalid_api_key_returns_401() {
    let (status, _) = gateway_post_with_key(
        "knol_live_invalid_key_that_does_not_exist",
        "/v1/memory",
        &json!({ "content": "test" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn missing_api_key_returns_401() {
    let resp = client()
        .post(format!("{}/v1/memory", gateway_url()))
        .json(&json!({ "content": "test" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ── 8. Tenant isolation ────────────────────────────────────────────────────

#[tokio::test]
async fn tenant_a_key_cannot_see_tenant_b_memories() {
    // Signup two separate tenants
    let (_client_a, key_a, _csrf_a, _) = signup_tenant("iso-a").await;
    let (_client_b, key_b, _csrf_b, _) = signup_tenant("iso-b").await;

    // Tenant A writes a memory
    let content = unique_content("tenant-a-only");
    let memory = json!({ "content": content });
    let (status, _) = gateway_post_with_key(&key_a, "/v1/memory", &memory).await;
    assert_eq!(status, StatusCode::OK);

    // Give the write pipeline a moment to process
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Tenant B searches — should NOT find Tenant A's memory
    let search = json!({ "query": content, "limit": 10 });
    let (status, resp) = gateway_post_with_key(&key_b, "/v1/memory/search", &search).await;
    assert_eq!(status, StatusCode::OK);

    let empty = vec![];
    let results = resp["results"].as_array().unwrap_or(&empty);
    for result in results {
        let found_content = result["memory"]["content"].as_str().unwrap_or("");
        assert_ne!(
            found_content, content,
            "Tenant B should not see Tenant A's memories"
        );
    }
}

// ── 9. Full lifecycle: signup → login → write → search ─────────────────────

#[tokio::test]
async fn full_lifecycle_signup_login_write_search() {
    // Step 1: Signup
    let (_signup_client, initial_key, _signup_csrf, signup_body) = signup_tenant("lifecycle").await;
    let email = signup_body["user"]["email"].as_str().unwrap().to_string();

    // Step 2: Login with the same credentials
    let (login_client, login_csrf, login_body) = login_tenant(&email, TEST_PASSWORD).await;
    assert!(login_body["token"].is_string());

    // Step 3: Create a dedicated developer key
    let (status, created) = tenant_post_auth(
        &login_client,
        &login_csrf,
        "/app/api-keys",
        &json!({ "name": "lifecycle-key", "role": "developer" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "Create key after login failed: {:?}", created);
    let dev_key = created["api_key"].as_str().unwrap().to_string();

    // Step 4: Write a memory with the developer key
    let content = unique_content("lifecycle-memory");
    let memory = json!({
        "content": content,
        "role": "user",
        "session_id": "lifecycle-session",
    });
    let (status, write_resp) = gateway_post_with_key(&dev_key, "/v1/memory", &memory).await;
    assert_eq!(status, StatusCode::OK, "Write failed: {:?}", write_resp);
    let episode_id = write_resp["episode_id"].as_str().unwrap();
    assert!(!episode_id.is_empty());

    // Step 5: Also verify the initial key still works
    let memory2 = json!({ "content": unique_content("lifecycle-initial-key") });
    let (status, _) = gateway_post_with_key(&initial_key, "/v1/memory", &memory2).await;
    assert_eq!(status, StatusCode::OK, "Initial key should still work");

    // Step 6: Search for the written memory
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let search = json!({ "query": content, "limit": 5 });
    let (status, search_resp) = gateway_post_with_key(&dev_key, "/v1/memory/search", &search).await;
    assert_eq!(status, StatusCode::OK, "Search failed: {:?}", search_resp);
    assert!(search_resp["results"].is_array());
}
