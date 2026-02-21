//! Client IP extraction for rate limiting.

use axum::http::HeaderMap;

/// Extract client IP for rate limiting.
///
/// SECURITY: Prefer `x-real-ip` (set by the trusted reverse proxy like Caddy)
/// over `x-forwarded-for`. If using XFF, take the LAST entry (the one appended
/// by the trusted proxy) rather than the first (which is client-controlled and
/// can be spoofed to bypass per-IP rate limiting).
pub fn extract_client_ip(headers: &HeaderMap) -> String {
    // 1. X-Real-IP — set by the trusted reverse proxy, cannot be spoofed by client
    if let Some(real_ip) = headers
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        return real_ip;
    }
    // 2. X-Forwarded-For — take the LAST entry (added by the trusted proxy)
    if let Some(xff) = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
    {
        if let Some(last) = xff.rsplit(',').next().map(|s| s.trim().to_string()) {
            if !last.is_empty() {
                return last;
            }
        }
    }
    "unknown".to_string()
}
