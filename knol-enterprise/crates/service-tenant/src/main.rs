use std::{net::SocketAddr, sync::Arc};
use tracing::{info, warn};

use service_tenant::TenantAppState;

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

    let idle_timeout_mins: i64 = std::env::var("IDLE_SESSION_TIMEOUT_MINS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);

    let totp_encryption_key = std::env::var("TOTP_ENCRYPTION_KEY")
        .ok()
        .filter(|s| !s.is_empty());

    let insecure_dev_mode = std::env::var("ADMIN_SECURE_COOKIES")
        .map(|v| v == "false")
        .unwrap_or(false);

    let state = Arc::new(TenantAppState {
        db_pool,
        jwt_secret,
        http_client: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?,
        rate_limiter: enterprise_common::rate_limit::new_rate_limiter(),
        api_rate_limiter: enterprise_common::api_rate_limit::ApiRateLimiter::new(),
        stripe_secret_key,
        stripe_webhook_secret,
        idle_timeout_mins,
        totp_encryption_key,
        secure_cookies: !insecure_dev_mode,
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
    if insecure_dev_mode {
        warn!("ADMIN_SECURE_COOKIES=false — running in insecure dev mode. Do NOT use in production!");
    }

    let app = service_tenant::build_router(state, allowed_origin, insecure_dev_mode)?;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Tenant service listening on {}", addr);
    info!("Swagger UI available at http://0.0.0.0:{}/docs", port);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
