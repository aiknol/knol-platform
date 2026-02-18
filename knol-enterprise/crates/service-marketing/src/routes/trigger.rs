use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

use crate::scheduler::campaigns;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct TriggerParams {
    #[serde(default)]
    pub dry_run: bool,
}

pub async fn trigger(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(params): Query<TriggerParams>,
) -> Json<serde_json::Value> {
    match campaigns::execute_campaign(&state, &name, params.dry_run).await {
        Ok(results) => Json(serde_json::json!({
            "campaign": name,
            "dry_run": params.dry_run,
            "results": results,
        })),
        Err(e) => Json(serde_json::json!({
            "campaign": name,
            "error": e.to_string(),
        })),
    }
}
