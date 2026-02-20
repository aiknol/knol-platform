//! Memory Jobs Service
//!
//! Background cron jobs: importance decay, dedup scan, retention enforcement,
//! stale edge cleanup, memory consolidation, and conflict detection.
#![allow(dead_code)]

use std::sync::Arc;
use tracing::{error, info};

mod conflict;
mod consolidation;

struct AppState {
    db_pool: sqlx::PgPool,
    llm_client: memory_llm::AnthropicClient,
}

type JobFuture = std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<u64>> + Send>>;
type PeriodicJob = fn(Arc<AppState>) -> JobFuture;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .json()
        .init();

    info!("Starting Memory Jobs Service...");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://memory:memory_dev@localhost:5432/memory".into());

    let db_pool = memory_db::create_pool(&database_url, 4).await?;

    // API key from env (secret — not stored in system_config)
    let anthropic_api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
    // Model from admin panel DB → env → default
    let anthropic_model = memory_common::db_config::load_str(
        &db_pool,
        "llm.anthropic_model",
        "ANTHROPIC_MODEL",
        "claude-3-haiku-20240307",
    )
    .await;
    let llm_client = memory_llm::AnthropicClient::new(anthropic_api_key, anthropic_model);

    let state = Arc::new(AppState {
        db_pool,
        llm_client,
    });

    // Run all jobs concurrently
    let decay_state = state.clone();
    let dedup_state = state.clone();
    let retention_state = state.clone();
    let cleanup_state = state.clone();
    let consolidation_state = state.clone();
    let conflict_state = state.clone();

    tokio::select! {
        _ = run_periodic(decay_state, "importance_decay", std::time::Duration::from_secs(3600), importance_decay) => {},
        _ = run_periodic(dedup_state, "dedup_scan", std::time::Duration::from_secs(7200), dedup_scan) => {},
        _ = run_periodic(retention_state, "retention_enforce", std::time::Duration::from_secs(86400), retention_enforce) => {},
        _ = run_periodic(cleanup_state, "stale_edge_cleanup", std::time::Duration::from_secs(86400), stale_edge_cleanup) => {},
        _ = run_periodic(consolidation_state, "consolidation", std::time::Duration::from_secs(43200), memory_consolidation) => {},
        _ = run_periodic(conflict_state, "conflict_detection", std::time::Duration::from_secs(14400), conflict_detection) => {},
    }

    Ok(())
}

async fn run_periodic(
    state: Arc<AppState>,
    name: &'static str,
    interval: std::time::Duration,
    job: PeriodicJob,
) {
    info!("Job '{}' scheduled every {:?}", name, interval);
    loop {
        tokio::time::sleep(interval).await;
        info!("Running job '{}'...", name);
        match job(state.clone()).await {
            Ok(affected) => info!("Job '{}' complete: {} records affected", name, affected),
            Err(e) => error!("Job '{}' failed: {}", name, e),
        }
    }
}

/// Decay importance scores using exponential decay: importance *= e^(-λ * days_since_update)
fn importance_decay(state: Arc<AppState>) -> JobFuture {
    Box::pin(async move {
        let decay_lambda = 0.01_f64;
        let result = sqlx::query(
            r#"
            UPDATE memories
            SET importance = importance * exp(-$1 * EXTRACT(EPOCH FROM (now() - updated_at)) / 86400.0),
                updated_at = now()
            WHERE status = 'active'
              AND importance > 0.05
              AND updated_at < now() - interval '1 day'
            "#,
        )
        .bind(decay_lambda)
        .execute(&state.db_pool)
        .await?;

        // Log decay actions to audit
        let affected = result.rows_affected();
        if affected > 0 {
            sqlx::query(
                r#"
                INSERT INTO memory_audit (tenant_id, memory_id, target_table, action, actor_type, reason)
                SELECT tenant_id, id, 'memories', 'decay', 'system', 'periodic importance decay'
                FROM memories
                WHERE status = 'active' AND importance <= 0.05
                "#,
            )
            .execute(&state.db_pool)
            .await
            .ok();
        }

        Ok(affected)
    })
}

/// Scan for and mark duplicate memories.
fn dedup_scan(state: Arc<AppState>) -> JobFuture {
    Box::pin(async move {
        // Find duplicate content_hashes within the same tenant
        let result = sqlx::query(
            r#"
            WITH dupes AS (
                SELECT mv.memory_id, mv.tenant_id,
                       ROW_NUMBER() OVER (PARTITION BY mv.tenant_id, mv.content_hash ORDER BY mv.created_at ASC) as rn
                FROM memory_vectors mv
                WHERE mv.status = 'active'
            )
            UPDATE memories m
            SET status = 'superseded', valid_to = now(), updated_at = now()
            FROM dupes d
            WHERE d.memory_id = m.id AND d.rn > 1
            "#,
        )
        .execute(&state.db_pool)
        .await?;

        Ok(result.rows_affected())
    })
}

/// Enforce retention policies (delete memories past their retention period).
fn retention_enforce(state: Arc<AppState>) -> JobFuture {
    Box::pin(async move {
        // Check tenant-level retention policies
        let result = sqlx::query(
            r#"
            UPDATE memories m
            SET status = 'archived', updated_at = now()
            FROM tenants t
            WHERE m.tenant_id = t.id
              AND m.status = 'active'
              AND (t.config->>'retention_days')::int IS NOT NULL
              AND m.created_at < now() - make_interval(days => (t.config->>'retention_days')::int)
            "#,
        )
        .execute(&state.db_pool)
        .await?;

        // Also enforce policy-based retention
        let policy_result = sqlx::query(
            r#"
            UPDATE memories m
            SET status = 'archived', updated_at = now()
            FROM memory_policies p
            WHERE p.tenant_id = m.tenant_id
              AND p.rule_type = 'retention'
              AND p.enabled = true
              AND m.status = 'active'
              AND m.created_at < now() - make_interval(days => (p.config->>'days')::int)
            "#,
        )
        .execute(&state.db_pool)
        .await?;

        Ok(result.rows_affected() + policy_result.rows_affected())
    })
}

/// Clean up stale edges (edges where source or target entity is deleted/merged).
fn stale_edge_cleanup(state: Arc<AppState>) -> JobFuture {
    Box::pin(async move {
        let result = sqlx::query(
            r#"
            UPDATE edges e
            SET status = 'deleted', valid_to = now(), updated_at = now()
            FROM entities src, entities tgt
            WHERE e.source_entity_id = src.id
              AND e.target_entity_id = tgt.id
              AND e.status = 'active'
              AND (src.status != 'active' OR tgt.status != 'active')
            "#,
        )
        .execute(&state.db_pool)
        .await?;

        Ok(result.rows_affected())
    })
}

/// Consolidate episodic memories into semantic memories using LLM synthesis.
fn memory_consolidation(state: Arc<AppState>) -> JobFuture {
    Box::pin(async move {
        let engine = consolidation::ConsolidationEngine::new(
            state.db_pool.clone(),
            state.llm_client.clone(),
        );

        match engine.run_consolidation().await {
            Ok(count) => Ok(count),
            Err(e) => {
                error!("Memory consolidation failed: {}", e);
                Err(e)
            }
        }
    })
}

/// Detect and resolve conflicts between memories in same tenant/scope.
fn conflict_detection(state: Arc<AppState>) -> JobFuture {
    Box::pin(async move {
        let detector = conflict::ConflictDetector::new(state.db_pool.clone());

        match detector.run_conflict_detection().await {
            Ok(count) => Ok(count),
            Err(e) => {
                error!("Memory conflict detection failed: {}", e);
                Err(e)
            }
        }
    })
}
