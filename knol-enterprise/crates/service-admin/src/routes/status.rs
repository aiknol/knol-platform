//! System health dashboard — aggregate status of all services.

use axum::{extract::State, Json};
use std::sync::Arc;

use crate::auth::{AdminClaims, AdminError};
use crate::AdminAppState;

pub async fn system_status(
    State(state): State<Arc<AdminAppState>>,
    _claims: AdminClaims,
) -> Result<Json<serde_json::Value>, AdminError> {
    // Service health check URLs: resolve from env vars, falling back to
    // Docker Compose service names (which resolve via DNS in the compose network).
    let gateway_url =
        std::env::var("GATEWAY_HEALTH_URL").unwrap_or_else(|_| "http://gateway:8080/health".into());
    let write_url = std::env::var("WRITE_HEALTH_URL")
        .unwrap_or_else(|_| "http://write-service:8081/health".into());
    let retrieve_url = std::env::var("RETRIEVE_HEALTH_URL")
        .unwrap_or_else(|_| "http://retrieve-service:8082/health".into());
    let graph_url = std::env::var("GRAPH_HEALTH_URL")
        .unwrap_or_else(|_| "http://graph-service:8083/health".into());
    let jobs_url = std::env::var("JOBS_HEALTH_URL")
        .unwrap_or_else(|_| "http://jobs-service:8085/health".into());
    let billing_url = std::env::var("BILLING_HEALTH_URL")
        .unwrap_or_else(|_| "http://billing-service:8086/health".into());
    let ingest_url = std::env::var("INGEST_HEALTH_URL")
        .unwrap_or_else(|_| "http://ingest-service:8087/health".into());
    let marketing_url = std::env::var("MARKETING_SERVICE_URL")
        .map(|u| format!("{}/health", u))
        .unwrap_or_else(|_| "http://marketing-service:8088/health".into());

    let services = vec![
        ("gateway", gateway_url),
        ("write", write_url),
        ("retrieve", retrieve_url),
        ("graph", graph_url),
        ("jobs", jobs_url),
        ("billing", billing_url),
        ("ingest", ingest_url),
        ("marketing", marketing_url),
    ];

    let mut statuses = Vec::new();

    for (name, url) in &services {
        let health = check_service_health(&state.http_client, url.as_str()).await;
        statuses.push(serde_json::json!({
            "name": name,
            "status": health.0,
            "latency_ms": health.1,
            "error": health.2,
        }));
    }

    // Database status
    let db_status = check_db(&state.db_pool).await;

    // Config count
    let config_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM system_config")
        .fetch_one(&state.db_pool)
        .await
        .unwrap_or((0,));

    let credential_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM system_credentials")
        .fetch_one(&state.db_pool)
        .await
        .unwrap_or((0,));

    let tenant_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tenants")
        .fetch_one(&state.db_pool)
        .await
        .unwrap_or((0,));

    Ok(Json(serde_json::json!({
        "services": statuses,
        "database": db_status,
        "counts": {
            "configs": config_count.0,
            "credentials": credential_count.0,
            "tenants": tenant_count.0,
        },
    })))
}

async fn check_service_health(
    client: &reqwest::Client,
    url: &str,
) -> (String, u64, Option<String>) {
    let start = std::time::Instant::now();

    match client
        .get(url)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            ("healthy".into(), start.elapsed().as_millis() as u64, None)
        }
        Ok(resp) => (
            "unhealthy".into(),
            start.elapsed().as_millis() as u64,
            Some(format!("HTTP {}", resp.status())),
        ),
        Err(e) => {
            let msg = if e.is_connect() {
                "Connection refused".into()
            } else if e.is_timeout() {
                "Timeout".into()
            } else {
                e.to_string()
            };
            (
                "unreachable".into(),
                start.elapsed().as_millis() as u64,
                Some(msg),
            )
        }
    }
}

async fn check_db(pool: &sqlx::PgPool) -> serde_json::Value {
    let start = std::time::Instant::now();
    match sqlx::query_as::<_, (String,)>("SELECT version()")
        .fetch_one(pool)
        .await
    {
        Ok((version,)) => {
            let pool_size = pool.size();
            serde_json::json!({
                "status": "connected",
                "version": version,
                "pool_size": pool_size,
                "latency_ms": start.elapsed().as_millis(),
            })
        }
        Err(_) => serde_json::json!({
            "status": "error",
            "error": "Database connection failed",
            "latency_ms": start.elapsed().as_millis(),
        }),
    }
}
