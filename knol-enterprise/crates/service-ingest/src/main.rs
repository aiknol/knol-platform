//! Memory Ingest Service
//!
//! Connector framework for ingesting data from external sources
//! (Slack, GitHub, email, etc.) into the memory pipeline.

use axum::{
    extract::{Json, State},
    http::HeaderMap,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tracing::info;
use uuid::Uuid;

struct AppState {
    nats_js: async_nats::jetstream::Context,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .json()
        .init();

    info!("Starting Memory Ingest Service...");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://memory:memory_dev@localhost:5432/memory".into());
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".into());

    let db_pool = memory_db::create_pool(&database_url, 4).await?;

    let port: u16 = memory_common::db_config::load_u64(
        &db_pool,
        "services.ingest_port",
        "INGEST_SERVICE_PORT",
        8087,
    )
    .await as u16;
    let (_nats_client, nats_js) = memory_queue::connect(&nats_url).await?;
    memory_queue::ensure_stream(&nats_js).await?;

    let state = Arc::new(AppState { nats_js });

    let app = Router::new()
        .route("/internal/connectors", get(list_connectors))
        .route("/internal/connectors/webhook", post(webhook_ingest))
        .route("/internal/connectors/bulk", post(bulk_ingest))
        .route(
            "/health",
            get(|| async {
                axum::Json(serde_json::json!({"status": "ok", "service": "memory-ingest"}))
            }),
        )
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Ingest service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn extract_tenant_id(headers: &HeaderMap) -> Result<Uuid, memory_common::MemoryError> {
    headers
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| memory_common::MemoryError::Auth("Missing x-tenant-id".into()))
}

/// List available connectors and their status.
async fn list_connectors() -> Json<Vec<ConnectorInfo>> {
    Json(vec![
        ConnectorInfo {
            id: "webhook".into(),
            name: "Webhook".into(),
            description: "Generic webhook connector for custom integrations".into(),
            status: "available".into(),
        },
        ConnectorInfo {
            id: "slack".into(),
            name: "Slack".into(),
            description: "Ingest messages from Slack channels".into(),
            status: "coming_soon".into(),
        },
        ConnectorInfo {
            id: "github".into(),
            name: "GitHub".into(),
            description: "Ingest issues, PRs, and discussions from GitHub".into(),
            status: "coming_soon".into(),
        },
        ConnectorInfo {
            id: "email".into(),
            name: "Email".into(),
            description: "Ingest emails via IMAP or webhook".into(),
            status: "coming_soon".into(),
        },
    ])
}

/// Generic webhook ingestion endpoint.
async fn webhook_ingest(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<WebhookPayload>,
) -> Result<Json<serde_json::Value>, memory_common::MemoryError> {
    let tenant_id = extract_tenant_id(&headers)?;

    let mut ingested = 0;
    for item in &body.items {
        let event = memory_common::MemoryWriteEvent {
            episode_id: uuid::Uuid::new_v4(),
            tenant_id,
            user_id: item.user_id,
            content: item.content.clone(),
            role: item.role.clone().unwrap_or_else(|| "system".into()),
            session_id: item.session_id.clone(),
            agent_id: item.agent_id.clone(),
            metadata: item.metadata.clone().unwrap_or(serde_json::json!({})),
            timestamp: chrono::Utc::now(),
        };

        if let Err(e) =
            memory_queue::publish(&state.nats_js, memory_queue::SUBJECT_WRITE, &event).await
        {
            tracing::error!("Failed to publish ingest event: {}", e);
        } else {
            ingested += 1;
        }
    }

    Ok(Json(serde_json::json!({
        "ingested": ingested,
        "total": body.items.len(),
        "source": body.source,
    })))
}

/// Bulk text ingestion (for importing documents, chat histories, etc.).
async fn bulk_ingest(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<BulkIngestRequest>,
) -> Result<Json<serde_json::Value>, memory_common::MemoryError> {
    let tenant_id = extract_tenant_id(&headers)?;
    let mut ingested = 0;

    for text in &body.texts {
        let event = memory_common::MemoryWriteEvent {
            episode_id: uuid::Uuid::new_v4(),
            tenant_id,
            user_id: body.user_id,
            content: text.clone(),
            role: "system".into(),
            session_id: body.session_id.clone(),
            agent_id: body.agent_id.clone(),
            metadata: body.metadata.clone().unwrap_or(serde_json::json!({})),
            timestamp: chrono::Utc::now(),
        };

        if let Err(e) =
            memory_queue::publish(&state.nats_js, memory_queue::SUBJECT_WRITE, &event).await
        {
            tracing::error!("Failed to publish bulk event: {}", e);
        } else {
            ingested += 1;
        }
    }

    Ok(Json(serde_json::json!({
        "ingested": ingested,
        "total": body.texts.len(),
    })))
}

// ── Types ──

#[derive(Debug, Serialize)]
struct ConnectorInfo {
    id: String,
    name: String,
    description: String,
    status: String,
}

#[derive(Debug, Deserialize)]
struct WebhookPayload {
    source: String,
    items: Vec<WebhookItem>,
}

#[derive(Debug, Deserialize)]
struct WebhookItem {
    content: String,
    user_id: Option<Uuid>,
    role: Option<String>,
    session_id: Option<String>,
    agent_id: Option<String>,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct BulkIngestRequest {
    texts: Vec<String>,
    user_id: Option<Uuid>,
    session_id: Option<String>,
    agent_id: Option<String>,
    metadata: Option<serde_json::Value>,
}
