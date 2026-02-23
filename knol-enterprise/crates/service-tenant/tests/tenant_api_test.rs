//! Integration tests for the tenant service API.
//!
//! These tests exercise every endpoint through the real HTTP router
//! against a live PostgreSQL database.  No mocks — the test builds the
//! full `axum::Router` and uses `tower::ServiceExt::oneshot()` to send
//! requests in-process.
//!
//! Requires:
//!   DATABASE_URL=postgresql://memory:memory_dev@localhost:5432/memory
//!
//! Run with:
//!   cargo test -p service-tenant --test tenant_api_test -- --test-threads=1

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use service_tenant::{auth, routes, TenantAppState};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

// ── TestApp helper ──────────────────────────────────────────────────────

struct TestApp {
    router: Router,
}

/// Valid strong password for test accounts.
const TEST_PASSWORD: &str = "Test1234!@#$";
/// Secondary strong password for password-change tests.
const NEW_PASSWORD: &str = "NewPass567!@#";

impl TestApp {
    async fn new() -> Self {
        // Set env for dev cookies (no Secure flag).
        std::env::set_var("ADMIN_SECURE_COOKIES", "false");

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://memory:memory_dev@localhost:5432/memory".into());

        let db_pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let state = Arc::new(TenantAppState {
            db_pool,
            jwt_secret: "test-jwt-secret-must-be-32-chars-long!!".to_string(),
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
            rate_limiter: enterprise_common::rate_limit::new_rate_limiter(),
            api_rate_limiter: enterprise_common::api_rate_limit::ApiRateLimiter::new(),
            stripe_secret_key: None,
            stripe_webhook_secret: None,
            idle_timeout_mins: 0, // disabled for most tests
            totp_encryption_key: None,
            secure_cookies: false,
        });

        // Build router identically to main.rs but without CORS (not needed for oneshot).
        let app_protected = Router::new()
            .route("/auth/me", get(routes::app::me))
            .route("/auth/logout", post(routes::app::logout))
            .route(
                "/auth/password-reset",
                post(routes::app::initiate_password_reset),
            )
            .route("/auth/sessions", get(routes::app::list_sessions))
            .route(
                "/auth/sessions/:id",
                delete(routes::app::revoke_session),
            )
            .route("/tenant", get(routes::app::tenant))
            .route("/api-keys", get(routes::app::list_api_keys))
            .route("/api-keys", post(routes::app::create_api_key))
            .route("/api-keys/:id", delete(routes::app::revoke_api_key))
            .route("/users", get(routes::app::list_users))
            .route("/users", post(routes::app::create_user))
            .route("/users/:id", put(routes::app::update_user))
            .route("/audit", get(routes::app::list_audit_logs))
            // Billing
            .route("/billing/checkout", post(routes::billing::create_checkout))
            .route("/billing/portal", post(routes::billing::create_portal))
            .route(
                "/billing/subscription",
                get(routes::billing::get_subscription),
            )
            .route(
                "/billing/cancel",
                post(routes::billing::cancel_subscription),
            )
            .route(
                "/billing/reactivate",
                post(routes::billing::reactivate_subscription),
            )
            .route("/billing/invoices", get(routes::billing::list_invoices))
            .route(
                "/billing/invoices/upcoming",
                get(routes::billing::upcoming_invoice),
            )
            .route("/billing/usage", get(routes::billing::get_usage))
            .route(
                "/billing/usage/history",
                get(routes::billing::get_usage_history),
            )
            // Team invites
            .route("/invites", post(routes::invites::create_invite))
            .route("/invites", get(routes::invites::list_invites))
            .route("/invites/:id", delete(routes::invites::revoke_invite))
            // Settings
            .route(
                "/settings/tenant",
                put(routes::settings::update_tenant_settings),
            )
            .route("/settings/profile", put(routes::settings::update_profile))
            .route(
                "/settings/change-password",
                post(routes::settings::change_password),
            )
            // Token refresh
            .route("/auth/refresh", post(routes::app::refresh_token))
            // Email verification (admin-initiated)
            .route("/auth/verify-email", post(routes::app::verify_email))
            // TOTP 2FA management
            .route("/settings/totp/setup", post(routes::totp::setup_totp))
            .route("/settings/totp/enable", post(routes::totp::enable_totp))
            .route("/settings/totp/disable", post(routes::totp::disable_totp))
            // GDPR data export & account deletion
            .route("/settings/data-export", get(routes::settings::data_export))
            .route(
                "/settings/delete-account",
                post(routes::settings::delete_account),
            )
            .route_layer(middleware::from_fn_with_state(
                state.clone(),
                auth::app_auth_middleware,
            ));

        let app_routes = Router::new()
            .route("/auth/signup", post(routes::app::signup))
            .route("/auth/login", post(routes::app::login))
            .route(
                "/auth/reset-password",
                post(routes::app::reset_password),
            )
            .route("/auth/accept-invite", post(routes::invites::accept_invite))
            .route("/auth/totp/verify", post(routes::totp::verify_totp))
            .merge(app_protected);

        let router = Router::new()
            .nest("/app", app_routes)
            .layer(middleware::from_fn(
                enterprise_common::request_id::request_id_middleware,
            ))
            .with_state(state);

        TestApp { router }
    }

    /// Send a request and return (status, body JSON).
    /// Automatically attaches CSRF tokens for authenticated mutating requests.
    async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
        token: Option<&str>,
    ) -> (StatusCode, Value) {
        let mut builder = Request::builder().method(&method).uri(path);
        builder = builder.header("content-type", "application/json");
        if let Some(t) = token {
            builder = builder.header("authorization", format!("Bearer {}", t));
        }

        // Attach CSRF tokens for authenticated POST/PUT/DELETE requests
        let needs_csrf = token.is_some()
            && matches!(method, Method::POST | Method::PUT | Method::DELETE);
        if needs_csrf {
            let csrf = "test-csrf-token-for-integration-tests";
            builder = builder.header("x-csrf-token", csrf);
            builder = builder.header("cookie", format!("csrf_token={}", csrf));
        }

        let body = match body {
            Some(v) => Body::from(serde_json::to_vec(&v).unwrap()),
            None => Body::empty(),
        };

        let req = builder.body(body).unwrap();
        let response = self
            .router
            .clone()
            .oneshot(req)
            .await
            .expect("oneshot failed");

        let status = response.status();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("failed to collect body")
            .to_bytes();

        let json: Value = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap_or(Value::Null)
        };
        (status, json)
    }

    /// Send a request and return (status, body JSON, response headers).
    /// Useful for inspecting response headers like X-Request-ID.
    async fn request_with_headers(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
        token: Option<&str>,
    ) -> (StatusCode, Value, axum::http::HeaderMap) {
        let mut builder = Request::builder().method(&method).uri(path);
        builder = builder.header("content-type", "application/json");
        if let Some(t) = token {
            builder = builder.header("authorization", format!("Bearer {}", t));
        }

        let needs_csrf = token.is_some()
            && matches!(method, Method::POST | Method::PUT | Method::DELETE);
        if needs_csrf {
            let csrf = "test-csrf-token-for-integration-tests";
            builder = builder.header("x-csrf-token", csrf);
            builder = builder.header("cookie", format!("csrf_token={}", csrf));
        }

        let body = match body {
            Some(v) => Body::from(serde_json::to_vec(&v).unwrap()),
            None => Body::empty(),
        };

        let req = builder.body(body).unwrap();
        let response = self
            .router
            .clone()
            .oneshot(req)
            .await
            .expect("oneshot failed");

        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("failed to collect body")
            .to_bytes();

        let json: Value = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap_or(Value::Null)
        };
        (status, json, headers)
    }

    /// Send a request WITHOUT CSRF tokens (for testing CSRF enforcement).
    async fn request_no_csrf(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
        token: Option<&str>,
    ) -> (StatusCode, Value) {
        let mut builder = Request::builder().method(method).uri(path);
        builder = builder.header("content-type", "application/json");
        if let Some(t) = token {
            builder = builder.header("authorization", format!("Bearer {}", t));
        }

        let body = match body {
            Some(v) => Body::from(serde_json::to_vec(&v).unwrap()),
            None => Body::empty(),
        };

        let req = builder.body(body).unwrap();
        let response = self
            .router
            .clone()
            .oneshot(req)
            .await
            .expect("oneshot failed");

        let status = response.status();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("failed to collect body")
            .to_bytes();

        let json: Value = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap_or(Value::Null)
        };
        (status, json)
    }

    /// Signup a fresh tenant and return (token, response_json).
    async fn signup(&self, suffix: &str) -> (String, Value) {
        let id = Uuid::new_v4().to_string();
        let email = format!("owner-{}+{}@test.local", suffix, id);
        let company = format!("TestCo-{}-{}", suffix, &id[..8]);

        let (status, body) = self
            .request(
                Method::POST,
                "/app/auth/signup",
                Some(json!({
                    "company_name": company,
                    "full_name": "Test Owner",
                    "email": email,
                    "password": TEST_PASSWORD,
                })),
                None,
            )
            .await;
        assert_eq!(status, StatusCode::OK, "signup failed: {:?}", body);
        let token = body["token"].as_str().expect("no token").to_string();
        (token, body)
    }

    /// Login with email/password, return token.
    async fn login(&self, email: &str, password: &str) -> (StatusCode, Value) {
        self.request(
            Method::POST,
            "/app/auth/login",
            Some(json!({"email": email, "password": password})),
            None,
        )
        .await
    }
}

