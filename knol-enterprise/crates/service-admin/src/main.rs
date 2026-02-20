//! Memory Admin Service
//!
//! CRUD operations, governance rules, audit log browsing,
//! retention policy management, memory simulation/replay,
//! and the admin panel API for system configuration.

mod auth;
mod crypto;
mod routes;

use axum::extract::DefaultBodyLimit;
use axum::{
    extract::{Json, Path, Query, State},
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, COOKIE},
        HeaderMap, HeaderValue, Method,
    },
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};
use uuid::Uuid;

/// Shared state for the admin service.
pub struct AdminAppState {
    pub db_pool: sqlx::PgPool,
    pub jwt_secret: String,
    pub encryption_key: [u8; 32],
    pub http_client: reqwest::Client,
    /// Per-IP login rate limiter: tracks (attempt_count, first_attempt_time).
    pub login_rate_limiter:
        std::sync::Mutex<std::collections::HashMap<String, (u32, std::time::Instant)>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .json()
        .init();

    info!("Starting Memory Admin Service...");

    let database_url =
        std::env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;

    let jwt_secret = std::env::var("ADMIN_JWT_SECRET")
        .map_err(|_| anyhow::anyhow!("ADMIN_JWT_SECRET must be set"))?;
    if jwt_secret.len() < 32 {
        return Err(anyhow::anyhow!(
            "ADMIN_JWT_SECRET must be at least 32 characters"
        ));
    }
    let encryption_key = crypto::load_encryption_key()
        .map_err(|e| anyhow::anyhow!("Encryption key error: {}", e))?;

    let db_pool = memory_db::create_pool(&database_url, 6).await?;

    let port: u16 = memory_common::db_config::load_u64(
        &db_pool,
        "services.admin_port",
        "ADMIN_SERVICE_PORT",
        8084,
    )
    .await as u16;

    // Seed initial admin user if none exists
    auth::seed_initial_admin(&db_pool).await?;

    let state = Arc::new(AdminAppState {
        db_pool,
        jwt_secret,
        encryption_key,
        http_client: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?,
        login_rate_limiter: std::sync::Mutex::new(std::collections::HashMap::new()),
    });

    // CORS for admin/dashboard + local demos: DB → env → default.
    let allowed_origin = memory_common::db_config::load_str(
        &state.db_pool,
        "services.admin_cors_origin",
        "ADMIN_CORS_ORIGIN",
        "http://localhost:3006,http://localhost:3005,http://localhost:8080",
    )
    .await;
    let allowed_origin = allowed_origin.trim();
    let allowed_methods = [
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::OPTIONS,
    ];
    let allowed_headers = [AUTHORIZATION, CONTENT_TYPE, ACCEPT, COOKIE];
    // SECURITY: When using HttpOnly cookies (`credentials: 'include`), CORS
    // cannot use a wildcard origin. We always require explicit origins.
    let cors = if allowed_origin == "*" {
        let is_dev = std::env::var("ADMIN_SECURE_COOKIES")
            .map(|v| v == "false")
            .unwrap_or(false);
        if !is_dev {
            return Err(anyhow::anyhow!(
                "ADMIN_CORS_ORIGIN='*' is not allowed in production (credential cookies require explicit origins). \
                 Set ADMIN_CORS_ORIGIN to your frontend URL, or set ADMIN_SECURE_COOKIES=false for local dev."
            ));
        }
        warn!("ADMIN_CORS_ORIGIN='*' with ADMIN_SECURE_COOKIES=false — dev mode only.");
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(allowed_methods)
            .allow_headers(allowed_headers)
    } else {
        let mut origins = Vec::new();
        for raw in allowed_origin.split(',') {
            let item = raw.trim();
            if item.is_empty() {
                continue;
            }
            origins.push(
                HeaderValue::from_str(item)
                    .map_err(|_| anyhow::anyhow!("Invalid ADMIN_CORS_ORIGIN value"))?,
            );
        }
        if origins.is_empty() {
            return Err(anyhow::anyhow!("ADMIN_CORS_ORIGIN must not be empty"));
        }
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(allowed_methods)
            .allow_headers(allowed_headers)
            .allow_credentials(true)
    };

