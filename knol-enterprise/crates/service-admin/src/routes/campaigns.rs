//! Marketing campaign management.
//! Supports the zero-cost marketing strategy: phases, descriptions, stats, and trigger.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::{AdminClaims, AdminError};
use crate::AdminAppState;

pub async fn list_campaigns(
    State(state): State<Arc<AdminAppState>>,
    _claims: AdminClaims,
) -> Result<Json<Vec<serde_json::Value>>, AdminError> {
    let has_phase = has_column(&state.db_pool, "marketing_campaigns", "phase").await?;
    let has_description = has_column(&state.db_pool, "marketing_campaigns", "description").await?;

    // Compatibility: local/dev databases created from older consolidated migrations
    // may not have `phase`/`description` yet. Project those fields with defaults.
    let phase_expr = if has_phase {
        "phase"
    } else {
        "'content_engine'::text AS phase"
    };
    let description_expr = if has_description {
        "description"
    } else {
        "''::text AS description"
    };
    let order_clause = if has_phase { "phase, name" } else { "name" };
    let sql = format!(
        "SELECT id, name, cron, channels, enabled, {}, {}, created_at, updated_at FROM marketing_campaigns ORDER BY {}",
        phase_expr, description_expr, order_clause
    );

    let rows = sqlx::query_as::<_, CampaignRow>(&sql)
        .fetch_all(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            AdminError::Internal("Database error".into())
        })?;

    let mut campaigns = Vec::new();
    for r in &rows {
        // Get last publish for this campaign
        let last_publish = sqlx::query_as::<_, LastPublishRow>(
            "SELECT channel, success, published_at FROM marketing_publish_log WHERE campaign = $1 ORDER BY published_at DESC LIMIT 1",
        )
        .bind(&r.name)
        .fetch_optional(&state.db_pool)
        .await
        .ok()
        .flatten();

        // Count total publishes and success rate
        let stats = sqlx::query_as::<_, CampaignStatsRow>(
            "SELECT COUNT(*) as total, COALESCE(SUM(CASE WHEN success THEN 1 ELSE 0 END), 0) as successes FROM marketing_publish_log WHERE campaign = $1",
        )
        .bind(&r.name)
        .fetch_optional(&state.db_pool)
        .await
        .ok()
        .flatten();

        campaigns.push(serde_json::json!({
            "id": r.id,
            "name": r.name,
            "cron": r.cron,
            "channels": r.channels,
            "enabled": r.enabled,
            "phase": r.phase,
            "description": r.description,
            "created_at": r.created_at.to_rfc3339(),
            "updated_at": r.updated_at.to_rfc3339(),
            "last_publish": last_publish.map(|lp| serde_json::json!({
                "channel": lp.channel,
                "success": lp.success,
                "published_at": lp.published_at.to_rfc3339(),
            })),
            "stats": stats.map(|s| serde_json::json!({
                "total_publishes": s.total,
                "successful": s.successes,
                "success_rate": if s.total > 0 { (s.successes as f64 / s.total as f64 * 100.0).round() } else { 0.0 },
            })),
        }));
    }

    Ok(Json(campaigns))
}

#[derive(Deserialize)]
pub struct UpdateCampaign {
    pub enabled: Option<bool>,
    pub cron: Option<String>,
    pub channels: Option<Vec<String>>,
    pub phase: Option<String>,
    pub description: Option<String>,
}

