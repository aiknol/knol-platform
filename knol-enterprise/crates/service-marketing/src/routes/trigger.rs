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
    let result =
        campaigns::execute_campaign_with_options(&state, &name, params.dry_run, params.force).await;

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