    // ── Admin panel routes (JWT-protected) ───────────────────────
    let admin_protected = Router::new()
        .route("/auth/logout", post(auth::logout))
        .route("/auth/change-password", post(auth::change_password))
        // Config
        .route("/config", get(routes::config::list_configs))
        .route("/config/:key", get(routes::config::get_config))
        .route("/config/:key", put(routes::config::upsert_config))
        .route("/config/:key", delete(routes::config::delete_config))
        // Credentials
        .route("/credentials", get(routes::credentials::list_credentials))
        .route(
            "/credentials/:name",
            put(routes::credentials::upsert_credential),
        )
        .route(
            "/credentials/:name",
            delete(routes::credentials::delete_credential),
        )
        .route(
            "/credentials/:name/test",
            post(routes::credentials::test_credential),
        )
        // Campaigns
        .route("/campaigns", get(routes::campaigns::list_campaigns))
        .route("/campaigns/:name", put(routes::campaigns::update_campaign))
        .route(
            "/campaigns/:name/logs",
            get(routes::campaigns::campaign_logs),
        )
        .route(
            "/campaigns/:name/trigger",
            post(routes::campaigns::trigger_campaign),
        )
        // Marketing stats & metrics
        .route("/marketing/stats", get(routes::campaigns::marketing_stats))
        .route("/marketing/metrics", post(routes::campaigns::record_metric))
        // Tenants
        .route("/tenants", get(routes::tenants::list_tenants))
        .route("/tenants/:id", get(routes::tenants::get_tenant))
        .route("/tenants/:id", put(routes::tenants::update_tenant))
        // System status
        .route("/status", get(routes::status::system_status))
        // Users
        .route("/users", get(routes::users::list_users))
        .route("/users", post(routes::users::create_user))
        .route("/users/:id", put(routes::users::update_user))
        .route("/users/:id", delete(routes::users::delete_user))
        // Audit
        .route("/audit", get(routes::audit::list_audit))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::admin_auth_middleware,
        ));

    // Admin routes: login + demo endpoints are public, everything else is protected
    let admin_routes = Router::new()
        .route("/auth/login", post(auth::login))
        .route("/demo/config", get(routes::demo::demo_config))
        .route("/demo/extract", post(routes::demo::demo_extract))
        .merge(admin_protected);

    // Build full app
    let app = Router::new()
        // Legacy internal routes (tenant-authenticated)
        .route("/internal/memory/:id", put(update_memory))
        .route("/internal/memory/:id", delete(delete_memory))
        .route("/internal/memory/merge", post(merge_memories))
        .route("/internal/audit", get(list_audit))
        .route("/internal/policies", get(list_policies))
        .route("/internal/policies", post(create_policy))
        .route("/internal/simulate/replay", post(simulate_replay))
        // Admin panel API
        .nest("/admin", admin_routes)
        // Health
        .route(
            "/health",
            get(|| async {
                axum::Json(serde_json::json!({"status": "ok", "service": "memory-admin"}))
            }),
        )
        .layer(cors)
        // SECURITY: Limit request body size to 1MB to prevent DoS via large payloads.
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Admin service listening on {}", addr);
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

// ── Memory CRUD ──

