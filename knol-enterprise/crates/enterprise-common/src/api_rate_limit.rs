//! Per-tenant API rate limiting.
//!
//! Sliding-window counter keyed by tenant UUID. Limits are determined by plan.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use uuid::Uuid;

/// Default requests per minute for tenants without a configured limit.
pub const DEFAULT_RATE_LIMIT: u32 = 100;

/// Sliding-window bucket for a single tenant.
struct Bucket {
    count: u32,
    window_start: Instant,
}

pub struct ApiRateLimiter {
    buckets: Mutex<HashMap<Uuid, Bucket>>,
    window_secs: u64,
}

impl ApiRateLimiter {
    pub fn new() -> Self {
        Self {
            buckets: Mutex::new(HashMap::new()),
            window_secs: 60,
        }
    }

    /// Check and increment the counter for a tenant. Returns `Err(retry_after_secs)` if blocked.
    pub fn check(&self, tenant_id: Uuid, limit: u32) -> Result<(), u64> {
        let mut map = self.buckets.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();

        // Clean up expired buckets periodically (every 100th call is fine)
        if map.len() > 1000 {
            map.retain(|_, b| now.duration_since(b.window_start).as_secs() < self.window_secs);
        }

        let bucket = map.entry(tenant_id).or_insert_with(|| Bucket {
            count: 0,
            window_start: now,
        });

        let elapsed = now.duration_since(bucket.window_start).as_secs();
        if elapsed >= self.window_secs {
            // Reset window
            bucket.count = 1;
            bucket.window_start = now;
            return Ok(());
        }

        if bucket.count >= limit {
            return Err(self.window_secs - elapsed);
        }

        bucket.count += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_under_limit_passes() {
        let limiter = ApiRateLimiter::new();
        let tenant = Uuid::new_v4();
        for _ in 0..99 {
            assert!(limiter.check(tenant, 100).is_ok());
        }
    }

    #[test]
    fn test_at_limit_blocks() {
        let limiter = ApiRateLimiter::new();
        let tenant = Uuid::new_v4();
        for _ in 0..100 {
            assert!(limiter.check(tenant, 100).is_ok());
        }
        assert!(limiter.check(tenant, 100).is_err());
    }

    #[test]
    fn test_different_tenants_independent() {
        let limiter = ApiRateLimiter::new();
        let t1 = Uuid::new_v4();
        let t2 = Uuid::new_v4();
        for _ in 0..100 {
            limiter.check(t1, 100).unwrap();
        }
        assert!(limiter.check(t1, 100).is_err());
        assert!(limiter.check(t2, 100).is_ok());
    }

    #[test]
    fn test_retry_after_value() {
        let limiter = ApiRateLimiter::new();
        let tenant = Uuid::new_v4();
        for _ in 0..100 {
            limiter.check(tenant, 100).unwrap();
        }
        if let Err(retry_after) = limiter.check(tenant, 100) {
            assert!(retry_after <= 60);
            assert!(retry_after >= 58); // allow 2 seconds tolerance
        }
    }
}
