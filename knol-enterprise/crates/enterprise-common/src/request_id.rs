//! Request correlation ID middleware.
//!
//! Generates or propagates a unique `X-Request-ID` header for every request,
//! making it easy to trace requests across logs and services.

use axum::{
    extract::Request,
    http::HeaderValue,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

const REQUEST_ID_HEADER: &str = "x-request-id";

/// A unique identifier for each request, stored in request extensions.
#[derive(Clone, Debug)]
pub struct RequestId(pub Uuid);

/// Middleware that reads an incoming `X-Request-ID` header (if valid UUID)
/// or generates a new one, then echoes it back on the response.
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_else(Uuid::new_v4);

    request.extensions_mut().insert(RequestId(request_id));

    let mut response = next.run(request).await;
    if let Ok(val) = HeaderValue::from_str(&request_id.to_string()) {
        response.headers_mut().insert(REQUEST_ID_HEADER, val);
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id_clone() {
        let id = RequestId(Uuid::new_v4());
        let cloned = id.clone();
        assert_eq!(id.0, cloned.0);
    }
}
