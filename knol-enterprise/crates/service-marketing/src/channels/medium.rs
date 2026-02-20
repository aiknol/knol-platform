//! Medium channel adapter — cross-posting via Medium API.
//!
//! Medium's API allows creating posts under a user account.
//! Requires: MEDIUM_TOKEN and MEDIUM_AUTHOR_ID env vars.

use tracing::{info, warn};

use super::{PublishContent, PublishResult};
use crate::config::ChannelCredentials;
use crate::error::MarketingError;

pub async fn publish(
    content: &PublishContent,
    http_client: &reqwest::Client,
    credentials: &ChannelCredentials,
) -> Result<PublishResult, MarketingError> {
    let token = credentials
        .medium_token
        .as_ref()
        .ok_or_else(|| MarketingError::Channel {
            channel: "medium".into(),
            message: "Medium integration token not configured".into(),
        })?;

    let author_id =
        credentials
            .medium_author_id
            .as_ref()
            .ok_or_else(|| MarketingError::Channel {
                channel: "medium".into(),
                message: "Medium author ID not configured".into(),
            })?;

    let title = content.title.as_deref().unwrap_or("Untitled");
    let body_markdown = content.body.as_deref().unwrap_or(&content.text);

    let article = serde_json::json!({
        "title": title,
        "contentFormat": "markdown",
        "content": body_markdown,
        "tags": content.tags.iter().take(5).collect::<Vec<_>>(),
        "publishStatus": "public",
    });

    let url = format!("https://api.medium.com/v1/users/{}/posts", author_id);

    let resp = http_client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&article)
        .send()
        .await
        .map_err(|e| MarketingError::Channel {
            channel: "medium".into(),
            message: e.to_string(),
        })?;

    let status = resp.status();
    let body: serde_json::Value = resp.json().await.unwrap_or_default();

    if status.as_u16() == 201 {
        let post_id = body["data"]["id"].as_str().map(|s| s.to_string());
        let post_url = body["data"]["url"].as_str().map(|s| s.to_string());
        info!(
            "Medium: published article {}",
            post_url.as_deref().unwrap_or("?")
        );
        Ok(PublishResult::success("medium", post_id, post_url))
    } else if status.as_u16() == 429 {
        warn!("Medium: rate limited");
        Err(MarketingError::RateLimited {
            channel: "medium".into(),
            current: 0,
            limit: 0,
            window: "api".into(),
        })
    } else {
        let err_msg = body["errors"]
            .as_array()
            .and_then(|errs| errs.first())
            .and_then(|e| e["message"].as_str())
            .unwrap_or("Unknown error")
            .to_string();
        warn!("Medium: HTTP {} — {}", status, err_msg);
        Ok(PublishResult::failure(
            "medium",
            format!("HTTP {}: {}", status, err_msg),
        ))
    }
}
