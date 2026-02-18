//! Application state shared across all handlers and scheduler tasks.

use std::collections::HashMap;
use std::sync::Arc;

use crate::config::{ChannelConfig, ChannelCredentials};
use crate::rate_limiter::RateLimiter;

/// Shared application state.
pub struct AppState {
    pub db_pool: sqlx::PgPool,
    pub http_client: reqwest::Client,
    pub rate_limiter: RateLimiter,
    pub channel_configs: HashMap<String, ChannelConfig>,
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

        Ok(Arc::new(Self {
            db_pool,
            http_client,
            rate_limiter,
            channel_configs,
            credentials,
        }))
    }
}
