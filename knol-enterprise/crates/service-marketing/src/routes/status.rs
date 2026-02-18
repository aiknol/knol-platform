use std::sync::Arc;
use axum::{extract::State, Json};

use crate::scheduler::campaigns;
use crate::state::AppState;

pub async fn status(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    // Rate limit status for all channels
    let all = state.rate_limiter.all_statuses().await;
    let rate_limits: Vec<serde_json::Value> = all
        .into_iter()
        .map(|(channel, statuses)| {
            let window_status: Vec<serde_json::Value> = statuses
                .into_iter()
                .map(|s| {
                    serde_json::json!({
                        "window": format!("{:?}", s.window),
                        "remaining": s.remaining,
                        "limit": s.limit,
                        "current": s.current,
                    })
                })
                .collect();
            serde_json::json!({
                "channel": channel,
                "windows": window_status,
            })
        })
        .collect();

    // Campaign definitions
    let campaign_list: Vec<serde_json::Value> = campaigns::all_campaigns()
        .iter()
        .map(|c| {
            serde_json::json!({
                "name": c.name,
                "cron": c.cron,
                "enabled": c.enabled,
                "channels": c.channels.iter().map(|t| &t.channel).collect::<Vec<_>>(),
            })
        })
        .collect();

    Json(serde_json::json!({
        "service": "marketing",
        "campaigns": campaign_list,
        "rate_limits": rate_limits,
    }))
}

pub async fn rate_limits(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let all = state.rate_limiter.all_statuses().await;
    let limits: Vec<serde_json::Value> = all
        .into_iter()
        .map(|(channel, statuses)| {
            let window_info: Vec<serde_json::Value> = statuses
                .into_iter()
                .map(|s| {
                    let usage_pct = if s.limit > 0 {
                        ((s.limit - s.remaining) as f64 / s.limit as f64 * 100.0).round()
                    } else {
                        0.0
                    };
                    serde_json::json!({
                        "window": format!("{:?}", s.window),
                        "remaining": s.remaining,
                        "limit": s.limit,
                        "current": s.current,
                        "usage_pct": usage_pct,
                    })
                })
                .collect();
            serde_json::json!({
                "channel": channel,
                "windows": window_info,
            })
        })
        .collect();

    Json(serde_json::json!({ "rate_limits": limits }))
}

pub async fn history(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let rows = sqlx::query_as::<_, PublishLogRow>(
        "SELECT campaign, channel, success, message_id, url, error, published_at
         FROM marketing_publish_log
         ORDER BY published_at DESC
         LIMIT 50",
    )
    .fetch_all(&state.db_pool)
    .await;

    match rows {
        Ok(rows) => {
            let entries: Vec<serde_json::Value> = rows
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
            Json(serde_json::json!({ "history": entries }))
        }
        Err(e) => Json(serde_json::json!({
            "error": format!("Database error: {}", e),
            "history": [],
        })),
    }
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
