//! Memory Billing Service
//!
//! Usage metering, plan limit enforcement, and billing data aggregation.

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
    db_pool: sqlx::PgPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .json()
        .init();

    info!("Starting Memory Billing Service...");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://memory:memory_dev@localhost:5432/memory".into());

    let db_pool = memory_db::create_pool(&database_url, 4).await?;

    let port: u16 = memory_common::db_config::load_u64(
        &db_pool,
        "services.billing_port",
        "BILLING_SERVICE_PORT",
        8086,
    )
    .await as u16;

    let state = Arc::new(AppState { db_pool });

    let app = Router::new()
        .route("/internal/usage", get(get_usage))
        .route("/internal/usage/record", post(record_usage))
        .route("/internal/plan/check", get(check_plan_limits))
        .route("/internal/billing/reset-monthly", post(reset_monthly_usage))
        .route(
            "/health",
            get(|| async {
                axum::Json(serde_json::json!({"status": "ok", "service": "memory-billing"}))
            }),
        )
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Billing service listening on {}", addr);
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

async fn get_usage(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<UsageResponse>, memory_common::MemoryError> {
    let tenant_id = extract_tenant_id(&headers)?;

    let row = sqlx::query_as::<_, UsageRow>(
        "SELECT plan, usage_ops_month, usage_limit FROM tenants WHERE id = $1",
    )
    .bind(tenant_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;

    let memory_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM memories WHERE tenant_id = $1 AND status = 'active'",
    )
    .bind(tenant_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;

    let entity_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM entities WHERE tenant_id = $1 AND status = 'active'",
    )
    .bind(tenant_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;

    Ok(Json(UsageResponse {
        plan: row.plan,
        ops_this_month: row.usage_ops_month,
        ops_limit: row.usage_limit,
        active_memories: memory_count,
        active_entities: entity_count,
    }))
}

async fn record_usage(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<RecordUsageRequest>,
) -> Result<Json<serde_json::Value>, memory_common::MemoryError> {
    let tenant_id = extract_tenant_id(&headers)?;

    sqlx::query("UPDATE tenants SET usage_ops_month = usage_ops_month + $1 WHERE id = $2")
        .bind(body.ops_count)
        .bind(tenant_id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;

    Ok(Json(serde_json::json!({ "recorded": body.ops_count })))
}

async fn check_plan_limits(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<PlanLimitsResponse>, memory_common::MemoryError> {
    let tenant_id = extract_tenant_id(&headers)?;

    let row = sqlx::query_as::<_, UsageRow>(
        "SELECT plan, usage_ops_month, usage_limit FROM tenants WHERE id = $1",
    )
    .bind(tenant_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;

    let limits = get_plan_limits(&row.plan);
    let within_limits = row
        .usage_limit
        .map(|l| row.usage_ops_month < l)
        .unwrap_or(true);

    Ok(Json(PlanLimitsResponse {
        plan: row.plan,
        within_limits,
        ops_used: row.usage_ops_month,
        ops_limit: row.usage_limit,
        max_memories_per_user: limits.max_memories_per_user,
        max_users: limits.max_users,
        features: limits.features,
    }))
}

async fn reset_monthly_usage(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, memory_common::MemoryError> {
    let result = sqlx::query("UPDATE tenants SET usage_ops_month = 0")
        .execute(&state.db_pool)
        .await
        .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;

    info!("Reset monthly usage for {} tenants", result.rows_affected());
    Ok(Json(serde_json::json!({ "reset": result.rows_affected() })))
}

fn get_plan_limits(plan: &str) -> PlanLimits {
    match plan {
        "free" => PlanLimits {
            max_memories_per_user: 1000,
            max_users: 1,
            features: vec!["vector_search".into(), "basic_graph".into()],
        },
        "developer" => PlanLimits {
            max_memories_per_user: 10_000,
            max_users: 5,
            features: vec!["vector_search".into(), "graph".into(), "temporal".into()],
        },
        "pro" => PlanLimits {
            max_memories_per_user: 100_000,
            max_users: 25,
            features: vec![
                "vector_search".into(),
                "graph".into(),
                "temporal".into(),
                "simulation".into(),
                "connectors".into(),
            ],
        },
        "team" => PlanLimits {
            max_memories_per_user: 500_000,
            max_users: 100,
            features: vec![
                "vector_search".into(),
                "graph".into(),
                "temporal".into(),
                "simulation".into(),
                "connectors".into(),
                "audit".into(),
                "sso".into(),
            ],
        },
        "enterprise" => PlanLimits {
            max_memories_per_user: i64::MAX,
            max_users: i64::MAX,
            features: vec![
                "vector_search".into(),
                "graph".into(),
                "temporal".into(),
                "simulation".into(),
                "connectors".into(),
                "audit".into(),
                "sso".into(),
                "air_gapped".into(),
                "custom_ontology".into(),
                "dedicated_infra".into(),
            ],
        },
        _ => get_plan_limits("free"),
    }
}

// ── Types ──

#[derive(Debug, sqlx::FromRow)]
struct UsageRow {
    plan: String,
    usage_ops_month: i32,
    usage_limit: Option<i32>,
}

#[derive(Debug, Serialize)]
struct UsageResponse {
    plan: String,
    ops_this_month: i32,
    ops_limit: Option<i32>,
    active_memories: i64,
    active_entities: i64,
}

#[derive(Debug, Deserialize)]
struct RecordUsageRequest {
    ops_count: i32,
}

#[derive(Debug, Serialize)]
struct PlanLimitsResponse {
    plan: String,
    within_limits: bool,
    ops_used: i32,
    ops_limit: Option<i32>,
    max_memories_per_user: i64,
    max_users: i64,
    features: Vec<String>,
}

struct PlanLimits {
    max_memories_per_user: i64,
    max_users: i64,
    features: Vec<String>,
}