pub async fn update_campaign(
    State(state): State<Arc<AdminAppState>>,
    claims: AdminClaims,
    Path(name): Path<String>,
    Json(body): Json<UpdateCampaign>,
) -> Result<Json<serde_json::Value>, AdminError> {
    if claims.role == "read_only" {
        return Err(AdminError::Forbidden);
    }

    // SECURITY: Validate inputs before touching the database.
    validate_campaign_name(&name)?;
    if let Some(cron) = &body.cron {
        validate_cron_schedule(cron)?;
    }
    if let Some(phase) = &body.phase {
        let valid_phases = ["launch", "content_engine", "community", "conversion"];
        if !valid_phases.contains(&phase.as_str()) {
            return Err(AdminError::BadRequest(format!(
                "Invalid phase '{}'. Must be one of: {:?}",
                phase, valid_phases
            )));
        }
    }
    if let Some(channels) = &body.channels {
        let valid_channels = [
            "twitter",
            "blog",
            "devto",
            "hashnode",
            "medium",
            "reddit",
            "linkedin",
            "email",
            "github",
            "producthunt",
            "hackernews",
        ];
        for ch in channels {
            if !valid_channels.contains(&ch.as_str()) {
                return Err(AdminError::BadRequest(format!(
                    "Invalid channel '{}'. Must be one of: {:?}",
                    ch, valid_channels
                )));
            }
        }
    }

    let has_phase = has_column(&state.db_pool, "marketing_campaigns", "phase").await?;
    let has_description = has_column(&state.db_pool, "marketing_campaigns", "description").await?;

    // Get old values for audit, compatible across schema versions.
    let phase_expr = if has_phase {
        "phase"
    } else {
        "'content_engine'::text AS phase"
    };
    let description_expr = if has_description {
        "description"
    } else {
        "''::text AS description"
    };
    let old_sql = format!(
        "SELECT id, name, cron, channels, enabled, {}, {}, created_at, updated_at FROM marketing_campaigns WHERE name = $1",
        phase_expr, description_expr
    );
    let old = sqlx::query_as::<_, CampaignRow>(&old_sql)
        .bind(&name)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            AdminError::Internal("Database error".into())
        })?
        .ok_or_else(|| AdminError::NotFound(format!("Campaign '{}' not found", name)))?;

    if let Some(enabled) = body.enabled {
        sqlx::query(
            "UPDATE marketing_campaigns SET enabled = $1, updated_at = NOW() WHERE name = $2",
        )
        .bind(enabled)
        .bind(&name)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            AdminError::Internal("Database error".into())
        })?;
    }

    if let Some(cron) = &body.cron {
        sqlx::query("UPDATE marketing_campaigns SET cron = $1, updated_at = NOW() WHERE name = $2")
            .bind(cron)
            .bind(&name)
            .execute(&state.db_pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error: {}", e);
                AdminError::Internal("Database error".into())
            })?;
    }

    if let Some(channels) = &body.channels {
        sqlx::query(
            "UPDATE marketing_campaigns SET channels = $1, updated_at = NOW() WHERE name = $2",
        )
        .bind(channels)
        .bind(&name)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            AdminError::Internal("Database error".into())
        })?;
    }

    let mut skipped_fields = Vec::new();

    if let Some(phase) = &body.phase {
        if has_phase {
            sqlx::query(
                "UPDATE marketing_campaigns SET phase = $1, updated_at = NOW() WHERE name = $2",
            )
            .bind(phase)
            .bind(&name)
            .execute(&state.db_pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error: {}", e);
                AdminError::Internal("Database error".into())
            })?;
        } else {
            tracing::warn!("Skipping phase update for campaign '{}' because marketing_campaigns.phase does not exist", name);
            skipped_fields.push("phase");
        }
    }

    if let Some(description) = &body.description {
        if has_description {
            sqlx::query("UPDATE marketing_campaigns SET description = $1, updated_at = NOW() WHERE name = $2")
                .bind(description)
                .bind(&name)
                .execute(&state.db_pool)
                .await
                .map_err(|e| {
                    tracing::error!("Database error: {}", e);
                    AdminError::Internal("Database error".into())
                })?;
        } else {
            tracing::warn!("Skipping description update for campaign '{}' because marketing_campaigns.description does not exist", name);
            skipped_fields.push("description");
        }
    }

    // Audit
    let _ = sqlx::query(
        "INSERT INTO admin_audit_log (admin_id, admin_email, action, resource_type, resource_key, old_value, new_value) VALUES ($1, $2, 'update', 'campaign', $3, $4, $5)",
    )
    .bind(claims.sub)
    .bind(&claims.email)
    .bind(&name)
    .bind(serde_json::json!({"enabled": old.enabled, "cron": old.cron, "channels": old.channels, "phase": old.phase, "description": old.description}))
    .bind(serde_json::json!({"enabled": body.enabled, "cron": body.cron, "channels": body.channels, "phase": body.phase, "description": body.description}))
    .execute(&state.db_pool)
    .await;

    Ok(Json(serde_json::json!({
        "name": name,
        "updated": true,
        "skipped_fields": skipped_fields,
    })))
}

