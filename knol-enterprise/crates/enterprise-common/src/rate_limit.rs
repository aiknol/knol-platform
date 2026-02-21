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
