//! Dynamic LLM provider that reloads configuration from the database.
//!
//! Instead of baking the API key and provider choice into the process at startup,
//! [`DynamicLlmProvider`] re-reads the admin DB on a configurable interval
//! (default: 60 seconds) so that changes to `llm.provider`, model, or API key
//! take effect without a service restart.
//!
//! The wrapper implements [`LlmProvider`] itself, so callers don't need to know
//! whether they're talking to a static or dynamic provider.

use async_trait::async_trait;
use memory_common::{ExtractionResult, ExtractedMemory, MemoryVerification};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::error::LlmError;
use crate::factory::build_provider_from_db;
use crate::provider::{ExtractionOptions, LlmProvider};
use crate::types::TokenUsage;

/// How often the provider configuration is checked against the DB.
const DEFAULT_REFRESH_INTERVAL_SECS: u64 = 60;

/// A provider wrapper that transparently refreshes itself from the database.
pub struct DynamicLlmProvider {
    pool: PgPool,
    inner: RwLock<InnerState>,
    refresh_interval: Duration,
}

struct InnerState {
    provider: Arc<dyn LlmProvider>,
    last_refresh: Instant,
    /// Cached for display; updated on each refresh.
    provider_name: String,
    model_name: String,
}

impl DynamicLlmProvider {
    /// Create a new dynamic provider.
    ///
    /// Performs an initial load from the DB (same as `build_provider_from_db`).
    /// Subsequent calls will refresh the underlying provider if the refresh
    /// interval has elapsed.
    pub async fn new(pool: PgPool) -> Result<Arc<Self>, LlmError> {
        Self::with_interval(pool, Duration::from_secs(DEFAULT_REFRESH_INTERVAL_SECS)).await
    }

    /// Create with a custom refresh interval.
    pub async fn with_interval(pool: PgPool, refresh_interval: Duration) -> Result<Arc<Self>, LlmError> {
        let provider = build_provider_from_db(&pool).await?;
        let provider_name = provider.provider_name().to_string();
        let model_name = provider.model_name().to_string();

        info!(
            "DynamicLlmProvider initialized: {} ({}) — refresh every {}s",
            provider_name,
            model_name,
            refresh_interval.as_secs()
        );

        Ok(Arc::new(Self {
            pool,
            inner: RwLock::new(InnerState {
                provider,
                last_refresh: Instant::now(),
                provider_name,
                model_name,
            }),
            refresh_interval,
        }))
    }

    /// Get a reference to the current inner provider, refreshing if the
    /// interval has elapsed. Returns the provider Arc (cheap clone).
    async fn get_provider(&self) -> Arc<dyn LlmProvider> {
        // Fast path: read lock — if not stale, return immediately
        {
            let state = self.inner.read().await;
            if state.last_refresh.elapsed() < self.refresh_interval {
                return state.provider.clone();
            }
        }

        // Slow path: acquire write lock and refresh
        let mut state = self.inner.write().await;

        // Double-check: another task may have refreshed while we were waiting
        if state.last_refresh.elapsed() < self.refresh_interval {
            return state.provider.clone();
        }

        match build_provider_from_db(&self.pool).await {
            Ok(new_provider) => {
                let new_name = new_provider.provider_name().to_string();
                let new_model = new_provider.model_name().to_string();

                if new_name != state.provider_name || new_model != state.model_name {
                    info!(
                        "LLM provider changed: {} ({}) → {} ({})",
                        state.provider_name, state.model_name, new_name, new_model
                    );
                }

                state.provider = new_provider;
                state.provider_name = new_name;
                state.model_name = new_model;
                state.last_refresh = Instant::now();
            }
            Err(e) => {
                warn!(
                    "Failed to refresh LLM provider from DB (keeping existing): {}",
                    e
                );
                // Still update the timestamp so we don't retry every call
                state.last_refresh = Instant::now();
            }
        }

        state.provider.clone()
    }
}

#[async_trait]
impl LlmProvider for DynamicLlmProvider {
    fn provider_name(&self) -> &str {
        // This is a synchronous method so we can't refresh here.
        // Return a static label; the actual provider name is logged on refresh.
        "dynamic"
    }

    fn model_name(&self) -> &str {
        "dynamic"
    }

    async fn extract_memories(
        &self,
        content: &str,
        role: &str,
        existing_entities: &[String],
    ) -> Result<ExtractionResult, LlmError> {
        let provider = self.get_provider().await;
        provider.extract_memories(content, role, existing_entities).await
    }

    async fn extract_memories_with_options(
        &self,
        content: &str,
        role: &str,
        existing_entities: &[String],
        options: &ExtractionOptions,
    ) -> Result<ExtractionResult, LlmError> {
        let provider = self.get_provider().await;
        provider
            .extract_memories_with_options(content, role, existing_entities, options)
            .await
    }

    async fn verify_memories(
        &self,
        memories: &[ExtractedMemory],
        source_content: &str,
    ) -> Result<Vec<MemoryVerification>, LlmError> {
        let provider = self.get_provider().await;
        provider.verify_memories(memories, source_content).await
    }

    async fn get_token_usage(&self) -> TokenUsage {
        let provider = self.get_provider().await;
        provider.get_token_usage().await
    }

    async fn reset_token_usage(&self) {
        let provider = self.get_provider().await;
        provider.reset_token_usage().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_refresh_interval() {
        assert_eq!(DEFAULT_REFRESH_INTERVAL_SECS, 60);
    }

    #[test]
    fn test_refresh_duration() {
        let dur = Duration::from_secs(DEFAULT_REFRESH_INTERVAL_SECS);
        assert_eq!(dur.as_secs(), 60);
    }
}
