//! Dev.to channel adapter — API key → POST /api/articles.

use tracing::{info, warn};

use super::{PublishContent, PublishResult};
use crate::config::ChannelCredentials;
use crate::error::MarketingError;

pub async fn publish(
    content: &PublishContent,
    http_client: &reqwest::Client,
    credentials: &ChannelCredentials,
) -> Result<PublishResult, MarketingError> {
    let api_key = credentials
        .devto_api_key
        .as_ref()
        .ok_or_else(|| MarketingError::Channel {
            channel: "devto".into(),
            message: "Dev.to credentials not configured".into(),
        })?;

    let title = content.title.as_deref().unwrap_or("Untitled");
    let body_markdown = content.body.as_deref().unwrap_or(&content.text);

    let article = serde_json::json!({
        "article": {
            "title": title,
            "body_markdown": body_markdown,
            "published": true,
            "tags": content.tags.iter().take(4).collect::<Vec<_>>(),
        }
    });

    let resp = http_client
        .post("https://dev.to/api/articles")
        .header("api-key", api_key.as_str())
        .header("Content-Type", "application/json")
        .json(&article)
        .send()
        .await
        .map_err(|e| MarketingError::Channel {
            channel: "devto".into(),
            message: e.to_string(),
        })?;

    let status = resp.status();
    let body: serde_json::Value = resp.json().await.unwrap_or_default();

    if status.as_u16() == 201 {
        let article_id = body["id"].as_i64().map(|id| id.to_string());
        let article_url = body["url"].as_str().map(|s| s.to_string());
        info!(
            "Dev.to: published article {}",
            article_url.as_deref().unwrap_or("?")
        );
        Ok(PublishResult::success("devto", article_id, article_url))
    } else if status.as_u16() == 429 {
        warn!("Dev.to: rate limited");
        Err(MarketingError::RateLimited {
            channel: "devto".into(),
            current: 0,
            limit: 0,
            window: "api".into(),
        })
    } else {
        let err_msg = body["error"]
            .as_str()
            .unwrap_or("Unknown error")
            .to_string();
        warn!("Dev.to: HTTP {} — {}", status, err_msg);
        Ok(PublishResult::failure(
            "devto",
            format!("HTTP {}: {}", status, err_msg),
        ))
    }
}
