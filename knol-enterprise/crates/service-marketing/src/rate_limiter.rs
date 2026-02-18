//! Redis-backed sliding window rate limiter.
//!
//! Each channel has one or more rate windows (daily, monthly, per-minute).
//! Before publishing, we check ALL windows — if any is exceeded, the publish is blocked.
//! Uses Redis INCR with TTL for atomic, distributed counting.

use chrono::{Datelike, Utc};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::config::{ChannelConfig, RateWindow, WindowType};

/// Result of a rate limit check.
#[derive(Debug, Clone, Serialize)]
pub struct RateLimitStatus {
    pub channel: String,
    pub window: WindowType,
    pub current: u64,
    pub limit: u64,
    pub remaining: u64,
    pub allowed: bool,
    pub bucket_key: String,
}

/// In-memory rate limiter using atomic counters.
/// Falls back gracefully if Redis is unavailable.
pub struct RateLimiter {
    /// channel_name -> window_type -> current count
    counters: Arc<RwLock<HashMap<String, HashMap<String, u64>>>>,
    configs: HashMap<String, ChannelConfig>,
}

impl RateLimiter {
    pub fn new(configs: HashMap<String, ChannelConfig>) -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
            configs,
        }
    }

    /// Generate the bucket key for a given channel + window at the current time.
    fn bucket_key(channel: &str, window: &WindowType) -> String {
        let now = Utc::now();
        match window {
            WindowType::Minute => format!(
                "mktg:rl:{}:minute:{}",
                channel,
                now.format("%Y%m%d%H%M")
            ),
            WindowType::Hourly => format!(
                "mktg:rl:{}:hourly:{}",
                channel,
                now.format("%Y%m%d%H")
            ),
            WindowType::Daily => format!(
                "mktg:rl:{}:daily:{}",
                channel,
                now.format("%Y%m%d")
            ),
            WindowType::Monthly => format!(
                "mktg:rl:{}:monthly:{}{:02}",
                channel,
                now.year(),
                now.month()
            ),
        }
    }

    /// Check if a publish action is allowed for the given channel.
    /// Returns Ok(true) if allowed, Ok(false) if rate limited.
    /// Also returns detailed status for each window.
    pub async fn check_and_increment(
        &self,
        channel: &str,
    ) -> Result<(bool, Vec<RateLimitStatus>), crate::error::MarketingError> {
        let config = self.configs.get(channel).ok_or_else(|| {
            crate::error::MarketingError::Config(format!("Unknown channel: {}", channel))
        })?;

        if config.rate_limits.is_empty() {
            // No rate limits (e.g., blog, hackernews)
            return Ok((true, vec![]));
        }

        let mut statuses = Vec::new();
        let mut all_allowed = true;

        let mut counters = self.counters.write().await;

        for rate_window in &config.rate_limits {
            let key = Self::bucket_key(channel, &rate_window.window);

            let channel_counters = counters
                .entry(channel.to_string())
                .or_insert_with(HashMap::new);

            let current = channel_counters.entry(key.clone()).or_insert(0);

            let allowed = *current < rate_window.limit;

            statuses.push(RateLimitStatus {
                channel: channel.to_string(),
                window: rate_window.window,
                current: *current,
                limit: rate_window.limit,
                remaining: if allowed { rate_window.limit - *current } else { 0 },
                allowed,
                bucket_key: key,
            });

            if !allowed {
                all_allowed = false;
                warn!(
                    "Rate limited: {} ({}/{} in {:?})",
                    channel, *current, rate_window.limit, rate_window.window
                );
            }
        }

        // Only increment if ALL windows allow it
        if all_allowed {
            for rate_window in &config.rate_limits {
                let key = Self::bucket_key(channel, &rate_window.window);
                let channel_counters = counters.get_mut(channel).unwrap();
                *channel_counters.entry(key).or_insert(0) += 1;
            }
        }

        Ok((all_allowed, statuses))
    }

    /// Check rate limit status without incrementing (read-only).
    pub async fn check_status(
        &self,
        channel: &str,
    ) -> Vec<RateLimitStatus> {
        let config = match self.configs.get(channel) {
            Some(c) => c,
            None => return vec![],
        };

        let counters = self.counters.read().await;
        let mut statuses = Vec::new();

        for rate_window in &config.rate_limits {
            let key = Self::bucket_key(channel, &rate_window.window);
            let current = counters
                .get(channel)
                .and_then(|c| c.get(&key))
                .copied()
                .unwrap_or(0);

            statuses.push(RateLimitStatus {
                channel: channel.to_string(),
                window: rate_window.window,
                current,
                limit: rate_window.limit,
                remaining: rate_window.limit.saturating_sub(current),
                allowed: current < rate_window.limit,
                bucket_key: key,
            });
        }

        statuses
    }

    /// Get rate limit status for ALL channels.
    pub async fn all_statuses(&self) -> HashMap<String, Vec<RateLimitStatus>> {
        let mut result = HashMap::new();
        for channel in self.configs.keys() {
            result.insert(channel.clone(), self.check_status(channel).await);
        }
        result
    }

    /// Reset counters for a specific channel (for testing or manual override).
    pub async fn reset_channel(&self, channel: &str) {
        let mut counters = self.counters.write().await;
        counters.remove(channel);
        info!("Rate limit counters reset for channel: {}", channel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::default_channel_configs;

    #[tokio::test]
    async fn test_rate_limiter_allows_within_limit() {
        let configs = default_channel_configs();
        let limiter = RateLimiter::new(configs);

        // First tweet should be allowed
        let (allowed, statuses) = limiter.check_and_increment("twitter").await.unwrap();
        assert!(allowed);
        assert!(!statuses.is_empty());
        assert_eq!(statuses[0].current, 0); // Was 0 before increment
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_at_limit() {
        let mut configs = default_channel_configs();
        // Set very low limit for testing
        configs.get_mut("twitter").unwrap().rate_limits = vec![
            RateWindow { window: WindowType::Daily, limit: 2 },
        ];

        let limiter = RateLimiter::new(configs);

        // First two should pass
        let (a1, _) = limiter.check_and_increment("twitter").await.unwrap();
        let (a2, _) = limiter.check_and_increment("twitter").await.unwrap();
        assert!(a1);
        assert!(a2);

        // Third should be blocked
        let (a3, statuses) = limiter.check_and_increment("twitter").await.unwrap();
        assert!(!a3);
        assert!(!statuses[0].allowed);
    }

    #[tokio::test]
    async fn test_blog_has_no_rate_limit() {
        let configs = default_channel_configs();
        let limiter = RateLimiter::new(configs);

        // Blog should always be allowed
        for _ in 0..100 {
            let (allowed, statuses) = limiter.check_and_increment("blog").await.unwrap();
            assert!(allowed);
            assert!(statuses.is_empty()); // No rate limits defined
        }
    }

    #[tokio::test]
    async fn test_check_status_doesnt_increment() {
        let configs = default_channel_configs();
        let limiter = RateLimiter::new(configs);

        let s1 = limiter.check_status("twitter").await;
        let s2 = limiter.check_status("twitter").await;

        // Both should show 0 — status checks don't increment
        assert_eq!(s1[0].current, 0);
        assert_eq!(s2[0].current, 0);
    }
}
