//! Tenant Service
//!
//! Dedicated service for SaaS tenant operations: authentication,
//! billing (Stripe), team invites, settings, API keys, and usage tracking.
//! Includes OpenAPI/Swagger documentation at /docs.

pub mod auth;
pub mod openapi;
pub mod routes;
pub mod stripe;

pub use openapi::ApiDoc;

use axum::{
    extract::DefaultBodyLimit,
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, COOKIE},
        HeaderValue, Method,
    },
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tracing::warn;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Shared state for the tenant service.
pub struct TenantAppState {
    pub db_pool: sqlx::PgPool,
    pub jwt_secret: String,
    pub http_client: reqwest::Client,
    /// Per-IP rate limiter for auth endpoints.
    pub rate_limiter: enterprise_common::rate_limit::RateLimiter,
    /// Per-tenant rate limiter for all protected API endpoints.
    pub api_rate_limiter: enterprise_common::api_rate_limit::ApiRateLimiter,
    /// Stripe API secret key for billing integration.
    pub stripe_secret_key: Option<String>,
    /// Stripe webhook signing secret for signature verification.
    pub stripe_webhook_secret: Option<String>,
    /// Idle session timeout in minutes (0 = disabled).
    pub idle_timeout_mins: i64,
    /// Encryption key for TOTP secrets (32 bytes hex-encoded).
    pub totp_encryption_key: Option<String>,
    /// Whether cookies should have the Secure flag.
    pub secure_cookies: bool,
}

/// Build the tenant service router with all routes, middleware, and CORS.
pub fn build_router(
    state: Arc<TenantAppState>,
    allowed_origin: &str,
    insecure_dev_mode: bool,
) -> Result<Router, anyhow::Error> {
    let allowed_methods = [
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::OPTIONS,
    ];
    let allowed_headers = [
        AUTHORIZATION,
        CONTENT_TYPE,
        ACCEPT,
        COOKIE,
        axum::http::header::HeaderName::from_static("x-csrf-token"),
    ];

    let cors = if allowed_origin == "*" {
        if !insecure_dev_mode {
            return Err(anyhow::anyhow!(
                "TENANT_CORS_ORIGIN='*' is not allowed in production (credential cookies require explicit origins). \
                 Set TENANT_CORS_ORIGIN to your frontend URL, or set ADMIN_SECURE_COOKIES=false for local dev."
            ));
        }
        warn!("TENANT_CORS_ORIGIN='*' with ADMIN_SECURE_COOKIES=false — dev mode only.");
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(allowed_methods)
            .allow_headers(allowed_headers)
            .expose_headers([axum::http::header::HeaderName::from_static("x-csrf-token")])
    } else {
        let mut origin_values: Vec<String> = Vec::new();
        for raw in allowed_origin.split(',') {
            let item = raw.trim();
            if item.is_empty() {
                continue;
            }
            if !origin_values.iter().any(|v| v == item) {
                origin_values.push(item.to_string());
            }
        }

        if insecure_dev_mode {
            for origin in [
                "http://localhost:3005",
                "http://localhost:3006",
                "http://localhost:3007",
                "http://localhost:3008",
                "http://127.0.0.1:3005",
                "http://127.0.0.1:3006",
                "http://127.0.0.1:3007",
                "http://127.0.0.1:3008",
            ] {
                if !origin_values.iter().any(|v| v == origin) {
                    origin_values.push(origin.to_string());
                }
            }
        }

        if origin_values.is_empty() {
            return Err(anyhow::anyhow!("TENANT_CORS_ORIGIN must not be empty"));
        }

        let mut origins = Vec::new();
        for origin in origin_values {
            origins.push(
                HeaderValue::from_str(&origin)
                    .map_err(|_| anyhow::anyhow!("Invalid TENANT_CORS_ORIGIN value"))?,
            );
        }
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(allowed_methods)
            .allow_headers(allowed_headers)
            .expose_headers([axum::http::header::HeaderName::from_static("x-csrf-token")])
            .allow_credentials(true)
    };

    // ── Routes ───────────────────────────────────────────────────────
    let app_protected = Router::new()
        .route("/auth/me", get(routes::app::me))
        .route("/auth/logout", post(routes::app::logout))
        .route(
            "/auth/password-reset",
            post(routes::app::initiate_password_reset),
        )
        .route("/auth/sessions", get(routes::app::list_sessions))
        .route("/auth/sessions/:id", delete(routes::app::revoke_session))
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
        .route("/auth/totp/verify", post(routes::totp::verify_totp))
        .route("/auth/reset-password", post(routes::app::reset_password))
        .route("/auth/accept-invite", post(routes::invites::accept_invite))
        .route("/webhooks/stripe", post(routes::billing::stripe_webhook))
        .merge(app_protected);

    // ── Build full app with Swagger UI ───────────────────────────────
    let app = Router::new()
        .nest("/app", app_routes)
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()))
        .route(
            "/health",
            get(|| async {
                axum::Json(serde_json::json!({"status": "ok", "service": "tenant-service"}))
            }),
        )
        .layer(cors)
        .layer(DefaultBodyLimit::max(1024 * 1024))
        // Security headers
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::HeaderName::from_static("x-xss-protection"),
            HeaderValue::from_static("1; mode=block"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::CACHE_CONTROL,
            HeaderValue::from_static("no-store"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=63072000; includeSubDomains"),
        ))
        .layer(middleware::from_fn(
            enterprise_common::request_id::request_id_middleware,
        ))
        .with_state(state);

    Ok(app)
}
