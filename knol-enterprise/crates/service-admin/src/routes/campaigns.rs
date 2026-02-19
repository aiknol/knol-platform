//! Marketing campaign management.

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
    let rows = sqlx::query_as::<_, CampaignRow>(
        "SELECT id, name, cron, channels, enabled, created_at, updated_at FROM marketing_campaigns ORDER BY name",
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AdminError::Internal(e.to_string()))?;

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

        campaigns.push(serde_json::json!({
            "id": r.id,
            "name": r.name,
            "cron": r.cron,
            "channels": r.channels,
            "enabled": r.enabled,
            "created_at": r.created_at.to_rfc3339(),
            "updated_at": r.updated_at.to_rfc3339(),
            "last_publish": last_publish.map(|lp| serde_json::json!({
                "channel": lp.channel,
                "success": lp.success,
                "published_at": lp.published_at.to_rfc3339(),
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

    // Get old values for audit
    let old = sqlx::query_as::<_, CampaignRow>(
        "SELECT id, name, cron, channels, enabled, created_at, updated_at FROM marketing_campaigns WHERE name = $1",
    )
    .bind(&name)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AdminError::Internal(e.to_string()))?
    .ok_or_else(|| AdminError::NotFound(format!("Campaign '{}' not found", name)))?;

    if let Some(enabled) = body.enabled {
        sqlx::query(
            "UPDATE marketing_campaigns SET enabled = $1, updated_at = NOW() WHERE name = $2",
        )
        .bind(enabled)
        .bind(&name)
        .execute(&state.db_pool)
        .await
        .map_err(|e| AdminError::Internal(e.to_string()))?;
    }

    if let Some(cron) = &body.cron {
        sqlx::query("UPDATE marketing_campaigns SET cron = $1, updated_at = NOW() WHERE name = $2")
            .bind(cron)
            .bind(&name)
            .execute(&state.db_pool)
            .await
            .map_err(|e| AdminError::Internal(e.to_string()))?;
    }

    if let Some(channels) = &body.channels {
        sqlx::query(
            "UPDATE marketing_campaigns SET channels = $1, updated_at = NOW() WHERE name = $2",
        )
        .bind(channels)
        .bind(&name)
        .execute(&state.db_pool)
        .await
        .map_err(|e| AdminError::Internal(e.to_string()))?;
    }

    // Audit
    let _ = sqlx::query(
        "INSERT INTO admin_audit_log (admin_id, admin_email, action, resource_type, resource_key, old_value, new_value) VALUES ($1, $2, 'update', 'campaign', $3, $4, $5)",
    )
    .bind(claims.sub)
    .bind(&claims.email)
    .bind(&name)
    .bind(serde_json::json!({"enabled": old.enabled, "cron": old.cron, "channels": old.channels}))
    .bind(serde_json::json!({"enabled": body.enabled, "cron": body.cron, "channels": body.channels}))
    .execute(&state.db_pool)
    .await;

    Ok(Json(serde_json::json!({"name": name, "updated": true})))
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
    .map_err(|e| AdminError::Internal(e.to_string()))?;

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

#[derive(sqlx::FromRow)]
struct CampaignRow {
    id: Uuid,
    name: String,
    cron: String,
    channels: Vec<String>,
    enabled: bool,
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