#[derive(Deserialize)]
pub struct LogParams {
    pub limit: Option<i64>,
}

pub async fn campaign_logs(
    State(state): State<Arc<AdminAppState>>,
    _claims: AdminClaims,
    Path(name): Path<String>,
    Query(params): Query<LogParams>,
) -> Result<Json<Vec<serde_json::Value>>, AdminError> {
    let limit = params.limit.unwrap_or(50);

    let rows = sqlx::query_as::<_, PublishLogRow>(
        "SELECT campaign, channel, success, message_id, url, error, published_at FROM marketing_publish_log WHERE campaign = $1 ORDER BY published_at DESC LIMIT $2",
    )
    .bind(&name)
    .bind(limit)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        AdminError::Internal("Database error".into())
    })?;

    let json: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "campaign": r.campaign,
                "channel": r.channel,
                "success": r.success,
                "message_id": r.message_id,
                "url": r.url,
                "error": r.error,
                "published_at": r.published_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(json))
}

/// SECURITY: Validate campaign name to prevent SSRF via path traversal.
/// Only lowercase letters, digits, and underscores are allowed.
fn validate_campaign_name(name: &str) -> Result<(), AdminError> {
    if name.is_empty() || name.len() > 64 {
        return Err(AdminError::BadRequest(
            "Campaign name must be 1-64 characters".into(),
        ));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err(AdminError::BadRequest(
            "Campaign name may only contain lowercase letters, digits, and underscores".into(),
        ));
    }
    Ok(())
}

/// SECURITY: Validate cron expression to prevent extreme scheduling.
fn validate_cron_schedule(cron: &str) -> Result<(), AdminError> {
    if cron.is_empty() || cron.len() > 64 {
        return Err(AdminError::BadRequest(
            "Cron expression must be 1-64 characters".into(),
        ));
    }
    // Block cron expressions that fire more than once per minute.
    // A 6-field cron like "* * * * * *" fires every second.
    // We require the seconds field is not "*" or "*/N" where N < 60.
    let parts: Vec<&str> = cron.split_whitespace().collect();
    if parts.len() == 6 {
        let seconds = parts[0];
        if seconds == "*" {
            return Err(AdminError::BadRequest(
                "Cron schedule fires every second — minimum interval is 1 minute (use '0' for seconds field)".into(),
            ));
        }
        if let Some(interval) = seconds.strip_prefix("*/") {
            if let Ok(n) = interval.parse::<u32>() {
                if n < 60 {
                    return Err(AdminError::BadRequest(format!(
                        "Cron schedule fires every {} seconds — minimum interval is 1 minute",
                        n
                    )));
                }
            }
        }
    }
    Ok(())
}

/// Trigger a campaign manually (calls the marketing service).
#[derive(Deserialize)]
pub struct TriggerParams {
    pub force: Option<bool>,
}

pub async fn trigger_campaign(
    State(state): State<Arc<AdminAppState>>,
    claims: AdminClaims,
    Path(name): Path<String>,
    Query(params): Query<TriggerParams>,
) -> Result<Json<serde_json::Value>, AdminError> {
    if claims.role == "read_only" {
        return Err(AdminError::Forbidden);
    }

    // SECURITY: Validate campaign name to prevent SSRF/path traversal when
    // building the URL to the marketing service.
    validate_campaign_name(&name)?;

    // Verify campaign exists in DB (also prevents triggering arbitrary endpoints)
    let exists = sqlx::query_scalar::<_, i64>("SELECT 1 FROM marketing_campaigns WHERE name = $1")
        .bind(&name)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching campaign: {}", e);
            AdminError::Internal("Database error".into())
        })?
        .is_some();
    if !exists {
        return Err(AdminError::NotFound(format!(
            "Campaign '{}' not found",
            name
        )));
    }

    let marketing_url = std::env::var("MARKETING_SERVICE_URL").map_err(|_| {
        tracing::error!("MARKETING_SERVICE_URL env var is not set");
        AdminError::Internal("Marketing service not configured".into())
    })?;

    let force = params.force.unwrap_or(false);
    let url = format!("{}/trigger/{}?force={}", marketing_url, name, force);

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Marketing service unreachable: {}", e);
            AdminError::Internal("Marketing service unavailable".into())
        })?;

    let status = resp.status();
    let body: serde_json::Value = resp.json().await.unwrap_or_default();

    // Audit the trigger
    let _ = sqlx::query(
        "INSERT INTO admin_audit_log (admin_id, admin_email, action, resource_type, resource_key, new_value) VALUES ($1, $2, 'trigger', 'campaign', $3, $4)",
    )
    .bind(claims.sub)
    .bind(&claims.email)
    .bind(&name)
    .bind(serde_json::json!({"force": force, "status": status.as_u16()}))
    .execute(&state.db_pool)
    .await;

    Ok(Json(serde_json::json!({
        "name": name,
        "triggered": status.is_success(),
        "status": status.as_u16(),
        "response": body,
    })))
}

