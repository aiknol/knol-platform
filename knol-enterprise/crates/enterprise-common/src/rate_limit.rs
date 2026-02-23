//! In-memory per-IP rate limiting for auth endpoints.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Max failed attempts before lockout.
pub const AUTH_MAX_ATTEMPTS: u32 = 8;
/// Window duration in seconds.
pub const AUTH_WINDOW_SECS: u64 = 900;

pub type RateLimiter = Mutex<HashMap<String, (u32, Instant)>>;

/// Create a new rate limiter.
pub fn new_rate_limiter() -> RateLimiter {
    Mutex::new(HashMap::new())
}

/// Check whether the given key is rate-limited. Returns `Err(retry_after_secs)` if blocked.
pub fn enforce_rate_limit(limiter: &RateLimiter, key: &str, prefix: &str) -> Result<(), u64> {
    let mut map = limiter.lock().unwrap_or_else(|e| e.into_inner());
    let now = Instant::now();
    map.retain(|k, (_, first)| {
        if !k.starts_with(prefix) {
            return true;
        }
        now.duration_since(*first).as_secs() < AUTH_WINDOW_SECS
    });

    if let Some((count, first)) = map.get(key) {
        let elapsed = now.duration_since(*first).as_secs();
        if elapsed < AUTH_WINDOW_SECS && *count >= AUTH_MAX_ATTEMPTS {
            return Err(AUTH_WINDOW_SECS - elapsed);
        }
    }
    Ok(())
}

/// Record a failed auth attempt.
pub fn record_failure(limiter: &RateLimiter, key: &str) {
    let mut map = limiter.lock().unwrap_or_else(|e| e.into_inner());
    let now = Instant::now();
    let entry = map.entry(key.to_string()).or_insert((0, now));
    if now.duration_since(entry.1) >= Duration::from_secs(AUTH_WINDOW_SECS) {
        *entry = (1, now);
    } else {
        entry.0 += 1;
    }
}

/// Clear rate limit for a key (on successful login).
pub fn clear_limit(limiter: &RateLimiter, key: &str) {
    let mut map = limiter.lock().unwrap_or_else(|e| e.into_inner());
    map.remove(key);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(AUTH_MAX_ATTEMPTS, 8);
        assert_eq!(AUTH_WINDOW_SECS, 900);
    }

    #[test]
    fn test_new_limiter_allows_requests() {
        let limiter = new_rate_limiter();
        assert!(enforce_rate_limit(&limiter, "auth:test_ip", "auth:").is_ok());
    }

    #[test]
    fn test_under_limit_passes() {
        let limiter = new_rate_limiter();
        let key = "auth:1.2.3.4";
        for _ in 0..(AUTH_MAX_ATTEMPTS - 1) {
            record_failure(&limiter, key);
        }
        assert!(enforce_rate_limit(&limiter, key, "auth:").is_ok());
    }

    #[test]
    fn test_at_limit_blocks() {
        let limiter = new_rate_limiter();
        let key = "auth:1.2.3.4";
        for _ in 0..AUTH_MAX_ATTEMPTS {
            record_failure(&limiter, key);
        }
        assert!(enforce_rate_limit(&limiter, key, "auth:").is_err());
    }

    #[test]
    fn test_retry_after_returns_remaining_seconds() {
        let limiter = new_rate_limiter();
        let key = "auth:1.2.3.4";
        for _ in 0..AUTH_MAX_ATTEMPTS {
            record_failure(&limiter, key);
        }
        if let Err(retry_after) = enforce_rate_limit(&limiter, key, "auth:") {
            // Should be close to AUTH_WINDOW_SECS (within 2 seconds)
            assert!(retry_after <= AUTH_WINDOW_SECS);
            assert!(retry_after >= AUTH_WINDOW_SECS - 2);
        } else {
            panic!("Expected Err");
        }
    }

    #[test]
    fn test_clear_limit_resets() {
        let limiter = new_rate_limiter();
        let key = "auth:1.2.3.4";
        for _ in 0..AUTH_MAX_ATTEMPTS {
            record_failure(&limiter, key);
        }
        assert!(enforce_rate_limit(&limiter, key, "auth:").is_err());
        clear_limit(&limiter, key);
        assert!(enforce_rate_limit(&limiter, key, "auth:").is_ok());
    }

    #[test]
    fn test_different_keys_independent() {
        let limiter = new_rate_limiter();
        let key_a = "auth:1.1.1.1";
        let key_b = "auth:2.2.2.2";
        for _ in 0..AUTH_MAX_ATTEMPTS {
            record_failure(&limiter, key_a);
        }
        assert!(enforce_rate_limit(&limiter, key_a, "auth:").is_err());
        assert!(enforce_rate_limit(&limiter, key_b, "auth:").is_ok());
    }

    #[test]
    fn test_record_failure_increments() {
        let limiter = new_rate_limiter();
        let key = "auth:test";
        record_failure(&limiter, key);
        record_failure(&limiter, key);
        let map = limiter.lock().unwrap();
        let (count, _) = map.get(key).expect("key should exist");
        assert_eq!(*count, 2);
    }

    #[test]
    fn test_prefix_scoped_cleanup() {
        let limiter = new_rate_limiter();
        // Insert entries with different prefixes
        record_failure(&limiter, "auth:1.1.1.1");
        record_failure(&limiter, "other:2.2.2.2");
        // enforce with "auth:" prefix triggers cleanup of stale "auth:" entries
        // but "other:" entries should be preserved
        assert!(enforce_rate_limit(&limiter, "auth:3.3.3.3", "auth:").is_ok());
        let map = limiter.lock().unwrap();
        // "other:" key should still exist since it doesn't match the prefix
        assert!(map.contains_key("other:2.2.2.2"));
    }
}
