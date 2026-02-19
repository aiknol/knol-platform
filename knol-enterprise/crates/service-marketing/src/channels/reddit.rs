//! Reddit channel adapter — OAuth2 password grant → POST /api/submit.

use tracing::{info, warn};

use super::{PublishContent, PublishResult};
use crate::config::ChannelCredentials;
use crate::error::MarketingError;

/// Authenticate with Reddit using password grant.
async fn authenticate(
    http_client: &reqwest::Client,
    credentials: &ChannelCredentials,
) -> Result<String, MarketingError> {
    let client_id =
        credentials
            .reddit_client_id
            .as_ref()
            .ok_or_else(|| MarketingError::Channel {
                channel: "reddit".into(),
                message: "Reddit credentials not configured".into(),
            })?;
    let client_secret = credentials.reddit_client_secret.as_deref().unwrap_or("");
    let username = credentials.reddit_username.as_deref().unwrap_or("");
    let password = credentials.reddit_password.as_deref().unwrap_or("");

    let resp = http_client
        .post("https://www.reddit.com/api/v1/access_token")
        .basic_auth(client_id, Some(client_secret))
        .header("User-Agent", "knol-marketing/0.1.0")
        .form(&[
            ("grant_type", "password"),
            ("username", username),
            ("password", password),
        ])
        .send()
        .await
        .map_err(|e| MarketingError::Channel {
            channel: "reddit".into(),
            message: format!("Auth failed: {}", e),
        })?;

    let body: serde_json::Value = resp.json().await.map_err(|e| MarketingError::Channel {
        channel: "reddit".into(),
        message: format!("Auth parse error: {}", e),
    })?;

    body["access_token"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| MarketingError::Channel {
            channel: "reddit".into(),
            message: format!("No access_token in response: {}", body),
        })
}

pub async fn publish(
    content: &PublishContent,
    http_client: &reqwest::Client,
    credentials: &ChannelCredentials,
) -> Result<PublishResult, MarketingError> {
    let token = authenticate(http_client, credentials).await?;

    let subreddit = content.subreddit.as_deref().unwrap_or("rust");
    let title = content.title.as_deref().unwrap_or(&content.text);

    let params = vec![
        ("sr", subreddit.to_string()),
        ("kind", "self".to_string()),
        ("title", title.to_string()),
        (
            "text",
            content.body.as_deref().unwrap_or(&content.text).to_string(),
        ),
        ("resubmit", "true".to_string()),
    ];

    let resp = http_client
        .post("https://oauth.reddit.com/api/submit")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "knol-marketing/0.1.0")
        .form(&params)
        .send()
        .await
        .map_err(|e| MarketingError::Channel {
            channel: "reddit".into(),
            message: e.to_string(),
        })?;

    let status = resp.status();
    let body: serde_json::Value = resp.json().await.unwrap_or_default();

    let post_url = body["json"]["data"]["url"].as_str().map(|s| s.to_string());
    let post_id = body["json"]["data"]["id"].as_str().map(|s| s.to_string());

    if body["success"].as_bool().unwrap_or(false) || post_url.is_some() {
        info!("Reddit: posted to r/{}", subreddit);
        Ok(PublishResult::success("reddit", post_id, post_url))
    } else if status.as_u16() == 429 {
        warn!("Reddit: rate limited");
        Err(MarketingError::RateLimited {
            channel: "reddit".into(),
            current: 0,
            limit: 0,
            window: "api".into(),
        })
    } else {
        let errors = &body["json"]["errors"];
        let err_msg = if errors.is_array() && !errors.as_array().unwrap().is_empty() {
            format!("{}", errors)
        } else {
            format!("HTTP {}", status)
        };
        warn!("Reddit: {}", err_msg);
        Ok(PublishResult::failure("reddit", err_msg))
    }
}