/// Get marketing stats/analytics.
#[derive(Deserialize)]
pub struct StatsParams {
    pub days: Option<i64>,
}

pub async fn marketing_stats(
    State(state): State<Arc<AdminAppState>>,
    _claims: AdminClaims,
    Query(params): Query<StatsParams>,
) -> Result<Json<serde_json::Value>, AdminError> {
    let days = params.days.unwrap_or(30);
    let has_phase = has_column(&state.db_pool, "marketing_campaigns", "phase").await?;
    let has_marketing_stats_table = has_table(&state.db_pool, "marketing_stats").await?;

    // Channel breakdown
    let channel_stats = sqlx::query_as::<_, ChannelStatsRow>(
        "SELECT channel, COUNT(*) as total, COALESCE(SUM(CASE WHEN success THEN 1 ELSE 0 END), 0) as successes FROM marketing_publish_log WHERE published_at > NOW() - make_interval(days => $1) GROUP BY channel ORDER BY total DESC",
    )
    .bind(days as i32)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        AdminError::Internal("Database error".into())
    })?;

    // Phase breakdown
    let phase_expr = if has_phase {
        "mc.phase"
    } else {
        "'content_engine'::text"
    };
    let phase_sql = format!(
        "SELECT {} as phase, COUNT(mpl.*) as total, COALESCE(SUM(CASE WHEN mpl.success THEN 1 ELSE 0 END), 0) as successes
         FROM marketing_campaigns mc
         LEFT JOIN marketing_publish_log mpl ON mpl.campaign = mc.name AND mpl.published_at > NOW() - make_interval(days => $1)
         GROUP BY 1 ORDER BY 1",
        phase_expr
    );
    let phase_stats = sqlx::query_as::<_, PhaseStatsRow>(&phase_sql)
        .bind(days as i32)
        .fetch_all(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            AdminError::Internal("Database error".into())
        })?;

    // Daily publish counts (last N days)
    let daily_counts = sqlx::query_as::<_, DailyCountRow>(
        "SELECT DATE(published_at) as day, COUNT(*) as total, COALESCE(SUM(CASE WHEN success THEN 1 ELSE 0 END), 0) as successes FROM marketing_publish_log WHERE published_at > NOW() - make_interval(days => $1) GROUP BY DATE(published_at) ORDER BY day",
    )
    .bind(days as i32)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        AdminError::Internal("Database error".into())
    })?;

    // Custom marketing metrics (from marketing_stats table)
    let metrics = if has_marketing_stats_table {
        sqlx::query_as::<_, MetricRow>(
            "SELECT DISTINCT ON (metric_name) metric_name, metric_value, recorded_at, metadata FROM marketing_stats ORDER BY metric_name, recorded_at DESC",
        )
        .fetch_all(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            AdminError::Internal("Database error".into())
        })?
    } else {
        Vec::new()
    };

    // Total summary
    let total_row = sqlx::query_as::<_, CampaignStatsRow>(
        "SELECT COUNT(*) as total, COALESCE(SUM(CASE WHEN success THEN 1 ELSE 0 END), 0) as successes FROM marketing_publish_log WHERE published_at > NOW() - make_interval(days => $1)",
    )
    .bind(days as i32)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        AdminError::Internal("Database error".into())
    })?;

    let (total, successes) = total_row.map(|r| (r.total, r.successes)).unwrap_or((0, 0));

    Ok(Json(serde_json::json!({
        "period_days": days,
        "strategy": "zero-cost",
        "summary": {
            "total_publishes": total,
            "successful": successes,
            "success_rate": if total > 0 { (successes as f64 / total as f64 * 100.0).round() } else { 0.0 },
        },
        "by_channel": channel_stats.iter().map(|c| serde_json::json!({
            "channel": c.channel,
            "total": c.total,
            "successful": c.successes,
            "success_rate": if c.total > 0 { (c.successes as f64 / c.total as f64 * 100.0).round() } else { 0.0 },
        })).collect::<Vec<_>>(),
        "by_phase": phase_stats.iter().map(|p| serde_json::json!({
            "phase": p.phase,
            "total": p.total,
            "successful": p.successes,
        })).collect::<Vec<_>>(),
        "daily": daily_counts.iter().map(|d| serde_json::json!({
            "date": d.day.to_string(),
            "total": d.total,
            "successful": d.successes,
        })).collect::<Vec<_>>(),
        "metrics": metrics.iter().map(|m| serde_json::json!({
            "name": m.metric_name,
            "value": m.metric_value,
            "recorded_at": m.recorded_at.to_string(),
            "metadata": m.metadata,
        })).collect::<Vec<_>>(),
    })))
}

