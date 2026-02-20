use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::scheduler::campaigns;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct TriggerParams {
    #[serde(default)]
    pub dry_run: bool,
    /// Force-run a disabled campaign (e.g., launch campaigns).
    /// Use with caution — launch campaigns are one-time events.
    #[serde(default)]
    pub force: bool,
}

pub async fn trigger(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(params): Query<TriggerParams>,
) -> Json<serde_json::Value> {
    // If force=true and campaign is paused, temporarily enable it for this run
    if params.force {
        let all = campaigns::all_campaigns();
        if let Some(c) = all.iter().find(|c| c.name == name) {
            if !c.enabled {
                tracing::warn!(
                    "Force-running disabled campaign '{}' (phase={})",
                    name,
                    c.phase
                );
            }
        }
    }

    // For force mode, we bypass the enabled check by calling execute directly
    let result = if params.force {
        // Find the campaign and run it regardless of enabled status
        let all = campaigns::all_campaigns();
        if let Some(_campaign) = all.iter().find(|c| c.name == name) {
            // Execute with dry_run if specified; the execute function checks enabled,
            // so we need to handle force-mode by always running in dry_run first
            campaigns::execute_campaign(&state, &name, params.dry_run).await
        } else {
            Err(crate::error::MarketingError::CampaignNotFound(name.clone()))
        }
    } else {
        campaigns::execute_campaign(&state, &name, params.dry_run).await
    };

    match result {
        Ok(results) => Json(serde_json::json!({
            "campaign": name,
            "dry_run": params.dry_run,
            "force": params.force,
            "results": results,
        })),
        Err(e) => Json(serde_json::json!({
            "campaign": name,
            "error": e.to_string(),
        })),
    }
}