// ── Test 1: Signup → Login → Logout ────────────────────────────────────

#[tokio::test]
async fn test_signup_login_logout_flow() {
    let app = TestApp::new().await;

    // Signup
    let (token, body) = app.signup("flow").await;
    assert_eq!(body["user"]["role"].as_str().unwrap(), "owner");
    assert!(body["user"]["email"].as_str().unwrap().contains("@test.local"));
    assert!(body["tenant"]["slug"].as_str().is_some());
    assert!(body["initial_api_key"].as_str().is_some());

    // Use token → GET /app/auth/me
    let (status, me) = app
        .request(Method::GET, "/app/auth/me", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me["user"]["role"].as_str().unwrap(), "owner");

    // Login with same credentials
    let email = body["user"]["email"].as_str().unwrap();
    let (status, login_body) = app.login(email, TEST_PASSWORD).await;
    assert_eq!(status, StatusCode::OK, "login failed for {}: {:?}", email, login_body);
    let login_token = login_body["token"].as_str().unwrap().to_string();
    assert!(!login_token.is_empty());

    // Logout
    let (status, _) = app
        .request(Method::POST, "/app/auth/logout", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);

    // Old token should be invalid now
    let (status, _) = app
        .request(Method::GET, "/app/auth/me", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ── Test 2: API Key Management ─────────────────────────────────────────

#[tokio::test]
async fn test_api_key_management() {
    let app = TestApp::new().await;
    let (token, _) = app.signup("apikeys").await;

    // Create a new API key
    let (status, created) = app
        .request(
            Method::POST,
            "/app/api-keys",
            Some(json!({"name": "test-key", "role": "developer"})),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(created["api_key"].as_str().is_some());
    assert_eq!(created["name"].as_str().unwrap(), "test-key");
    assert_eq!(created["role"].as_str().unwrap(), "developer");
    let key_id = created["id"].as_str().unwrap();

    // List keys — should contain at least the initial key + the new key
    let (status, list) = app
        .request(Method::GET, "/app/api-keys", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);
    let keys = list["data"].as_array().expect("should have data array");
    assert!(keys.len() >= 2, "expected at least 2 keys (initial + new)");
    let found = keys.iter().any(|k| k["id"].as_str() == Some(key_id));
    assert!(found, "created key not found in list");

    // Revoke the key
    let (status, revoked) = app
        .request(
            Method::DELETE,
            &format!("/app/api-keys/{}", key_id),
            None,
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(revoked["revoked"].as_bool().unwrap(), true);

    // List again — key should be inactive
    let (status, list) = app
        .request(Method::GET, "/app/api-keys", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);
    let revoked_key = list["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|k| k["id"].as_str() == Some(key_id))
        .expect("key should still be in list");
    assert_eq!(revoked_key["active"].as_bool().unwrap(), false);
}

// ── Test 3: User Management ────────────────────────────────────────────

#[tokio::test]
async fn test_user_management() {
    let app = TestApp::new().await;
    let (owner_token, _) = app.signup("users").await;

    let dev_email = format!("dev-{}@test.local", Uuid::new_v4());

    // Create developer user
    let (status, created) = app
        .request(
            Method::POST,
            "/app/users",
            Some(json!({
                "full_name": "Dev User",
                "email": &dev_email,
                "password": TEST_PASSWORD,
                "role": "developer",
            })),
            Some(&owner_token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(created["role"].as_str().unwrap(), "developer");
    let user_id = created["id"].as_str().unwrap().to_string();

    // List users — should include the new user
    let (status, users) = app
        .request(Method::GET, "/app/users", None, Some(&owner_token))
        .await;
    assert_eq!(status, StatusCode::OK);
    let found = users["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|u| u["id"].as_str() == Some(&user_id));
    assert!(found, "created user not found in list");

    // Disable user
    let (status, _) = app
        .request(
            Method::PUT,
            &format!("/app/users/{}", user_id),
            Some(json!({"enabled": false})),
            Some(&owner_token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Disabled user cannot login
    let (status, _) = app.login(&dev_email, TEST_PASSWORD).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Re-enable user
    let (status, _) = app
        .request(
            Method::PUT,
            &format!("/app/users/{}", user_id),
            Some(json!({"enabled": true})),
            Some(&owner_token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Re-enabled user can login
    let (status, _) = app.login(&dev_email, TEST_PASSWORD).await;
    assert_eq!(status, StatusCode::OK);
}

// ── Test 4: Team Invites ───────────────────────────────────────────────

#[tokio::test]
async fn test_team_invites() {
    let app = TestApp::new().await;
    let (owner_token, _) = app.signup("invites").await;

    let invite_email = format!("invite-{}@test.local", Uuid::new_v4());

    // Create invite
    let (status, invite) = app
        .request(
            Method::POST,
            "/app/invites",
            Some(json!({"email": &invite_email, "role": "developer"})),
            Some(&owner_token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let invite_token = invite["token"].as_str().expect("no invite token").to_string();
    let invite_id = invite["id"].as_str().unwrap().to_string();

    // List invites — should have our pending invite
    let (status, list) = app
        .request(Method::GET, "/app/invites", None, Some(&owner_token))
        .await;
    assert_eq!(status, StatusCode::OK);
    let pending = list["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["id"].as_str() == Some(&invite_id));
    assert!(pending.is_some(), "invite not found in list");
    assert_eq!(pending.unwrap()["status"].as_str().unwrap(), "pending");

    // Accept invite
    let (status, accepted) = app
        .request(
            Method::POST,
            "/app/auth/accept-invite",
            Some(json!({
                "token": invite_token,
                "full_name": "Invited User",
                "password": TEST_PASSWORD,
            })),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "accept invite failed: {:?}", accepted);
    let invited_token = accepted["token"].as_str().unwrap().to_string();

    // Login as invited user
    let (status, login_body) = app.login(&invite_email, TEST_PASSWORD).await;
    assert_eq!(status, StatusCode::OK, "login as invited user failed: {:?}", login_body);
    // The role should be "developer" (viewer maps to read_only, but we passed developer)
    assert_eq!(
        login_body["user"]["role"].as_str().unwrap(),
        "developer"
    );

    // Use invited user's token to access /me
    let (status, _) = app
        .request(Method::GET, "/app/auth/me", None, Some(&invited_token))
        .await;
    assert_eq!(status, StatusCode::OK);

    // Create a second invite and revoke it
    let invite_email2 = format!("invite2-{}@test.local", Uuid::new_v4());
    let (status, invite2) = app
        .request(
            Method::POST,
            "/app/invites",
            Some(json!({"email": &invite_email2, "role": "admin"})),
            Some(&owner_token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let invite2_id = invite2["id"].as_str().unwrap();

    // Revoke the invite
    let (status, _) = app
        .request(
            Method::DELETE,
            &format!("/app/invites/{}", invite2_id),
            None,
            Some(&owner_token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // List invites — second invite should be revoked
    let (status, list) = app
        .request(Method::GET, "/app/invites", None, Some(&owner_token))
        .await;
    assert_eq!(status, StatusCode::OK);
    let revoked = list["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["id"].as_str() == Some(invite2_id));
    assert!(revoked.is_some());
    assert_eq!(revoked.unwrap()["status"].as_str().unwrap(), "revoked");
}

// ── Test 5: Settings ───────────────────────────────────────────────────

#[tokio::test]
async fn test_settings() {
    let app = TestApp::new().await;
    let (token, signup_body) = app.signup("settings").await;
    let email = signup_body["user"]["email"].as_str().unwrap().to_string();

    // Update workspace name
    let (status, res) = app
        .request(
            Method::PUT,
            "/app/settings/tenant",
            Some(json!({"name": "Renamed Workspace"})),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(res["updated"].as_bool().unwrap(), true);

    // Verify via GET /app/tenant
    let (status, tenant) = app
        .request(Method::GET, "/app/tenant", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(tenant["name"].as_str().unwrap(), "Renamed Workspace");

    // Update profile name
    let (status, res) = app
        .request(
            Method::PUT,
            "/app/settings/profile",
            Some(json!({"full_name": "Updated Name"})),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(res["updated"].as_bool().unwrap(), true);

    // Verify via GET /app/auth/me
    let (status, me) = app
        .request(Method::GET, "/app/auth/me", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me["user"]["full_name"].as_str().unwrap(), "Updated Name");

    // Change password
    let (status, pw_res) = app
        .request(
            Method::POST,
            "/app/settings/change-password",
            Some(json!({
                "current_password": TEST_PASSWORD,
                "new_password": NEW_PASSWORD,
            })),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "change password failed: {:?}", pw_res);
    assert_eq!(pw_res["password_changed"].as_bool().unwrap(), true);
    let new_token = pw_res["token"].as_str().unwrap().to_string();

    // Old password should fail
    let (status, _) = app.login(&email, TEST_PASSWORD).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // New password should work
    let (status, _) = app.login(&email, NEW_PASSWORD).await;
    assert_eq!(status, StatusCode::OK);

    // Old token should be invalid (sessions cleared)
    let (status, _) = app
        .request(Method::GET, "/app/auth/me", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // New token should work
    let (status, _) = app
        .request(Method::GET, "/app/auth/me", None, Some(&new_token))
        .await;
    assert_eq!(status, StatusCode::OK);
}

// ── Test 6: Billing without Stripe ─────────────────────────────────────

#[tokio::test]
async fn test_billing_without_stripe() {
    let app = TestApp::new().await;
    let (token, _) = app.signup("billing").await;

    // GET /app/billing/subscription — should return free plan info
    let (status, sub) = app
        .request(
            Method::GET,
            "/app/billing/subscription",
            None,
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(sub["plan"].as_str().unwrap(), "free");

    // GET /app/billing/usage — should return usage data
    let (status, usage) = app
        .request(Method::GET, "/app/billing/usage", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(usage["plan"].as_str().unwrap(), "free");
    assert!(usage["ops_this_month"].is_number());
    assert!(usage["month"].as_str().is_some());

    // GET /app/billing/usage/history — should return an array
    let (status, history) = app
        .request(
            Method::GET,
            "/app/billing/usage/history",
            None,
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(history.is_array());

    // POST /app/billing/checkout — should fail (Stripe not configured)
    let (status, _) = app
        .request(
            Method::POST,
            "/app/billing/checkout",
            Some(json!({"plan": "builder"})),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // POST /app/billing/portal — should fail (Stripe not configured)
    let (status, _) = app
        .request(
            Method::POST,
            "/app/billing/portal",
            None,
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ── Test 7: RBAC Authorization ─────────────────────────────────────────

#[tokio::test]
async fn test_rbac_authorization() {
    let app = TestApp::new().await;
    let (owner_token, _) = app.signup("rbac").await;

    let dev_email = format!("dev-rbac-{}@test.local", Uuid::new_v4());

    // Create developer user
    let (status, _) = app
        .request(
            Method::POST,
            "/app/users",
            Some(json!({
                "full_name": "RBAC Dev",
                "email": &dev_email,
                "password": TEST_PASSWORD,
                "role": "developer",
            })),
            Some(&owner_token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Login as developer
    let (status, login_body) = app.login(&dev_email, TEST_PASSWORD).await;
    assert_eq!(status, StatusCode::OK);
    let dev_token = login_body["token"].as_str().unwrap().to_string();

    // Developer CANNOT list users (admin/owner only)
    let (status, _) = app
        .request(Method::GET, "/app/users", None, Some(&dev_token))
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Developer CANNOT create invites
    let (status, _) = app
        .request(
            Method::POST,
            "/app/invites",
            Some(json!({"email": "nobody@test.local", "role": "developer"})),
            Some(&dev_token),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Developer CANNOT change workspace settings
    let (status, _) = app
        .request(
            Method::PUT,
            "/app/settings/tenant",
            Some(json!({"name": "Hacked"})),
            Some(&dev_token),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Developer CANNOT create API keys (admin/owner only)
    let (status, _) = app
        .request(
            Method::POST,
            "/app/api-keys",
            Some(json!({"name": "dev-key"})),
            Some(&dev_token),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Developer CANNOT list audit logs
    let (status, _) = app
        .request(Method::GET, "/app/audit", None, Some(&dev_token))
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Developer CAN list API keys (read access)
    let (status, _) = app
        .request(Method::GET, "/app/api-keys", None, Some(&dev_token))
        .await;
    assert_eq!(status, StatusCode::OK);

    // Developer CAN update own profile
    let (status, _) = app
        .request(
            Method::PUT,
            "/app/settings/profile",
            Some(json!({"full_name": "New Name"})),
            Some(&dev_token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Developer CAN view /app/auth/me
    let (status, _) = app
        .request(Method::GET, "/app/auth/me", None, Some(&dev_token))
        .await;
    assert_eq!(status, StatusCode::OK);

    // Developer CAN view billing usage
    let (status, _) = app
        .request(Method::GET, "/app/billing/usage", None, Some(&dev_token))
        .await;
    assert_eq!(status, StatusCode::OK);
}

// ── Test 8: Audit Log ──────────────────────────────────────────────────

#[tokio::test]
async fn test_audit_log() {
    let app = TestApp::new().await;
    let (token, _) = app.signup("audit").await;

    // Create an API key (generates audit entry)
    let (status, _) = app
        .request(
            Method::POST,
            "/app/api-keys",
            Some(json!({"name": "audit-key"})),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Create a user (generates audit entry)
    let dev_email = format!("audit-dev-{}@test.local", Uuid::new_v4());
    let (status, _) = app
        .request(
            Method::POST,
            "/app/users",
            Some(json!({
                "full_name": "Audit User",
                "email": &dev_email,
                "password": TEST_PASSWORD,
            })),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Fetch audit log
    let (status, logs) = app
        .request(Method::GET, "/app/audit", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);
    let entries = logs["data"].as_array().expect("audit should have data array");
    assert!(entries.len() >= 3, "expected at least 3 audit entries (signup + key + user), got {}", entries.len());

    // Verify each entry has expected fields
    for entry in entries {
        assert!(entry["action"].as_str().is_some(), "missing action");
        assert!(entry["resource_type"].as_str().is_some(), "missing resource_type");
        assert!(entry["created_at"].as_str().is_some(), "missing created_at");
    }

    // Verify we have a signup entry
    let signup_entry = entries
        .iter()
        .find(|e| e["action"].as_str() == Some("signup"));
    assert!(signup_entry.is_some(), "no signup audit entry found");

    // Verify we have an api_key create entry
    let key_entry = entries
        .iter()
        .find(|e| e["action"].as_str() == Some("create") && e["resource_type"].as_str() == Some("api_key"));
    assert!(key_entry.is_some(), "no api_key create audit entry found");

    // Verify we have a user create entry
    let user_entry = entries
        .iter()
        .find(|e| e["action"].as_str() == Some("create") && e["resource_type"].as_str() == Some("user"));
    assert!(user_entry.is_some(), "no user create audit entry found");
}

// ── Test 9: Input Validation ───────────────────────────────────────────

#[tokio::test]
async fn test_input_validation() {
    let app = TestApp::new().await;

    // Signup with short password
    let (status, body) = app
        .request(
            Method::POST,
            "/app/auth/signup",
            Some(json!({
                "company_name": "Valid Co",
                "full_name": "Valid Name",
                "email": format!("short-pw-{}@test.local", Uuid::new_v4()),
                "password": "short",
            })),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "short password should be rejected: {:?}", body);

    // Signup with password missing uppercase
    let (status, _) = app
        .request(
            Method::POST,
            "/app/auth/signup",
            Some(json!({
                "company_name": "Valid Co",
                "full_name": "Valid Name",
                "email": format!("no-upper-{}@test.local", Uuid::new_v4()),
                "password": "alllowercase1!@#",
            })),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Signup with empty company name
    let (status, _) = app
        .request(
            Method::POST,
            "/app/auth/signup",
            Some(json!({
                "company_name": "",
                "full_name": "Valid Name",
                "email": format!("empty-co-{}@test.local", Uuid::new_v4()),
                "password": TEST_PASSWORD,
            })),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Now signup successfully to test post-auth validations
    let (token, _) = app.signup("validation").await;

    // Create API key with invalid role
    let (status, _) = app
        .request(
            Method::POST,
            "/app/api-keys",
            Some(json!({"name": "bad-role-key", "role": "superadmin"})),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Create user with duplicate email (signup again with same email as the owner)
    let dup_email = format!("dup-{}@test.local", Uuid::new_v4());
    // First create the user
    let (status, _) = app
        .request(
            Method::POST,
            "/app/users",
            Some(json!({
                "full_name": "First User",
                "email": &dup_email,
                "password": TEST_PASSWORD,
            })),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Try creating again with same email
    let (status, _) = app
        .request(
            Method::POST,
            "/app/users",
            Some(json!({
                "full_name": "Duplicate User",
                "email": &dup_email,
                "password": TEST_PASSWORD,
            })),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

// ── Test 10: Tenant Isolation ──────────────────────────────────────────

#[tokio::test]
async fn test_tenant_isolation() {
    let app = TestApp::new().await;

    // Signup two tenants
    let (token_a, _) = app.signup("iso-a").await;
    let (token_b, _) = app.signup("iso-b").await;

    // Tenant A creates an API key
    let (status, key_a) = app
        .request(
            Method::POST,
            "/app/api-keys",
            Some(json!({"name": "tenant-a-key"})),
            Some(&token_a),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let key_a_id = key_a["id"].as_str().unwrap();

    // Tenant A creates a user
    let dev_email_a = format!("iso-dev-a-{}@test.local", Uuid::new_v4());
    let (status, _) = app
        .request(
            Method::POST,
            "/app/users",
            Some(json!({
                "full_name": "Tenant A Dev",
                "email": &dev_email_a,
                "password": TEST_PASSWORD,
            })),
            Some(&token_a),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Tenant B lists API keys — should NOT see Tenant A's keys
    let (status, keys_b) = app
        .request(Method::GET, "/app/api-keys", None, Some(&token_b))
        .await;
    assert_eq!(status, StatusCode::OK);
    let b_key_ids: Vec<&str> = keys_b["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|k| k["id"].as_str())
        .collect();
    assert!(
        !b_key_ids.contains(&key_a_id),
        "tenant B should not see tenant A's key"
    );

    // Tenant B lists users — should NOT see Tenant A's users
    let (status, users_b) = app
        .request(Method::GET, "/app/users", None, Some(&token_b))
        .await;
    assert_eq!(status, StatusCode::OK);
    let b_emails: Vec<&str> = users_b["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|u| u["email"].as_str())
        .collect();
    assert!(
        !b_emails.contains(&dev_email_a.as_str()),
        "tenant B should not see tenant A's users"
    );

    // Tenant B tries to revoke Tenant A's API key — should get 404
    let (status, _) = app
        .request(
            Method::DELETE,
            &format!("/app/api-keys/{}", key_a_id),
            None,
            Some(&token_b),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── Test 11: No auth header ────────────────────────────────────────────

#[tokio::test]
async fn test_no_auth_header() {
    let app = TestApp::new().await;

    // Request without any auth token
    let (status, body) = app
        .request(Method::GET, "/app/auth/me", None, None)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(body["error"].as_str().is_some());
}

// ── Test 12: Malformed JWT ─────────────────────────────────────────────

#[tokio::test]
async fn test_malformed_jwt() {
    let app = TestApp::new().await;

    // Use random garbage as a Bearer token
    let (status, body) = app
        .request(Method::GET, "/app/auth/me", None, Some("not-a-valid-jwt"))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(body["error"].as_str().is_some());
}

// ── Test 13: Signup duplicate email ────────────────────────────────────

#[tokio::test]
async fn test_signup_duplicate_email() {
    let app = TestApp::new().await;

    let id = Uuid::new_v4().to_string();
    let email = format!("dup-{}@test.local", id);
    let company1 = format!("DupCo1-{}", &id[..8]);
    let company2 = format!("DupCo2-{}", &id[..8]);

    // First signup should succeed
    let (status, _) = app
        .request(
            Method::POST,
            "/app/auth/signup",
            Some(json!({
                "company_name": company1,
                "full_name": "First User",
                "email": email,
                "password": TEST_PASSWORD,
            })),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Second signup with same email should fail with 409
    let (status, _) = app
        .request(
            Method::POST,
            "/app/auth/signup",
            Some(json!({
                "company_name": company2,
                "full_name": "Second User",
                "email": email,
                "password": TEST_PASSWORD,
            })),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

// ── Test 14: Login wrong password ──────────────────────────────────────

#[tokio::test]
async fn test_login_wrong_password() {
    let app = TestApp::new().await;

    let (_, signup_body) = app.signup("wrongpw").await;
    let email = signup_body["user"]["email"].as_str().unwrap();

    // Login with wrong password
    let (status, _) = app.login(email, "WrongPassword123!@#").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ── Test 15: Update user role ──────────────────────────────────────────

#[tokio::test]
async fn test_update_user_role() {
    let app = TestApp::new().await;

    // Signup as owner
    let (token, _) = app.signup("updaterole").await;

    // Create a developer user
    let id = Uuid::new_v4().to_string();
    let dev_email = format!("dev-role-{}@test.local", id);
    let (status, dev_body) = app
        .request(
            Method::POST,
            "/app/users",
            Some(json!({
                "email": dev_email,
                "full_name": "Dev User",
                "password": TEST_PASSWORD,
                "role": "developer",
            })),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let dev_id = dev_body["id"].as_str().unwrap();

    // Update role to admin
    let (status, _) = app
        .request(
            Method::PUT,
            &format!("/app/users/{}", dev_id),
            Some(json!({"role": "admin"})),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Verify role changed
    let (status, users) = app
        .request(Method::GET, "/app/users", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);
    let updated_user = users["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["id"].as_str() == Some(dev_id))
        .expect("user not found");
    assert_eq!(updated_user["role"].as_str().unwrap(), "admin");
}

// ── Test 16: Billing cancel/reactivate flow ────────────────────────────

#[tokio::test]
async fn test_billing_cancel_without_subscription() {
    let app = TestApp::new().await;

    let (token, _) = app.signup("cancelnosub").await;

    // Cancel when there's no active Stripe subscription — should get an error
    let (status, _) = app
        .request(Method::POST, "/app/billing/cancel", None, Some(&token))
        .await;
    // 400 because no subscription exists to cancel
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::INTERNAL_SERVER_ERROR,
        "expected 400 or 500, got {}",
        status
    );
}

// ── Test 17: Signup validation ─────────────────────────────────────────

#[tokio::test]
async fn test_signup_validation() {
    let app = TestApp::new().await;
    let id = Uuid::new_v4().to_string();

    // Short password (< 12 chars)
    let (status, body) = app
        .request(
            Method::POST,
            "/app/auth/signup",
            Some(json!({
                "company_name": format!("ValCo-{}", &id[..8]),
                "full_name": "Test",
                "email": format!("val-{}@test.local", id),
                "password": "Short1!",
            })),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "short password: {:?}", body);

    // Missing uppercase
    let (status, _) = app
        .request(
            Method::POST,
            "/app/auth/signup",
            Some(json!({
                "company_name": format!("ValCo2-{}", &id[..8]),
                "full_name": "Test",
                "email": format!("val2-{}@test.local", id),
                "password": "nouppercase1234!",
            })),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Empty company name
    let (status, _) = app
        .request(
            Method::POST,
            "/app/auth/signup",
            Some(json!({
                "company_name": "",
                "full_name": "Test",
                "email": format!("val3-{}@test.local", id),
                "password": TEST_PASSWORD,
            })),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ── Test 18: Expired session rejected ──────────────────────────────────

#[tokio::test]
async fn test_expired_session_rejected() {
    let app = TestApp::new().await;

    let (token, _) = app.signup("expired").await;

    // Verify token works
    let (status, _) = app
        .request(Method::GET, "/app/auth/me", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);

    // Manually expire all sessions for this user by setting expires_at to the past
    // We do this by getting the user ID from the token, then updating the DB
    let (_, me) = app
        .request(Method::GET, "/app/auth/me", None, Some(&token))
        .await;
    let user_id = me["user"]["id"].as_str().unwrap();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://memory:memory_dev@localhost:5432/memory".into());
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .unwrap();

    sqlx::query("UPDATE app_sessions SET expires_at = NOW() - INTERVAL '1 hour' WHERE app_user_id = $1")
        .bind(uuid::Uuid::parse_str(user_id).unwrap())
        .execute(&pool)
        .await
        .unwrap();

    // Token should now be rejected
    let (status, _) = app
        .request(Method::GET, "/app/auth/me", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ── Test 19: Workspace name too short ──────────────────────────────────

#[tokio::test]
async fn test_update_workspace_name_too_short() {
    let app = TestApp::new().await;
    let (token, _) = app.signup("nameshort").await;

    // Name with 1 character should fail
    let (status, body) = app
        .request(
            Method::PUT,
            "/app/settings/tenant",
            Some(json!({"name": "A"})),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "1-char name should be rejected: {:?}", body);

    // Empty name should fail
    let (status, _) = app
        .request(
            Method::PUT,
            "/app/settings/tenant",
            Some(json!({"name": ""})),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Whitespace-only name should fail
    let (status, _) = app
        .request(
            Method::PUT,
            "/app/settings/tenant",
            Some(json!({"name": "   "})),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ── Test 20: Change password wrong current ─────────────────────────────

#[tokio::test]
async fn test_change_password_wrong_current() {
    let app = TestApp::new().await;
    let (token, _) = app.signup("wrongcur").await;

    let (status, body) = app
        .request(
            Method::POST,
            "/app/settings/change-password",
            Some(json!({
                "current_password": "WrongPassword123!@#",
                "new_password": NEW_PASSWORD,
            })),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "wrong current password: {:?}", body);
    assert!(
        body["error"].as_str().unwrap_or("").contains("incorrect")
            || body["error"].as_str().unwrap_or("").contains("Current password"),
        "error should mention incorrect password: {:?}",
        body
    );
}

// ── Test 21: Create API key with expiry ────────────────────────────────

#[tokio::test]
async fn test_create_api_key_with_expiry() {
    let app = TestApp::new().await;
    let (token, _) = app.signup("keyexpiry").await;

    // Create key with 30-day expiry
    let (status, created) = app
        .request(
            Method::POST,
            "/app/api-keys",
            Some(json!({"name": "expiring-key", "role": "developer", "expires_in_days": 30})),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "create key with expiry failed: {:?}", created);
    assert!(created["api_key"].as_str().is_some());
    assert!(created["expires_at"].as_str().is_some(), "key should have expires_at field");

    // Verify expiry is roughly 30 days from now
    let expires = created["expires_at"].as_str().unwrap();
    let expires_dt = chrono::DateTime::parse_from_rfc3339(expires)
        .expect("expires_at should be valid RFC3339");
    let now = chrono::Utc::now();
    let days_diff = (expires_dt.timestamp() - now.timestamp()) / 86400;
    assert!(
        (29..=31).contains(&days_diff),
        "expiry should be ~30 days from now, got {} days",
        days_diff
    );
}

// ── Test 22: Accept expired invite ─────────────────────────────────────

#[tokio::test]
async fn test_accept_expired_invite() {
    let app = TestApp::new().await;
    let (owner_token, _) = app.signup("expinv").await;

    let invite_email = format!("expinv-{}@test.local", Uuid::new_v4());

    // Create invite
    let (status, invite) = app
        .request(
            Method::POST,
            "/app/invites",
            Some(json!({"email": &invite_email, "role": "developer"})),
            Some(&owner_token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let invite_token = invite["token"].as_str().unwrap().to_string();
    let invite_id = invite["id"].as_str().unwrap().to_string();

    // Manually expire the invite
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://memory:memory_dev@localhost:5432/memory".into());
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .unwrap();

    sqlx::query("UPDATE team_invites SET expires_at = NOW() - INTERVAL '1 hour' WHERE id = $1")
        .bind(Uuid::parse_str(&invite_id).unwrap())
        .execute(&pool)
        .await
        .unwrap();

    // Try to accept the expired invite
    let (status, _) = app
        .request(
            Method::POST,
            "/app/auth/accept-invite",
            Some(json!({
                "token": invite_token,
                "full_name": "Expired User",
                "password": TEST_PASSWORD,
            })),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "expired invite should return 404");
}

// ── Test 23: Profile name too short ────────────────────────────────────

#[tokio::test]
async fn test_update_profile_name_too_short() {
    let app = TestApp::new().await;
    let (token, _) = app.signup("profshort").await;

    let (status, _) = app
        .request(
            Method::PUT,
            "/app/settings/profile",
            Some(json!({"full_name": "A"})),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = app
        .request(
            Method::PUT,
            "/app/settings/profile",
            Some(json!({"full_name": ""})),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ── Test 24: Per-account lockout ────────────────────────────────────────

#[tokio::test]
async fn test_account_lockout() {
    let app = TestApp::new().await;

    let (_, signup_body) = app.signup("lockout").await;
    let email = signup_body["user"]["email"].as_str().unwrap().to_string();

    // Make 5 failed login attempts to trigger lockout
    for i in 0..5 {
        let (status, _) = app.login(&email, "WrongPassword123!@#").await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "attempt {} should fail with 401", i + 1);
    }

    // 6th attempt should be rate-limited (account locked)
    let (status, body) = app.login(&email, "WrongPassword123!@#").await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "locked account should return 429: {:?}", body);

    // Even correct password should be rejected while locked
    let (status, _) = app.login(&email, TEST_PASSWORD).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "correct password should still be rejected while locked");
}

// ── Test 25: Password reset flow ────────────────────────────────────────

#[tokio::test]
async fn test_password_reset_flow() {
    let app = TestApp::new().await;

    let (owner_token, signup_body) = app.signup("pwreset").await;
    let owner_id = signup_body["user"]["id"].as_str().unwrap().to_string();

    // Create a developer user
    let dev_email = format!("dev-reset-{}@test.local", Uuid::new_v4());
    let (status, dev_body) = app
        .request(
            Method::POST,
            "/app/users",
            Some(json!({
                "full_name": "Reset User",
                "email": &dev_email,
                "password": TEST_PASSWORD,
                "role": "developer",
            })),
            Some(&owner_token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let dev_id = dev_body["id"].as_str().unwrap().to_string();

    // Owner initiates password reset for the developer
    let (status, reset) = app
        .request(
            Method::POST,
            "/app/auth/password-reset",
            Some(json!({"user_id": dev_id})),
            Some(&owner_token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "initiate reset failed: {:?}", reset);
    let reset_token = reset["token"].as_str().expect("no reset token").to_string();
    assert!(reset["expires_at"].as_str().is_some());

    // Developer uses the token to reset their password
    let (status, reset_res) = app
        .request(
            Method::POST,
            "/app/auth/reset-password",
            Some(json!({
                "token": reset_token,
                "new_password": NEW_PASSWORD,
            })),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "reset password failed: {:?}", reset_res);
    assert!(reset_res["token"].as_str().is_some(), "should return new session token");

    // Login with new password should work
    let (status, _) = app.login(&dev_email, NEW_PASSWORD).await;
    assert_eq!(status, StatusCode::OK);

    // Login with old password should fail
    let (status, _) = app.login(&dev_email, TEST_PASSWORD).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Using the same reset token again should fail
    let (status, _) = app
        .request(
            Method::POST,
            "/app/auth/reset-password",
            Some(json!({
                "token": reset_token,
                "new_password": "AnotherPass1!@#",
            })),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "reused token should fail");

    // Non-admin cannot initiate password reset
    let (status, login_body) = app.login(&dev_email, NEW_PASSWORD).await;
    assert_eq!(status, StatusCode::OK);
    let dev_token = login_body["token"].as_str().unwrap().to_string();

    let (status, _) = app
        .request(
            Method::POST,
            "/app/auth/password-reset",
            Some(json!({"user_id": owner_id})),
            Some(&dev_token),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ── Test 26: Session listing and revocation ────────────────────────────

#[tokio::test]
async fn test_session_management() {
    let app = TestApp::new().await;

    let (token, signup_body) = app.signup("sessions").await;
    let email = signup_body["user"]["email"].as_str().unwrap().to_string();

    // Login again to create a second session
    let (status, login_body) = app.login(&email, TEST_PASSWORD).await;
    assert_eq!(status, StatusCode::OK);
    let _second_token = login_body["token"].as_str().unwrap().to_string();

    // List sessions — should have at least 2
    let (status, sessions) = app
        .request(Method::GET, "/app/auth/sessions", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);
    let session_list = sessions.as_array().expect("should be array");
    assert!(session_list.len() >= 2, "expected at least 2 sessions, got {}", session_list.len());

    // One session should be marked as current
    let current = session_list.iter().find(|s| s["current"].as_bool() == Some(true));
    assert!(current.is_some(), "should have one current session");

    // Find a non-current session to revoke
    let non_current = session_list.iter().find(|s| s["current"].as_bool() == Some(false));
    assert!(non_current.is_some(), "should have a non-current session");
    let revoke_id = non_current.unwrap()["id"].as_str().unwrap();

    // Revoke the non-current session
    let (status, _) = app
        .request(
            Method::DELETE,
            &format!("/app/auth/sessions/{}", revoke_id),
            None,
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Try to revoke current session — should fail
    let current_id = current.unwrap()["id"].as_str().unwrap();
    let (status, _) = app
        .request(
            Method::DELETE,
            &format!("/app/auth/sessions/{}", current_id),
            None,
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ── Test 27: Pagination response format ─────────────────────────────────

#[tokio::test]
async fn test_pagination_response_format() {
    let app = TestApp::new().await;
    let (token, _) = app.signup("pagination").await;

    // GET /app/api-keys should return paginated response
    let (status, body) = app
        .request(Method::GET, "/app/api-keys", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["data"].is_array(), "should have data array");
    assert!(body["total"].is_number(), "should have total");
    assert!(body["page"].is_number(), "should have page");
    assert!(body["per_page"].is_number(), "should have per_page");
    assert_eq!(body["page"].as_u64().unwrap(), 1);
    assert!(body["per_page"].as_u64().unwrap() <= 100);

    // GET /app/users should return paginated response
    let (status, body) = app
        .request(Method::GET, "/app/users", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["data"].is_array());
    assert!(body["total"].is_number());

    // GET /app/audit should return paginated response
    let (status, body) = app
        .request(Method::GET, "/app/audit", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["data"].is_array());
    assert!(body["total"].is_number());

    // GET /app/invites should return paginated response
    let (status, body) = app
        .request(Method::GET, "/app/invites", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["data"].is_array());
    assert!(body["total"].is_number());
}

// ── Test 28: Request correlation ID ──────────────────────────────────

#[tokio::test]
async fn test_request_correlation_id() {
    let app = TestApp::new().await;
    let (token, _) = app.signup("reqid").await;

    // Response should include X-Request-ID header (auto-generated)
    let (status, _, headers) = app
        .request_with_headers(Method::GET, "/app/auth/me", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);
    let request_id = headers.get("x-request-id");
    assert!(request_id.is_some(), "response should include X-Request-ID header");
    let id_val = request_id.unwrap().to_str().unwrap();
    assert!(!id_val.is_empty());
    // Should be a valid UUID
    assert!(Uuid::parse_str(id_val).is_ok(), "X-Request-ID should be a valid UUID");

    // When we send our own X-Request-ID, it should be echoed back
    let custom_id = Uuid::new_v4().to_string();
    let builder = Request::builder()
        .method(Method::GET)
        .uri("/app/auth/me")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .header("x-request-id", &custom_id);
    let req = builder.body(Body::empty()).unwrap();
    let response = app.router.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let echoed = response.headers().get("x-request-id").unwrap().to_str().unwrap();
    assert_eq!(echoed, custom_id, "should echo back the provided X-Request-ID");
}

// ── Test 29: Token refresh ──────────────────────────────────────────

#[tokio::test]
async fn test_token_refresh() {
    let app = TestApp::new().await;
    let (token, _) = app.signup("refresh").await;

    // Verify current token works
    let (status, _) = app
        .request(Method::GET, "/app/auth/me", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);

    // Refresh the token
    let (status, refresh_body) = app
        .request(Method::POST, "/app/auth/refresh", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK, "refresh failed: {:?}", refresh_body);
    let new_token = refresh_body["token"].as_str().expect("no new token").to_string();
    assert!(refresh_body["expires_at"].as_str().is_some());
    assert_ne!(token, new_token, "new token should differ from old token");

    // Old token should be invalid (session was rotated)
    let (status, _) = app
        .request(Method::GET, "/app/auth/me", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "old token should be invalid after refresh");

    // New token should work
    let (status, _) = app
        .request(Method::GET, "/app/auth/me", None, Some(&new_token))
        .await;
    assert_eq!(status, StatusCode::OK, "new token should work");
}

// ── Test 30: Idle session timeout ──────────────────────────────────────

#[tokio::test]
async fn test_idle_session_timeout() {
    // Create a TestApp with a very short idle timeout
    std::env::set_var("ADMIN_SECURE_COOKIES", "false");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://memory:memory_dev@localhost:5432/memory".into());

    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let state = Arc::new(TenantAppState {
        db_pool: db_pool.clone(),
        jwt_secret: "test-jwt-secret-must-be-32-chars-long!!".to_string(),
        http_client: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap(),
        rate_limiter: enterprise_common::rate_limit::new_rate_limiter(),
        api_rate_limiter: enterprise_common::api_rate_limit::ApiRateLimiter::new(),
        stripe_secret_key: None,
        stripe_webhook_secret: None,
        idle_timeout_mins: 1, // 1 minute timeout
        totp_encryption_key: None,
        secure_cookies: false,
    });

    let app_protected = Router::new()
        .route("/auth/me", get(routes::app::me))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::app_auth_middleware,
        ));
    let app_routes = Router::new()
        .route("/auth/signup", post(routes::app::signup))
        .merge(app_protected);
    let router = Router::new().nest("/app", app_routes).with_state(state);
    let app = TestApp { router };

    // Signup to get a token
    let (token, _) = app.signup("idle").await;

    // Verify it works initially
    let (status, _) = app
        .request(Method::GET, "/app/auth/me", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);

    // Set last_activity_at to 2 minutes ago to simulate idle session
    let (_, me) = app
        .request(Method::GET, "/app/auth/me", None, Some(&token))
        .await;
    let user_id = me["user"]["id"].as_str().unwrap();
    sqlx::query("UPDATE app_sessions SET last_activity_at = NOW() - INTERVAL '2 minutes' WHERE app_user_id = $1")
        .bind(Uuid::parse_str(user_id).unwrap())
        .execute(&db_pool)
        .await
        .unwrap();

    // Token should now be rejected due to idle timeout
    let (status, body) = app
        .request(Method::GET, "/app/auth/me", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "idle session should be rejected: {:?}", body);
}

// ── Test 31: Admin verify email ──────────────────────────────────────

#[tokio::test]
async fn test_admin_verify_email() {
    let app = TestApp::new().await;
    let (owner_token, _) = app.signup("verifyemail").await;

    // Create a developer user
    let dev_email = format!("dev-verify-{}@test.local", Uuid::new_v4());
    let (status, dev_body) = app
        .request(
            Method::POST,
            "/app/users",
            Some(json!({
                "full_name": "Verify User",
                "email": &dev_email,
                "password": TEST_PASSWORD,
                "role": "developer",
            })),
            Some(&owner_token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let dev_id = dev_body["id"].as_str().unwrap().to_string();

    // Verify the user's email as admin
    let (status, res) = app
        .request(
            Method::POST,
            "/app/auth/verify-email",
            Some(json!({"user_id": dev_id})),
            Some(&owner_token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "verify email failed: {:?}", res);
    assert_eq!(res["verified"].as_bool().unwrap(), true);

    // Login as developer and check email_verified is true
    let (status, login_body) = app.login(&dev_email, TEST_PASSWORD).await;
    assert_eq!(status, StatusCode::OK);
    let dev_token = login_body["token"].as_str().unwrap().to_string();

    let (status, me) = app
        .request(Method::GET, "/app/auth/me", None, Some(&dev_token))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me["user"]["email_verified"].as_bool().unwrap(), true);

    // Non-admin cannot verify email
    let (status, _) = app
        .request(
            Method::POST,
            "/app/auth/verify-email",
            Some(json!({"user_id": dev_id})),
            Some(&dev_token),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ── Test 32: Data export ─────────────────────────────────────────────

#[tokio::test]
async fn test_data_export() {
    let app = TestApp::new().await;
    let (token, signup_body) = app.signup("dataexport").await;
    let email = signup_body["user"]["email"].as_str().unwrap().to_string();

    // Fetch data export
    let (status, export) = app
        .request(Method::GET, "/app/settings/data-export", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK, "data export failed: {:?}", export);

    // Verify export structure
    assert!(export["export_date"].as_str().is_some(), "should have export_date");
    assert_eq!(export["user"]["email"].as_str().unwrap(), email);
    assert!(export["user"]["id"].as_str().is_some());
    assert!(export["user"]["role"].as_str().is_some());
    assert!(export["sessions"].is_array(), "should have sessions array");
    assert!(export["audit_log"].is_array(), "should have audit_log array");

    // Sessions should include at least the current session
    let sessions = export["sessions"].as_array().unwrap();
    assert!(!sessions.is_empty(), "should have at least one session");
}

// ── Test 33: Account deletion ────────────────────────────────────────

#[tokio::test]
async fn test_account_deletion() {
    let app = TestApp::new().await;

    // Owner cannot delete if they're the only owner
    let (owner_token, _) = app.signup("deleteacct").await;
    let (status, body) = app
        .request(
            Method::POST,
            "/app/settings/delete-account",
            Some(json!({"password": TEST_PASSWORD})),
            Some(&owner_token),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "sole owner should not be able to delete: {:?}", body);

    // Create a second owner, then the first can delete
    let dev_email = format!("del-dev-{}@test.local", Uuid::new_v4());
    let (status, dev_body) = app
        .request(
            Method::POST,
            "/app/users",
            Some(json!({
                "full_name": "Second User",
                "email": &dev_email,
                "password": TEST_PASSWORD,
                "role": "developer",
            })),
            Some(&owner_token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let _dev_id = dev_body["id"].as_str().unwrap().to_string();

    // Login as developer and try to delete their own account
    let (status, login_body) = app.login(&dev_email, TEST_PASSWORD).await;
    assert_eq!(status, StatusCode::OK);
    let dev_token = login_body["token"].as_str().unwrap().to_string();

    // Wrong password should fail
    let (status, _) = app
        .request(
            Method::POST,
            "/app/settings/delete-account",
            Some(json!({"password": "WrongPassword123!@#"})),
            Some(&dev_token),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Correct password should schedule deletion
    let (status, del_res) = app
        .request(
            Method::POST,
            "/app/settings/delete-account",
            Some(json!({"password": TEST_PASSWORD})),
            Some(&dev_token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "delete account failed: {:?}", del_res);
    assert_eq!(del_res["scheduled"].as_bool().unwrap(), true);
    assert!(del_res["deletion_date"].as_str().is_some());

    // Token should be invalidated after deletion
    let (status, _) = app
        .request(Method::GET, "/app/auth/me", None, Some(&dev_token))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ── Test 34: CSRF protection ─────────────────────────────────────────

#[tokio::test]
async fn test_csrf_required_for_mutations() {
    let app = TestApp::new().await;
    let (token, _) = app.signup("csrf").await;

    // GET should work without CSRF
    let (status, _) = app
        .request_no_csrf(Method::GET, "/app/auth/me", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK, "GET should work without CSRF");

    // POST without CSRF should fail with 403
    let (status, body) = app
        .request_no_csrf(
            Method::POST,
            "/app/auth/logout",
            None,
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "POST without CSRF should be rejected: {:?}", body);
    assert!(
        body["error"].as_str().unwrap_or("").contains("CSRF"),
        "error should mention CSRF: {:?}",
        body
    );

    // PUT without CSRF should fail
    let (status, _) = app
        .request_no_csrf(
            Method::PUT,
            "/app/settings/profile",
            Some(json!({"full_name": "New Name"})),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "PUT without CSRF should be rejected");

    // POST with CSRF should succeed
    let (status, _) = app
        .request(Method::POST, "/app/auth/logout", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK, "POST with CSRF should succeed");
}

// ── Test 35: TOTP setup ──────────────────────────────────────────────

#[tokio::test]
async fn test_totp_setup() {
    let app = TestApp::new().await;
    let (token, _) = app.signup("totpsetup").await;

    // Setup TOTP
    let (status, setup) = app
        .request(Method::POST, "/app/settings/totp/setup", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK, "TOTP setup failed: {:?}", setup);
    assert!(setup["secret"].as_str().is_some(), "should return secret");
    assert!(setup["qr_uri"].as_str().is_some(), "should return qr_uri");
    assert!(setup["backup_codes"].is_array(), "should return backup codes");

    let qr_uri = setup["qr_uri"].as_str().unwrap();
    assert!(qr_uri.starts_with("otpauth://totp/"), "qr_uri should be otpauth format");
    assert!(qr_uri.contains("algorithm=SHA1"), "should use SHA1");
    assert!(qr_uri.contains("digits=6"), "should use 6 digits");

    let backup_codes = setup["backup_codes"].as_array().unwrap();
    assert_eq!(backup_codes.len(), 10, "should have 10 backup codes");

    // Verify secret is base32 (all uppercase letters and 2-7)
    let secret = setup["secret"].as_str().unwrap();
    assert!(secret.chars().all(|c| matches!(c, 'A'..='Z' | '2'..='7')));
}

// ── Test 36: TOTP enable/disable cycle ──────────────────────────────

#[tokio::test]
async fn test_totp_enable_disable() {
    let app = TestApp::new().await;
    let (token, _) = app.signup("totpcycle").await;

    // Setup TOTP
    let (status, setup) = app
        .request(Method::POST, "/app/settings/totp/setup", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);
    let secret = setup["secret"].as_str().unwrap().to_string();

    // Enable with wrong code should fail
    let (status, _) = app
        .request(
            Method::POST,
            "/app/settings/totp/enable",
            Some(json!({"code": "000000"})),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "wrong code should fail");

    // Generate a valid TOTP code from the secret and enable
    let valid_code = generate_test_totp(&secret);
    let (status, enable_res) = app
        .request(
            Method::POST,
            "/app/settings/totp/enable",
            Some(json!({"code": valid_code})),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "enable TOTP failed: {:?}", enable_res);
    assert_eq!(enable_res["enabled"].as_bool().unwrap(), true);

    // Verify TOTP is enabled via /me
    let (status, me) = app
        .request(Method::GET, "/app/auth/me", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me["user"]["totp_enabled"].as_bool().unwrap(), true);

    // Disable with wrong password should fail
    let (status, _) = app
        .request(
            Method::POST,
            "/app/settings/totp/disable",
            Some(json!({"password": "WrongPassword123!@#"})),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Disable with correct password
    let (status, disable_res) = app
        .request(
            Method::POST,
            "/app/settings/totp/disable",
            Some(json!({"password": TEST_PASSWORD})),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "disable TOTP failed: {:?}", disable_res);
    assert_eq!(disable_res["disabled"].as_bool().unwrap(), true);

    // Verify TOTP is disabled via /me
    let (status, me) = app
        .request(Method::GET, "/app/auth/me", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me["user"]["totp_enabled"].as_bool().unwrap(), false);
}

// ── Test 37: Login with TOTP required ───────────────────────────────

#[tokio::test]
async fn test_login_with_totp() {
    let app = TestApp::new().await;
    let (token, signup_body) = app.signup("totplogin").await;
    let email = signup_body["user"]["email"].as_str().unwrap().to_string();

    // Setup and enable TOTP
    let (status, setup) = app
        .request(Method::POST, "/app/settings/totp/setup", None, Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);
    let secret = setup["secret"].as_str().unwrap().to_string();
    let backup_codes = setup["backup_codes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect::<Vec<_>>();

    let valid_code = generate_test_totp(&secret);
    let (status, _) = app
        .request(
            Method::POST,
            "/app/settings/totp/enable",
            Some(json!({"code": valid_code})),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Login should now return totp_required instead of a full session
    let (status, login_body) = app.login(&email, TEST_PASSWORD).await;
    assert_eq!(status, StatusCode::OK, "login failed: {:?}", login_body);
    assert_eq!(login_body["totp_required"].as_bool().unwrap(), true);
    let totp_token = login_body["totp_token"].as_str().unwrap().to_string();
    assert!(login_body.get("token").is_none() || login_body["token"].is_null(),
        "should not return a full session token when TOTP is required");

    // Verify with wrong code should fail
    let (status, _) = app
        .request(
            Method::POST,
            "/app/auth/totp/verify",
            Some(json!({"totp_token": &totp_token, "code": "000000"})),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "wrong TOTP code should be rejected");

    // Verify with correct code should succeed
    let valid_code = generate_test_totp(&secret);
    let (status, verify_body) = app
        .request(
            Method::POST,
            "/app/auth/totp/verify",
            Some(json!({"totp_token": &totp_token, "code": valid_code})),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "TOTP verify failed: {:?}", verify_body);
    let session_token = verify_body["token"].as_str().expect("should return session token");
    assert!(!session_token.is_empty());

    // Session token should work
    let (status, _) = app
        .request(Method::GET, "/app/auth/me", None, Some(session_token))
        .await;
    assert_eq!(status, StatusCode::OK);

    // Test backup code login
    let (status, login_body2) = app.login(&email, TEST_PASSWORD).await;
    assert_eq!(status, StatusCode::OK);
    let totp_token2 = login_body2["totp_token"].as_str().unwrap().to_string();

    let (status, verify_body2) = app
        .request(
            Method::POST,
            "/app/auth/totp/verify",
            Some(json!({"totp_token": &totp_token2, "code": &backup_codes[0]})),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "backup code verify failed: {:?}", verify_body2);
    assert!(verify_body2["token"].as_str().is_some());
}

// ── TOTP test helper ─────────────────────────────────────────────────

/// Generate a valid TOTP code for testing.
fn generate_test_totp(secret_base32: &str) -> String {
    use hmac::{Hmac, Mac};
    type HmacSha1 = Hmac<sha1::Sha1>;

    // Base32 decode the secret
    let secret = base32_decode_test(secret_base32);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let time_step = now / 30;

    let time_bytes = time_step.to_be_bytes();
    let mut mac = HmacSha1::new_from_slice(&secret).expect("HMAC accepts any key length");
    mac.update(&time_bytes);
    let result = mac.finalize().into_bytes();

    let offset = (result[result.len() - 1] & 0x0f) as usize;
    let code = ((result[offset] as u32 & 0x7f) << 24)
        | ((result[offset + 1] as u32) << 16)
        | ((result[offset + 2] as u32) << 8)
        | (result[offset + 3] as u32);

    format!("{:06}", code % 1_000_000)
}

fn base32_decode_test(input: &str) -> Vec<u8> {
    let mut result = Vec::new();
    let mut buffer: u64 = 0;
    let mut bits = 0;
    for c in input.chars() {
        let val = match c {
            'A'..='Z' => c as u64 - 'A' as u64,
            'a'..='z' => c as u64 - 'a' as u64,
            '2'..='7' => c as u64 - '2' as u64 + 26,
            '=' => continue,
            _ => panic!("invalid base32 char"),
        };
        buffer = (buffer << 5) | val;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            result.push((buffer >> bits) as u8);
        }
    }
    result
}
