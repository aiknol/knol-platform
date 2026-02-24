// =============================================================================
// Webhooks — End-to-End Tests
// =============================================================================
//
// Covers: create webhook, list webhooks, delete webhook,
//         role-based access (admin-only for create/delete).
// =============================================================================

use crate::tenant_helpers::*;
use reqwest::StatusCode;
use serde_json::json;

// ── Create Webhook ──────────────────────────────────────────────────────────

#[tokio::test]
async fn create_webhook_with_admin_key() {
    let (client, _initial_key, csrf, _body) = signup_tenant("wh-create").await;

    // Create an admin API key
    let (status, created) = tenant_post_auth(
        &client,
        &csrf,
        "/app/api-keys",
        &json!({ "name": "admin-wh", "role": "admin" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let admin_key = created["api_key"].as_str().unwrap();

    // Create a webhook (using a public HTTPS URL to pass SSRF validation)
    let (status, resp) = gateway_post_with_key(
        admin_key,
        "/v1/webhooks",
        &json!({
            "url": "https://example.com/webhook",
            "event_types": ["memory.created"],
            "description": "E2E test webhook"
        }),
    )
    .await;
    assert!(
        status == StatusCode::CREATED || status == StatusCode::OK,
        "Create webhook failed with {}: {:?}",
        status,
        resp
    );
    assert!(resp["id"].is_string(), "Should return webhook id");
    assert_eq!(resp["url"].as_str().unwrap(), "https://example.com/webhook");
}

#[tokio::test]
async fn create_webhook_with_wildcard_events() {
    let (client, _initial_key, csrf, _body) = signup_tenant("wh-wildcard").await;

    let (_, created) = tenant_post_auth(
        &client,
        &csrf,
        "/app/api-keys",
        &json!({ "name": "admin-wh2", "role": "admin" }),
    )
    .await;
    let admin_key = created["api_key"].as_str().unwrap();

    let (status, resp) = gateway_post_with_key(
        admin_key,
        "/v1/webhooks",
        &json!({
            "url": "https://example.com/all-events",
            "event_types": ["*"]
        }),
    )
    .await;
    assert!(
        status == StatusCode::CREATED || status == StatusCode::OK,
        "Create wildcard webhook failed: {:?}",
        resp
    );
}

// ── Non-admin cannot create webhook ─────────────────────────────────────────

#[tokio::test]
async fn developer_cannot_create_webhook() {
    let (client, _initial_key, csrf, _body) = signup_tenant("wh-deny").await;

    let (_, created) = tenant_post_auth(
        &client,
        &csrf,
        "/app/api-keys",
        &json!({ "name": "dev-wh", "role": "developer" }),
    )
    .await;
    let dev_key = created["api_key"].as_str().unwrap();

    let (status, _) = gateway_post_with_key(
        dev_key,
        "/v1/webhooks",
        &json!({ "url": "https://example.com/nope", "event_types": ["*"] }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "Developer should not create webhooks");
}

// ── List Webhooks ───────────────────────────────────────────────────────────

#[tokio::test]
async fn list_webhooks_shows_created() {
    let (client, _initial_key, csrf, _body) = signup_tenant("wh-list").await;

    let (_, created) = tenant_post_auth(
        &client,
        &csrf,
        "/app/api-keys",
        &json!({ "name": "admin-list", "role": "admin" }),
    )
    .await;
    let admin_key = created["api_key"].as_str().unwrap();

    // Create a webhook
    gateway_post_with_key(
        admin_key,
        "/v1/webhooks",
        &json!({
            "url": "https://example.com/listed",
            "event_types": ["memory.created"]
        }),
    )
    .await;

    // List webhooks
    let (status, resp) = gateway_get_with_key(admin_key, "/v1/webhooks").await;
    assert_eq!(status, StatusCode::OK, "List webhooks failed: {:?}", resp);

    let webhooks = resp.as_array().expect("Should return array");
    assert!(!webhooks.is_empty(), "Should have at least one webhook");

    // Secrets should be masked
    for wh in webhooks {
        if wh["secret"].is_string() {
            assert_eq!(wh["secret"].as_str().unwrap(), "****");
        }
    }
}

#[tokio::test]
async fn read_only_can_list_webhooks() {
    let (client, _initial_key, csrf, _body) = signup_tenant("wh-ro-list").await;

    let (_, created) = tenant_post_auth(
        &client,
        &csrf,
        "/app/api-keys",
        &json!({ "name": "ro-wh", "role": "read_only" }),
    )
    .await;
    let ro_key = created["api_key"].as_str().unwrap();

    let (status, _) = gateway_get_with_key(ro_key, "/v1/webhooks").await;
    assert_eq!(status, StatusCode::OK, "read_only should list webhooks");
}

// ── Delete Webhook ──────────────────────────────────────────────────────────

#[tokio::test]
async fn delete_webhook_with_admin_key() {
    let (client, _initial_key, csrf, _body) = signup_tenant("wh-delete").await;

    let (_, created) = tenant_post_auth(
        &client,
        &csrf,
        "/app/api-keys",
        &json!({ "name": "admin-del", "role": "admin" }),
    )
    .await;
    let admin_key = created["api_key"].as_str().unwrap();

    // Create a webhook
    let (status, wh) = gateway_post_with_key(
        admin_key,
        "/v1/webhooks",
        &json!({
            "url": "https://example.com/to-delete",
            "event_types": ["memory.deleted"]
        }),
    )
    .await;
    assert!(status == StatusCode::CREATED || status == StatusCode::OK);
    let wh_id = wh["id"].as_str().unwrap();

    // Delete it
    let (status, _) = gateway_delete_with_key(admin_key, &format!("/v1/webhooks/{}", wh_id)).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "Delete webhook should return 204");
}

#[tokio::test]
async fn developer_cannot_delete_webhook() {
    let (client, _initial_key, csrf, _body) = signup_tenant("wh-del-deny").await;

    // Create admin key and developer key
    let (_, admin_created) = tenant_post_auth(
        &client,
        &csrf,
        "/app/api-keys",
        &json!({ "name": "admin-wh", "role": "admin" }),
    )
    .await;
    let admin_key = admin_created["api_key"].as_str().unwrap();

    let (_, dev_created) = tenant_post_auth(
        &client,
        &csrf,
        "/app/api-keys",
        &json!({ "name": "dev-wh", "role": "developer" }),
    )
    .await;
    let dev_key = dev_created["api_key"].as_str().unwrap();

    // Admin creates a webhook
    let (_, wh) = gateway_post_with_key(
        admin_key,
        "/v1/webhooks",
        &json!({
            "url": "https://example.com/protected",
            "event_types": ["*"]
        }),
    )
    .await;
    let wh_id = wh["id"].as_str().unwrap();

    // Developer cannot delete it
    let (status, _) = gateway_delete_with_key(dev_key, &format!("/v1/webhooks/{}", wh_id)).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "Developer should not delete webhooks");
}

// ── SSRF validation ─────────────────────────────────────────────────────────

#[tokio::test]
async fn webhook_rejects_localhost_url() {
    let (client, _initial_key, csrf, _body) = signup_tenant("wh-ssrf").await;

    let (_, created) = tenant_post_auth(
        &client,
        &csrf,
        "/app/api-keys",
        &json!({ "name": "admin-ssrf", "role": "admin" }),
    )
    .await;
    let admin_key = created["api_key"].as_str().unwrap();

    let (status, _) = gateway_post_with_key(
        admin_key,
        "/v1/webhooks",
        &json!({
            "url": "http://localhost:9999/evil",
            "event_types": ["*"]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "localhost URL should be rejected");
}
