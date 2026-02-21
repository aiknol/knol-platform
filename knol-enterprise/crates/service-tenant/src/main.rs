//! Tenant Service
//!
//! Dedicated service for SaaS tenant operations: authentication,
//! billing (Stripe), team invites, settings, API keys, and usage tracking.
//! Includes OpenAPI/Swagger documentation at /docs.

pub mod auth;
mod openapi;
pub mod routes;
mod stripe;

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
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Shared state for the tenant service.
pub struct TenantAppState {
    pub db_pool: sqlx::PgPool,
    pub jwt_secret: String,
    pub http_client: reqwest::Client,
    /// Per-IP rate limiter for auth endpoints.
    pub rate_limiter: enterprise_common::rate_limit::RateLimiter,
    /// Stripe API secret key for billing integration.
    pub stripe_secret_key: Option<String>,
    /// Stripe webhook signing secret for signature verification.
    pub stripe_webhook_secret: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .json()
        .init();

    info!("Starting Tenant Service...");

    let database_url =
        std::env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;

    let jwt_secret = std::env::var("ADMIN_JWT_SECRET")
        .map_err(|_| anyhow::anyhow!("ADMIN_JWT_SECRET must be set"))?;
    if jwt_secret.len() < 32 {
        return Err(anyhow::anyhow!(
            "ADMIN_JWT_SECRET must be at least 32 characters"
        ));
    }

    let db_pool = memory_db::create_pool(&database_url, 6).await?;

    let port: u16 = memory_common::db_config::load_u64(
        &db_pool,
        "services.tenant_port",
        "TENANT_SERVICE_PORT",
        8085,
    )
    .await as u16;

    let stripe_secret_key = std::env::var("STRIPE_SECRET_KEY")
        .ok()
        .filter(|s| !s.is_empty());
    let stripe_webhook_secret = std::env::var("STRIPE_WEBHOOK_SECRET")
        .ok()
        .filter(|s| !s.is_empty());
    if stripe_secret_key.is_none() {
        warn!("STRIPE_SECRET_KEY not set — billing features disabled");
    }

    let state = Arc::new(TenantAppState {
        db_pool,
        jwt_secret,
        http_client: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?,
        rate_limiter: enterprise_common::rate_limit::new_rate_limiter(),
        stripe_secret_key,
        stripe_webhook_secret,
    });

    // ── CORS ─────────────────────────────────────────────────────────
    let allowed_origin = memory_common::db_config::load_str(
        &state.db_pool,
        "services.tenant_cors_origin",
        "TENANT_CORS_ORIGIN",
        "http://localhost:3005,http://localhost:3006,http://localhost:3007,http://localhost:3008",
    )
    .await;
    let allowed_origin = allowed_origin.trim();
    let allowed_methods = [
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::OPTIONS,
    ];
    let allowed_headers = [AUTHORIZATION, CONTENT_TYPE, ACCEPT, COOKIE];
    let insecure_dev_mode = std::env::var("ADMIN_SECURE_COOKIES")
        .map(|v| v == "false")
        .unwrap_or(false);

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
            .allow_credentials(true)
    };

    // ── Routes ───────────────────────────────────────────────────────
    let app_protected = Router::new()
        .route("/auth/me", get(routes::app::me))
        .route("/auth/logout", post(routes::app::logout))
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
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::app_auth_middleware,
        ));

    let app_routes = Router::new()
        .route("/auth/signup", post(routes::app::signup))
        .route("/auth/login", post(routes::app::login))
        .route(
            "/auth/accept-invite",
            post(routes::invites::accept_invite),
        )
        .route(
            "/webhooks/stripe",
            post(routes::billing::stripe_webhook),
        )
        .merge(app_protected);

    // ── Build full app with Swagger UI ───────────────────────────────
    let app = Router::new()
        .nest("/app", app_routes)
        .merge(
            SwaggerUi::new("/docs")
                .url("/api-docs/openapi.json", openapi::ApiDoc::openapi()),
        )
        .route(
            "/health",
            get(|| async {
                axum::Json(serde_json::json!({"status": "ok", "service": "tenant-service"}))
            }),
        )
        .layer(cors)
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Tenant service listening on {}", addr);
    info!("Swagger UI available at http://0.0.0.0:{}/docs", port);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
