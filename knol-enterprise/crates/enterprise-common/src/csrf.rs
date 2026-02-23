//! CSRF protection via double-submit cookie pattern.
//!
//! A non-HttpOnly `csrf_token` cookie is set on login/signup. The frontend
//! reads it from `document.cookie` and sends it as the `X-CSRF-Token` header.
//! The middleware verifies that both values match on mutating requests.

use axum::http::HeaderMap;

const CSRF_COOKIE_NAME: &str = "csrf_token";
const CSRF_HEADER_NAME: &str = "x-csrf-token";

/// Generate a CSRF token string (64 hex chars = 32 bytes of randomness).
pub fn generate_csrf_token() -> String {
    use rand::Rng;
    let bytes: [u8; 32] = rand::thread_rng().gen();
    hex::encode(bytes)
}

/// Build the Set-Cookie value for the CSRF token (non-HttpOnly so JS can read it).
pub fn csrf_cookie(token: &str, secure: bool) -> String {
    format!(
        "csrf_token={}; SameSite=Lax; Path=/; Max-Age=86400{}",
        token,
        if secure { "; Secure" } else { "" }
    )
}

/// Build a Set-Cookie value to clear the CSRF cookie.
pub fn clear_csrf_cookie() -> String {
    "csrf_token=; SameSite=Lax; Path=/; Max-Age=0".to_string()
}

/// Verify that the `X-CSRF-Token` header matches the `csrf_token` cookie.
/// Returns `true` if valid, `false` if mismatch or missing.
pub fn verify_csrf(headers: &HeaderMap) -> bool {
    let cookie_token = extract_cookie_value(headers, CSRF_COOKIE_NAME);
    let header_token = headers
        .get(CSRF_HEADER_NAME)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    match (cookie_token, header_token) {
        (Some(c), Some(h)) if !c.is_empty() && c == h => true,
        _ => false,
    }
}

fn extract_cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|c| {
                let c = c.trim();
                c.strip_prefix(&format!("{}=", name)).map(|v| v.to_string())
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_csrf_token_length() {
        let token = generate_csrf_token();
        assert_eq!(token.len(), 64);
    }

    #[test]
    fn test_generate_csrf_token_unique() {
        let a = generate_csrf_token();
        let b = generate_csrf_token();
        assert_ne!(a, b);
    }

    #[test]
    fn test_csrf_cookie_format() {
        let cookie = csrf_cookie("abc123", false);
        assert!(cookie.contains("csrf_token=abc123"));
        assert!(cookie.contains("SameSite=Lax"));
        assert!(!cookie.contains("Secure"));

        let cookie_secure = csrf_cookie("abc123", true);
        assert!(cookie_secure.contains("; Secure"));
    }

    #[test]
    fn test_verify_csrf_matching() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::COOKIE,
            "csrf_token=test123".parse().unwrap(),
        );
        headers.insert("x-csrf-token", "test123".parse().unwrap());
        assert!(verify_csrf(&headers));
    }

    #[test]
    fn test_verify_csrf_mismatch() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::COOKIE,
            "csrf_token=test123".parse().unwrap(),
        );
        headers.insert("x-csrf-token", "wrong".parse().unwrap());
        assert!(!verify_csrf(&headers));
    }

    #[test]
    fn test_verify_csrf_missing_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::COOKIE,
            "csrf_token=test123".parse().unwrap(),
        );
        assert!(!verify_csrf(&headers));
    }

    #[test]
    fn test_verify_csrf_missing_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert("x-csrf-token", "test123".parse().unwrap());
        assert!(!verify_csrf(&headers));
    }
}
