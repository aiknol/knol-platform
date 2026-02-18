//! System health dashboard — aggregate status of all services.

use axum::{extract::State, Json};
use std::sync::Arc;

use crate::auth::{AdminClaims, AdminError};
use crate::AdminAppState;

pub async fn system_status(
    State(state): State<Arc<AdminAppState>>,
    _claims: AdminClaims,
) -> Result<Json<serde_json::Value>, AdminError> {
    let services = vec![
        ("gateway", "http://localhost:8080/health"),
        ("write", "http://localhost:8081/health"),
        ("retrieve", "http://localhost:8082/health"),
        ("graph", "http://localhost:8083/health"),
        ("admin", "http://localhost:8084/health"),
        ("jobs", "http://localhost:8085/health"),
        ("billing", "http://localhost:8086/health"),
        ("ingest", "http://localhost:8087/health"),
        ("marketing", "http://localhost:8088/health"),
    ];

    let mut statuses = Vec::new();

    for (name, url) in &services {
        let health = check_service_health(&state.http_client, url).await;
        statuses.push(serde_json::json!({
            "name": name,
            "url": url,
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

    match client.get(url).timeout(std::time::Duration::from_secs(3)).send().await {
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
            ("unreachable".into(), start.elapsed().as_millis() as u64, Some(msg))
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
        Err(e) => serde_json::json!({
            "status": "error",
            "error": e.to_string(),
            "latency_ms": start.elapsed().as_millis(),
        }),
    }
}
