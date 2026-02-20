//! Application state shared across all handlers and scheduler tasks.

use std::sync::Arc;
use tracing::info;

use crate::config::ChannelCredentials;
use crate::rate_limiter::RateLimiter;

/// Shared application state.
pub struct AppState {
    pub db_pool: sqlx::PgPool,
    pub http_client: reqwest::Client,
    pub rate_limiter: RateLimiter,
    pub credentials: ChannelCredentials,
}

impl AppState {
    pub async fn from_env() -> anyhow::Result<Arc<Self>> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://memory:memory_dev@localhost:5432/memory".into());

        let db_pool = memory_db::create_pool(&database_url, 4).await?;

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("knol-marketing/0.1.0")
            .build()?;

        let channel_configs = crate::config::default_channel_configs();
        let rate_limiter = RateLimiter::new(channel_configs.clone());

        // Load credentials from DB (encrypted) → env var → None fallback chain
        let credentials = crate::config_loader::load_credentials(&db_pool).await;
        info!(
            "Marketing channels configured: twitter={} linkedin={} reddit={} devto={} hashnode={} medium={} producthunt={} github={} email={} anthropic={}",
            credentials.has_twitter(),
            credentials.has_linkedin(),
            credentials.has_reddit(),
            credentials.has_devto(),
            credentials.has_hashnode(),
            credentials.has_medium(),
            credentials.has_producthunt(),
            credentials.has_github(),
            credentials.has_email(),
            credentials.has_anthropic(),
        );

        Ok(Arc::new(Self {
            db_pool,
            http_client,
            rate_limiter,
            credentials,
        }))
    }
}
