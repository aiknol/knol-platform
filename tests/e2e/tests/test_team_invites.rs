// =============================================================================
// Team Invites — End-to-End Tests
// =============================================================================
//
// Covers: create invite, list invites, revoke invite, accept invite,
//         invite validation and error cases.
// =============================================================================

use crate::tenant_helpers::*;
use reqwest::StatusCode;
use serde_json::json;

// ── Create Invite ───────────────────────────────────────────────────────────

#[tokio::test]
async fn create_invite_returns_token() {
    let (client, _api_key, csrf, _body) = signup_tenant("inv-create").await;

    let invite_email = unique_email("invited");
    let (status, body) = tenant_post_auth(
        &client,
        &csrf,
        "/app/invites",
        &json!({ "email": invite_email, "role": "developer" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "Create invite failed: {:?}", body);
    assert!(body["id"].is_string());
    assert_eq!(body["email"].as_str().unwrap(), invite_email);
    assert!(body["token"].is_string(), "Should return invite token");
    assert!(body["expires_at"].is_string());
}

#[tokio::test]
async fn create_invite_with_viewer_role() {
    let (client, _api_key, csrf, _body) = signup_tenant("inv-viewer").await;

    let (status, body) = tenant_post_auth(
        &client,
        &csrf,
        "/app/invites",
        &json!({ "email": unique_email("viewer"), "role": "viewer" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "Create viewer invite failed: {:?}", body);
}

#[tokio::test]
async fn non_owner_cannot_create_invite() {
    let (client, _api_key, csrf, _body) = signup_tenant("inv-deny").await;

    // Create a developer user
    let dev_email = unique_email("dev-inv");
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

    // Login as developer
    let (dev_client, dev_csrf, _) = login_tenant(&dev_email, TEST_PASSWORD).await;

    let (status, _) = tenant_post_auth(
        &dev_client,
        &dev_csrf,
        "/app/invites",
        &json!({ "email": unique_email("nope"), "role": "developer" }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ── List Invites ────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_invites_shows_pending() {
    let (client, _api_key, csrf, _body) = signup_tenant("inv-list").await;

    // Create two invites
    tenant_post_auth(
        &client,
        &csrf,
        "/app/invites",
        &json!({ "email": unique_email("list-a"), "role": "developer" }),
    )
    .await;
    tenant_post_auth(
        &client,
        &csrf,
        "/app/invites",
        &json!({ "email": unique_email("list-b"), "role": "admin" }),
    )
    .await;

    let (status, body) = tenant_get_auth(&client, "/app/invites").await;
    assert_eq!(status, StatusCode::OK, "List invites failed: {:?}", body);

    let invites = body["data"].as_array().expect("Should return data array");
    assert!(invites.len() >= 2, "Should have at least 2 invites");

    // Verify structure
    let invite = &invites[0];
    assert!(invite["id"].is_string());
    assert!(invite["email"].is_string());
    assert!(invite["status"].is_string());
}

// ── Revoke Invite ───────────────────────────────────────────────────────────

#[tokio::test]
async fn revoke_pending_invite() {
    let (client, _api_key, csrf, _body) = signup_tenant("inv-revoke").await;

    let (status, invite) = tenant_post_auth(
        &client,
        &csrf,
        "/app/invites",
        &json!({ "email": unique_email("revoke-me"), "role": "developer" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let invite_id = invite["id"].as_str().unwrap();

    // Revoke
    let (status, body) = tenant_delete_auth(
        &client,
        &csrf,
        &format!("/app/invites/{}", invite_id),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "Revoke invite failed: {:?}", body);
    assert_eq!(body["revoked"].as_bool().unwrap(), true);
}

// ── Accept Invite ───────────────────────────────────────────────────────────

#[tokio::test]
async fn accept_invite_creates_user_and_session() {
    let (client, _api_key, csrf, _body) = signup_tenant("inv-accept").await;

    // Create an invite
    let invite_email = unique_email("accept-me");
    let (status, invite) = tenant_post_auth(
        &client,
        &csrf,
        "/app/invites",
        &json!({ "email": invite_email, "role": "developer" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let invite_token = invite["token"].as_str().unwrap();

    // Accept the invite (public endpoint, no auth needed)
    let accept_client = http();
    let (status, body, _csrf) = tenant_post(
        &accept_client,
        "/app/auth/accept-invite",
        &json!({
            "token": invite_token,
            "full_name": "Invited User",
            "password": TEST_PASSWORD
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "Accept invite failed: {:?}", body);
    assert!(body["token"].is_string(), "Should return session token");
    assert_eq!(body["user"]["email"].as_str().unwrap(), invite_email);
    assert_eq!(body["user"]["role"].as_str().unwrap(), "developer");
}

#[tokio::test]
async fn accept_invite_with_revoked_token_fails() {
    let (client, _api_key, csrf, _body) = signup_tenant("inv-rev-accept").await;

    // Create and revoke an invite
    let (status, invite) = tenant_post_auth(
        &client,
        &csrf,
        "/app/invites",
        &json!({ "email": unique_email("revoked"), "role": "developer" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let invite_token = invite["token"].as_str().unwrap();
    let invite_id = invite["id"].as_str().unwrap();

    // Revoke the invite
    tenant_delete_auth(&client, &csrf, &format!("/app/invites/{}", invite_id)).await;

    // Try to accept the revoked invite
    let accept_client = http();
    let (status, _, _) = tenant_post(
        &accept_client,
        "/app/auth/accept-invite",
        &json!({
            "token": invite_token,
            "full_name": "Should Fail",
            "password": TEST_PASSWORD
        }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "Revoked invite should not be accepted");
}

#[tokio::test]
async fn accepted_user_joins_correct_tenant() {
    let (owner_client, _api_key, csrf, owner_body) = signup_tenant("inv-tenant").await;
    let owner_tenant_id = owner_body["tenant"]["id"].as_str().unwrap();

    // Create invite
    let invite_email = unique_email("join-tenant");
    let (_, invite) = tenant_post_auth(
        &owner_client,
        &csrf,
        "/app/invites",
        &json!({ "email": invite_email, "role": "developer" }),
    )
    .await;
    let invite_token = invite["token"].as_str().unwrap();

    // Accept
    let accept_client = http();
    let (status, body, _) = tenant_post(
        &accept_client,
        "/app/auth/accept-invite",
        &json!({
            "token": invite_token,
            "full_name": "Joined User",
            "password": TEST_PASSWORD
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["user"]["tenant_id"].as_str().unwrap(),
        owner_tenant_id,
        "Invited user should join the inviting tenant"
    );
}
