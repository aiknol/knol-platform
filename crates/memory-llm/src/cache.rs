//! LLM extraction result cache.
//!
//! Uses SHA-256(content + role + entity_names_hash) as a cache key.
//! When a cache hit occurs, the full LLM call is skipped, saving
//! both latency and token cost.

use memory_common::ExtractionResult;
use sha2::{Digest, Sha256};
use std::time::Duration;
use tracing::{debug, warn};

/// Cache configuration.
#[derive(Debug, Clone)]
pub struct LlmCacheConfig {
    /// Enable the LLM response cache.
    pub enabled: bool,
    /// TTL for cached extraction results.
    pub ttl_secs: u64,
}

impl Default for LlmCacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl_secs: 3600, // 1 hour
        }
    }
}

/// Compute the cache key for an extraction request.
///
/// The key is a hex-encoded SHA-256 digest of the content, role, and
/// a sorted hash of entity names. This ensures that two identical
/// requests with the same entity context produce the same key.
pub fn cache_key(content: &str, role: &str, entity_names: &[String]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hasher.update(b"|role:");
    hasher.update(role.as_bytes());
    hasher.update(b"|entities:");
    // Sort entity names for deterministic key regardless of input order
    let mut sorted = entity_names.to_vec();
    sorted.sort();
    for name in &sorted {
        hasher.update(name.as_bytes());
        hasher.update(b",");
    }
    let hash = hasher.finalize();
    format!("llm_cache:{:x}", hash)
}

/// Try to retrieve a cached extraction result from Redis.
pub async fn get_cached(redis: &fred::prelude::RedisClient, key: &str) -> Option<ExtractionResult> {
    match memory_cache::get::<ExtractionResult>(redis, key).await {
        Ok(Some(result)) => {
            debug!("LLM cache HIT: {}", key);
            Some(result)
        }
        Ok(None) => None,
        Err(e) => {
            warn!("LLM cache get error (proceeding without cache): {}", e);
            None
        }
    }
}

/// Store an extraction result in the Redis cache.
pub async fn set_cached(
    redis: &fred::prelude::RedisClient,
    key: &str,
    result: &ExtractionResult,
    ttl_secs: u64,
) {
    let ttl = Duration::from_secs(ttl_secs);
    if let Err(e) = memory_cache::set(redis, key, result, ttl).await {
        warn!("LLM cache set error (non-fatal): {}", e);
    } else {
        debug!("LLM cache SET: {} (ttl={}s)", key, ttl_secs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_deterministic() {
        let k1 = cache_key("hello", "user", &["Alice".into(), "Bob".into()]);
        let k2 = cache_key("hello", "user", &["Alice".into(), "Bob".into()]);
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_cache_key_entity_order_independent() {
        let k1 = cache_key("hello", "user", &["Alice".into(), "Bob".into()]);
        let k2 = cache_key("hello", "user", &["Bob".into(), "Alice".into()]);
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_cache_key_different_content() {
        let k1 = cache_key("hello", "user", &[]);
        let k2 = cache_key("world", "user", &[]);
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_cache_key_different_role() {
        let k1 = cache_key("hello", "user", &[]);
        let k2 = cache_key("hello", "assistant", &[]);
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_cache_key_different_entities() {
        let k1 = cache_key("hello", "user", &["Alice".into()]);
        let k2 = cache_key("hello", "user", &["Bob".into()]);
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_cache_key_prefix() {
        let key = cache_key("test", "user", &[]);
        assert!(key.starts_with("llm_cache:"));
    }

    #[test]
    fn test_default_config() {
        let config = LlmCacheConfig::default();
        assert!(config.enabled);
        assert_eq!(config.ttl_secs, 3600);
    }
}