async fn update_memory(
    State(state): State<Arc<AdminAppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateMemoryRequest>,
) -> Result<Json<serde_json::Value>, memory_common::MemoryError> {
    let tenant_id = extract_tenant_id(&headers)?;
    let mut tx = memory_db::begin_tenant_tx(&state.db_pool, tenant_id)
        .await
        .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;

    let current = sqlx::query_as::<_, MemoryRow>(
        "SELECT * FROM memories WHERE id = $1 AND status = 'active'",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?
    .ok_or_else(|| memory_common::MemoryError::NotFound(format!("Memory {} not found", id)))?;

    if let Some(content) = &body.content {
        sqlx::query("UPDATE memories SET content = $1, updated_at = now() WHERE id = $2")
            .bind(content)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;
    }
    if let Some(status) = &body.status {
        sqlx::query("UPDATE memories SET status = $1, updated_at = now() WHERE id = $2")
            .bind(status)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;
    }
    if let Some(importance) = body.importance {
        sqlx::query("UPDATE memories SET importance = $1, updated_at = now() WHERE id = $2")
            .bind(importance)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;
    }

    sqlx::query(
        r#"INSERT INTO memory_audit (tenant_id, memory_id, target_table, action, actor_type, diff)
        VALUES ($1, $2, 'memories', 'update', 'user', $3)"#,
    )
    .bind(tenant_id).bind(id)
    .bind(serde_json::json!({
        "before": { "content": current.content, "status": current.status, "importance": current.importance },
        "after": { "content": body.content, "status": body.status, "importance": body.importance }
    }))
    .execute(&mut *tx).await
    .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;
    Ok(Json(serde_json::json!({ "id": id, "updated": true })))
}

async fn delete_memory(
    State(state): State<Arc<AdminAppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, memory_common::MemoryError> {
    let tenant_id = extract_tenant_id(&headers)?;
    let mut tx = memory_db::begin_tenant_tx(&state.db_pool, tenant_id)
        .await
        .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;

    sqlx::query("UPDATE memories SET status = 'deleted', updated_at = now() WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;
    sqlx::query("INSERT INTO memory_audit (tenant_id, memory_id, target_table, action, actor_type) VALUES ($1, $2, 'memories', 'delete', 'user')")
        .bind(tenant_id).bind(id).execute(&mut *tx).await
        .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;
    Ok(Json(serde_json::json!({ "id": id, "deleted": true })))
}

async fn merge_memories(
    State(state): State<Arc<AdminAppState>>,
    headers: HeaderMap,
    Json(body): Json<MergeRequest>,
) -> Result<Json<serde_json::Value>, memory_common::MemoryError> {
    let tenant_id = extract_tenant_id(&headers)?;
    let mut tx = memory_db::begin_tenant_tx(&state.db_pool, tenant_id)
        .await
        .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;

    for source_id in &body.source_ids {
        sqlx::query("UPDATE memories SET status = 'superseded', valid_to = now(), updated_at = now() WHERE id = $1")
            .bind(source_id).execute(&mut *tx).await
            .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;
    }

    let merged_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO memories (tenant_id, user_id, scope, kind, content, confidence, importance, created_by, metadata) VALUES ($1, $2, $3, $4, $5, $6, $7, 'system', $8) RETURNING id",
    )
    .bind(tenant_id).bind(body.user_id)
    .bind(body.scope.as_deref().unwrap_or("user"))
    .bind(body.kind.as_deref().unwrap_or("fact"))
    .bind(&body.merged_content)
    .bind(body.confidence.unwrap_or(0.9))
    .bind(body.importance.unwrap_or(0.7))
    .bind(serde_json::json!({ "merged_from": body.source_ids }))
    .fetch_one(&mut *tx).await
    .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;

    for source_id in &body.source_ids {
        sqlx::query("INSERT INTO memory_audit (tenant_id, memory_id, target_table, action, actor_type, diff) VALUES ($1, $2, 'memories', 'merge', 'system', $3)")
            .bind(tenant_id).bind(source_id)
            .bind(serde_json::json!({ "merged_into": merged_id }))
            .execute(&mut *tx).await
            .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;
    }

    tx.commit()
        .await
        .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;
    Ok(Json(
        serde_json::json!({ "merged_id": merged_id, "source_ids": body.source_ids }),
    ))
}

async fn list_audit(
    State(state): State<Arc<AdminAppState>>,
    headers: HeaderMap,
    Query(params): Query<AuditParams>,
) -> Result<Json<Vec<serde_json::Value>>, memory_common::MemoryError> {
    let tenant_id = extract_tenant_id(&headers)?;
    let mut conn = memory_db::acquire_tenant_conn(&state.db_pool, tenant_id)
        .await
        .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;

    let rows: Vec<AuditRow> = sqlx::query_as(
        "SELECT id, memory_id, target_table, action, actor_type, actor_id, diff, reason, timestamp FROM memory_audit WHERE ($1::uuid IS NULL OR memory_id = $1) ORDER BY timestamp DESC LIMIT $2",
    )
    .bind(params.memory_id).bind(params.limit.unwrap_or(50))
    .fetch_all(conn.as_mut()).await
    .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;

    let json: Vec<serde_json::Value> = rows
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| memory_common::MemoryError::Internal(format!("Serialization error: {}", e)))?;
    Ok(Json(json))
}

async fn list_policies(
    State(state): State<Arc<AdminAppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, memory_common::MemoryError> {
    let tenant_id = extract_tenant_id(&headers)?;
    let mut conn = memory_db::acquire_tenant_conn(&state.db_pool, tenant_id)
        .await
        .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;

    let rows: Vec<PolicyRow> = sqlx::query_as("SELECT * FROM memory_policies WHERE enabled = true")
        .fetch_all(conn.as_mut())
        .await
        .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;

    let json: Vec<serde_json::Value> = rows
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| memory_common::MemoryError::Internal(format!("Serialization error: {}", e)))?;
    Ok(Json(json))
}

async fn create_policy(
    State(state): State<Arc<AdminAppState>>,
    headers: HeaderMap,
    Json(body): Json<CreatePolicyRequest>,
) -> Result<Json<serde_json::Value>, memory_common::MemoryError> {
    let tenant_id = extract_tenant_id(&headers)?;
    let id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO memory_policies (tenant_id, name, rule_type, config) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(tenant_id).bind(&body.name).bind(&body.rule_type).bind(&body.config)
    .fetch_one(&state.db_pool).await
    .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn simulate_replay(
    State(state): State<Arc<AdminAppState>>,
    headers: HeaderMap,
    Json(body): Json<SimulateRequest>,
) -> Result<Json<serde_json::Value>, memory_common::MemoryError> {
    let tenant_id = extract_tenant_id(&headers)?;
    let mut conn = memory_db::acquire_tenant_conn(&state.db_pool, tenant_id)
        .await
        .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;

    let rows: Vec<MemoryRow> = sqlx::query_as(
        "SELECT * FROM memories WHERE valid_from <= $1 AND (valid_to IS NULL OR valid_to > $1) AND status != 'deleted' AND ($2::uuid IS NULL OR user_id = $2) ORDER BY importance DESC LIMIT $3",
    )
    .bind(body.point_in_time).bind(body.user_id).bind(body.limit.unwrap_or(100) as i64)
    .fetch_all(conn.as_mut()).await
    .map_err(|e| memory_common::MemoryError::Database(e.to_string()))?;

    let json: Vec<serde_json::Value> = rows
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| memory_common::MemoryError::Internal(format!("Serialization error: {}", e)))?;
    Ok(Json(serde_json::json!({
        "point_in_time": body.point_in_time,
        "memory_count": json.len(),
        "memories": json,
    })))
}

// ── Types ──

#[derive(Debug, Deserialize)]
struct UpdateMemoryRequest {
    content: Option<String>,
    status: Option<String>,
    importance: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct MergeRequest {
    source_ids: Vec<Uuid>,
    merged_content: String,
    user_id: Option<Uuid>,
    scope: Option<String>,
    kind: Option<String>,
    confidence: Option<f32>,
    importance: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct AuditParams {
    limit: Option<i64>,
    memory_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
struct CreatePolicyRequest {
    name: String,
    rule_type: String,
    config: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct SimulateRequest {
    point_in_time: chrono::DateTime<chrono::Utc>,
    user_id: Option<Uuid>,
    limit: Option<usize>,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
struct MemoryRow {
    id: Uuid,
    tenant_id: Uuid,
    user_id: Option<Uuid>,
    scope: String,
    kind: String,
    content: String,
    content_json: Option<serde_json::Value>,
    confidence: f32,
    importance: f32,
    status: String,
    valid_from: chrono::DateTime<chrono::Utc>,
    valid_to: Option<chrono::DateTime<chrono::Utc>>,
    event_time: Option<chrono::DateTime<chrono::Utc>>,
    ingested_at: chrono::DateTime<chrono::Utc>,
    source_episode_id: Option<Uuid>,
    created_by: String,
    tags: Vec<String>,
    metadata: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
struct AuditRow {
    id: Uuid,
    memory_id: Uuid,
    target_table: String,
    action: String,
    actor_type: String,
    actor_id: Option<String>,
    diff: Option<serde_json::Value>,
    reason: Option<String>,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
struct PolicyRow {
    id: Uuid,
    tenant_id: Uuid,
    name: String,
    rule_type: String,
    config: serde_json::Value,
    enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}