/// Record a marketing metric (called by marketing service or manually).
#[derive(Deserialize)]
pub struct RecordMetric {
    pub metric_name: String,
    pub metric_value: f64,
    pub metadata: Option<serde_json::Value>,
}

pub async fn record_metric(
    State(state): State<Arc<AdminAppState>>,
    claims: AdminClaims,
    Json(body): Json<RecordMetric>,
) -> Result<Json<serde_json::Value>, AdminError> {
    if claims.role == "read_only" {
        return Err(AdminError::Forbidden);
    }

    sqlx::query(
        "INSERT INTO marketing_stats (metric_name, metric_value, metadata) VALUES ($1, $2, $3) ON CONFLICT (metric_name, recorded_at) DO UPDATE SET metric_value = $2, metadata = $3",
    )
    .bind(&body.metric_name)
    .bind(body.metric_value)
    .bind(&body.metadata)
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        AdminError::Internal("Database error".into())
    })?;

    Ok(Json(
        serde_json::json!({"recorded": true, "metric": body.metric_name}),
    ))
}

async fn has_column(pool: &sqlx::PgPool, table: &str, column: &str) -> Result<bool, AdminError> {
    let exists = sqlx::query_scalar::<_, i64>(
        r#"SELECT 1
           FROM information_schema.columns
           WHERE table_schema = 'public'
             AND table_name = $1
             AND column_name = $2
           LIMIT 1"#,
    )
    .bind(table)
    .bind(column)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error checking column {}.{}: {}", table, column, e);
        AdminError::Internal("Database error".into())
    })?;
    Ok(exists.is_some())
}

async fn has_table(pool: &sqlx::PgPool, table: &str) -> Result<bool, AdminError> {
    let exists = sqlx::query_scalar::<_, i64>(
        r#"SELECT 1
           FROM information_schema.tables
           WHERE table_schema = 'public'
             AND table_name = $1
           LIMIT 1"#,
    )
    .bind(table)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error checking table {}: {}", table, e);
        AdminError::Internal("Database error".into())
    })?;
    Ok(exists.is_some())
}

// ── Row types ────────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct CampaignRow {
    id: Uuid,
    name: String,
    cron: String,
    channels: Vec<String>,
    enabled: bool,
    phase: String,
    description: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
struct LastPublishRow {
    channel: String,
    success: bool,
    published_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
struct PublishLogRow {
    campaign: String,
    channel: String,
    success: bool,
    message_id: Option<String>,
    url: Option<String>,
    error: Option<String>,
    published_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
struct CampaignStatsRow {
    total: i64,
    successes: i64,
}

#[derive(sqlx::FromRow)]
struct ChannelStatsRow {
    channel: String,
    total: i64,
    successes: i64,
}

#[derive(sqlx::FromRow)]
struct PhaseStatsRow {
    phase: String,
    total: i64,
    successes: i64,
}

#[derive(sqlx::FromRow)]
struct DailyCountRow {
    day: chrono::NaiveDate,
    total: i64,
    successes: i64,
}

#[derive(sqlx::FromRow)]
struct MetricRow {
    metric_name: String,
    metric_value: f64,
    recorded_at: chrono::NaiveDate,
    metadata: Option<serde_json::Value>,
}
