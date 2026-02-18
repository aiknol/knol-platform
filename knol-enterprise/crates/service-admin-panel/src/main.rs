//! Admin Panel Microservice
//!
//! Lightweight API gateway for admin UI traffic.
//! Proxies `/admin/*` requests to the enterprise `admin-service`.

use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{any, get},
    Router,
};
use serde::Serialize;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

#[derive(Clone)]
struct AppState {
    client: reqwest::Client,
    admin_service_url: String,
}

#[derive(Serialize)]
struct HealthResponse<'a> {
    status: &'a str,
    service: &'a str,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .json()
        .init();

    let port: u16 = std::env::var("ADMIN_PANEL_SERVICE_PORT")
        .unwrap_or_else(|_| "8084".into())
        .parse()
        .unwrap_or(8084);

    let admin_service_url = std::env::var("ADMIN_SERVICE_URL")
        .unwrap_or_else(|_| "http://admin-service:3001".into());

    let state = Arc::new(AppState {
        client: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .build()?,
        admin_service_url,
    });

    let allowed_origin = std::env::var("ADMIN_CORS_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:3006".into());
    let cors = if allowed_origin == "*" {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let origin = HeaderValue::from_str(&allowed_origin)
            .map_err(|_| anyhow::anyhow!("Invalid ADMIN_CORS_ORIGIN value"))?;
        CorsLayer::new()
            .allow_origin(origin)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/admin/*path", any(proxy_admin))
        .route("/admin", any(proxy_admin_root))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Admin panel service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> impl IntoResponse {
    axum::Json(HealthResponse {
        status: "ok",
        service: "memory-admin-panel",
    })
}

async fn proxy_admin_root(
    State(state): State<Arc<AppState>>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: String,
) -> Response {
    proxy_request(state, method, uri, headers, body, "").await
}

async fn proxy_admin(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: String,
) -> Response {
    proxy_request(state, method, uri, headers, body, &path).await
}

async fn proxy_request(
    state: Arc<AppState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: String,
    path: &str,
) -> Response {
    let query = uri.query().map(|q| format!("?{}", q)).unwrap_or_default();
    let target = if path.is_empty() {
        format!("{}/admin{}", state.admin_service_url, query)
    } else {
        format!("{}/admin/{}{}", state.admin_service_url, path, query)
    };

    let mut req = state
        .client
        .request(method.clone(), &target)
        .body(body);

    for key in [
        "authorization",
        "content-type",
        "accept",
        "x-tenant-id",
        "x-user-id",
    ] {
        if let Some(val) = headers.get(key) {
            req = req.header(key, val);
        }
    }

    match req.send().await {
        Ok(resp) => {
            let status = resp.status();
            let content_type = resp
                .headers()
                .get("content-type")
                .cloned();
            let bytes = resp.bytes().await.unwrap_or_default();

            let mut out = Response::builder().status(status);
            if let Some(ct) = content_type {
                out = out.header("content-type", ct);
            }
            out.body(Body::from(bytes)).unwrap_or_else(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to build proxy response",
                )
                    .into_response()
            })
        }
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            format!("admin-service proxy error: {}", err),
        )
            .into_response(),
    }
}
