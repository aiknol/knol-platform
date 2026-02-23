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
    if let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        if let Some(last) = xff.rsplit(',').next().map(|s| s.trim().to_string()) {
            if !last.is_empty() {
                return last;
            }
        }
    }
    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x_real_ip_preferred() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", "10.0.0.1".parse().unwrap());
        headers.insert(
            "x-forwarded-for",
            "192.168.1.1, 172.16.0.1".parse().unwrap(),
        );
        assert_eq!(extract_client_ip(&headers), "10.0.0.1");
    }

    #[test]
    fn test_xff_last_entry() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            "spoofed, proxy1, 203.0.113.5".parse().unwrap(),
        );
        assert_eq!(extract_client_ip(&headers), "203.0.113.5");
    }

    #[test]
    fn test_xff_single_entry() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "1.2.3.4".parse().unwrap());
        assert_eq!(extract_client_ip(&headers), "1.2.3.4");
    }

    #[test]
    fn test_no_headers_returns_unknown() {
        let headers = HeaderMap::new();
        assert_eq!(extract_client_ip(&headers), "unknown");
    }

    #[test]
    fn test_empty_x_real_ip_falls_through() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", "".parse().unwrap());
        headers.insert("x-forwarded-for", "5.6.7.8".parse().unwrap());
        assert_eq!(extract_client_ip(&headers), "5.6.7.8");
    }

    #[test]
    fn test_whitespace_trimmed() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", " 1.2.3.4 ".parse().unwrap());
        assert_eq!(extract_client_ip(&headers), "1.2.3.4");
    }

    #[test]
    fn test_xff_with_spaces() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            "10.0.0.1 , 192.168.1.1".parse().unwrap(),
        );
        assert_eq!(extract_client_ip(&headers), "192.168.1.1");
    }
}
