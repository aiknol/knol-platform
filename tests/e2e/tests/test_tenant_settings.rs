// =============================================================================
// Tenant Settings — End-to-End Tests
// =============================================================================
//
// Covers: get tenant, update tenant name, update profile,
//         data export (GDPR), audit logs.
// =============================================================================

use crate::tenant_helpers::*;
use reqwest::StatusCode;
use serde_json::json;

// ── Get Tenant ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_tenant_returns_details() {
    let (client, _api_key, _csrf, signup_body) = signup_tenant("ts-get").await;
    let tenant_id = signup_body["tenant"]["id"].as_str().unwrap();

    let (status, body) = tenant_get_auth(&client, "/app/tenant").await;
    assert_eq!(status, StatusCode::OK, "Get tenant failed: {:?}", body);
    assert_eq!(body["id"].as_str().unwrap(), tenant_id);
    assert!(body["name"].is_string());
    assert!(body["plan"].is_string());
}

// ── Update Tenant Settings ──────────────────────────────────────────────────

#[tokio::test]
async fn update_tenant_name() {
    let (client, _api_key, csrf, _body) = signup_tenant("ts-rename").await;

    let new_name = format!("Renamed Co {}", uuid::Uuid::new_v4().to_string()[..6].to_string());
    let (status, body) = tenant_put_auth(
        &client,
        &csrf,
        "/app/settings/tenant",
        &json!({ "name": new_name }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "Update tenant name failed: {:?}", body);
    assert_eq!(body["updated"].as_bool().unwrap(), true);
    assert_eq!(body["name"].as_str().unwrap(), new_name);

    // Verify the name changed
    let (status, tenant) = tenant_get_auth(&client, "/app/tenant").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(tenant["name"].as_str().unwrap(), new_name);
}

#[tokio::test]
async fn update_tenant_name_too_short_fails() {
    let (client, _api_key, csrf, _body) = signup_tenant("ts-short").await;

    let (status, _) = tenant_put_auth(
        &client,
        &csrf,
        "/app/settings/tenant",
        &json!({ "name": "X" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "Short name should be rejected");
}

#[tokio::test]
async fn non_owner_cannot_update_tenant() {
    let (client, _api_key, csrf, _body) = signup_tenant("ts-deny").await;

    // Create a developer user
    let dev_email = unique_email("ts-dev");
    tenant_post_auth(
        &client,
        &csrf,
        "/app/users",
        &json!({
            "full_name": "Developer",
            "email": dev_email,
            "password": TEST_PASSWORD,
            "role": "developer"
        }),
    )
    .await;

    let (dev_client, dev_csrf, _) = login_tenant(&dev_email, TEST_PASSWORD).await;

    let (status, _) = tenant_put_auth(
        &dev_client,
        &dev_csrf,
        "/app/settings/tenant",
        &json!({ "name": "Hacked Name" }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "Developer should not update tenant");
}

// ── Update Profile ──────────────────────────────────────────────────────────

#[tokio::test]
async fn update_own_profile_name() {
    let (client, _api_key, csrf, _body) = signup_tenant("ts-profile").await;

    let (status, body) = tenant_put_auth(
        &client,
        &csrf,
        "/app/settings/profile",
        &json!({ "full_name": "Updated Name" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "Update profile failed: {:?}", body);
    assert_eq!(body["updated"].as_bool().unwrap(), true);
    assert_eq!(body["full_name"].as_str().unwrap(), "Updated Name");

    // Verify via /me
    let (status, me) = tenant_get_auth(&client, "/app/auth/me").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me["user"]["full_name"].as_str().unwrap(), "Updated Name");
}

#[tokio::test]
async fn update_profile_too_short_name_fails() {
    let (client, _api_key, csrf, _body) = signup_tenant("ts-prof-short").await;

    let (status, _) = tenant_put_auth(
        &client,
        &csrf,
        "/app/settings/profile",
        &json!({ "full_name": "X" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn developer_can_update_own_profile() {
    let (client, _api_key, csrf, _body) = signup_tenant("ts-dev-prof").await;

    // Create developer
    let dev_email = unique_email("dev-profile");
    tenant_post_auth(
        &client,
        &csrf,
        "/app/users",
        &json!({
            "full_name": "Dev User",
            "email": dev_email,
            "password": TEST_PASSWORD,
            "role": "developer"
        }),
    )
    .await;

    let (dev_client, dev_csrf, _) = login_tenant(&dev_email, TEST_PASSWORD).await;

    let (status, body) = tenant_put_auth(
        &dev_client,
        &dev_csrf,
        "/app/settings/profile",
        &json!({ "full_name": "Dev Updated" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "Developer should update own profile: {:?}", body);
}

// ── Data Export (GDPR) ──────────────────────────────────────────────────────

#[tokio::test]
async fn data_export_returns_user_data() {
    let (client, _api_key, _csrf, signup_body) = signup_tenant("ts-export").await;
    let email = signup_body["user"]["email"].as_str().unwrap();

    let (status, body) = tenant_get_auth(&client, "/app/settings/data-export").await;
    assert_eq!(status, StatusCode::OK, "Data export failed: {:?}", body);
    assert!(body["export_date"].is_string());
    assert!(body["user"].is_object());
    assert_eq!(body["user"]["email"].as_str().unwrap(), email);
    assert!(body["sessions"].is_array());
    assert!(body["audit_log"].is_array());
}

// ── Audit Logs ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn audit_log_records_signup() {
    let (client, _api_key, _csrf, _body) = signup_tenant("ts-audit").await;

    let (status, body) = tenant_get_auth(&client, "/app/audit").await;
    assert_eq!(status, StatusCode::OK, "Audit log failed: {:?}", body);

    let entries = body["data"].as_array().expect("Should return data array");
    assert!(!entries.is_empty(), "Should have audit entries after signup");

    // Check structure
    let entry = &entries[0];
    assert!(entry["action"].is_string());
    assert!(entry["resource_type"].is_string());
    assert!(entry["created_at"].is_string());

    // Should have a signup action somewhere
    let has_signup = entries.iter().any(|e| e["action"].as_str().unwrap_or("") == "signup");
    assert!(has_signup, "Should have a signup audit entry");
}

#[tokio::test]
async fn audit_log_records_api_key_creation() {
    let (client, _api_key, csrf, _body) = signup_tenant("ts-audit-key").await;

    // Create an API key (should generate audit entry)
    tenant_post_auth(
        &client,
        &csrf,
        "/app/api-keys",
        &json!({ "name": "audited-key", "role": "developer" }),
    )
    .await;

    let (status, body) = tenant_get_auth(&client, "/app/audit").await;
    assert_eq!(status, StatusCode::OK);

    let entries = body["data"].as_array().unwrap();
    let has_create = entries
        .iter()
        .any(|e| e["action"].as_str().unwrap_or("") == "create" && e["resource_type"].as_str().unwrap_or("") == "api_key");
    assert!(has_create, "Should have api_key create audit entry");
}

#[tokio::test]
async fn non_owner_cannot_view_audit_logs() {
    let (client, _api_key, csrf, _body) = signup_tenant("ts-audit-deny").await;

    let dev_email = unique_email("audit-dev");
    tenant_post_auth(
        &client,
        &csrf,
        "/app/users",
        &json!({
            "full_name": "Developer",
            "email": dev_email,
            "password": TEST_PASSWORD,
            "role": "developer"
        }),
    )
    .await;

    let (dev_client, _dev_csrf, _) = login_tenant(&dev_email, TEST_PASSWORD).await;

    let (status, _) = tenant_get_auth(&dev_client, "/app/audit").await;
    assert_eq!(status, StatusCode::FORBIDDEN, "Developer should not see audit logs");
}
