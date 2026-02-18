use axum::{extract::Query, Json};
use serde::Deserialize;

use crate::content::generator;

#[derive(Deserialize)]
pub struct PreviewParams {
    pub channel: String,
    #[serde(default = "default_category")]
    pub template: String,
}

fn default_category() -> String {
    "tweet_launch".to_string()
}

pub async fn preview(Query(params): Query<PreviewParams>) -> Json<serde_json::Value> {
    match generator::preview(&params.channel, &params.template) {
        Ok(content) => Json(serde_json::json!({
            "channel": params.channel,
            "template": params.template,
            "content": {
                "text": content.text,
                "title": content.title,
                "body": content.body,
                "tags": content.tags,
                "subreddit": content.subreddit,
                "subject": content.subject,
            }
        })),
        Err(e) => Json(serde_json::json!({
            "error": e.to_string(),
        })),
    }
}
