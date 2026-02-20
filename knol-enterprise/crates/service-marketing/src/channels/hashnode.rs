//! Hashnode channel adapter — cross-posting via Hashnode API.
//!
//! Hashnode uses a GraphQL API for article creation.
//! Requires: HASHNODE_API_KEY and HASHNODE_PUBLICATION_ID credentials.
//!
//! SECURITY: Uses GraphQL variables (not string interpolation) to prevent injection.

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
        .hashnode_api_key
        .as_ref()
        .ok_or_else(|| MarketingError::Channel {
            channel: "hashnode".into(),
            message: "Hashnode API key not configured".into(),
        })?;

    let publication_id = credentials
        .hashnode_publication_id
        .as_ref()
        .ok_or_else(|| MarketingError::Channel {
            channel: "hashnode".into(),
            message: "Hashnode publication ID not configured".into(),
        })?;

    let title = content.title.as_deref().unwrap_or("Untitled");
    let body_markdown = content.body.as_deref().unwrap_or(&content.text);
    let tags: Vec<serde_json::Value> = content
        .tags
        .iter()
        .take(5)
        .map(|t| serde_json::json!({"slug": t, "name": t}))
        .collect();

    // SECURITY: Use parameterized GraphQL variables — never interpolate user
    // content into the query string to prevent GraphQL injection attacks.
    let query = r#"mutation PublishPost($input: PublishPostInput!) {
        publishPost(input: $input) {
            post {
                id
                url
                title
            }
        }
    }"#;

    let variables = serde_json::json!({
        "input": {
            "title": title,
            "contentMarkdown": body_markdown,
            "publicationId": publication_id,
            "tags": tags
        }
    });

    let resp = http_client
        .post("https://gql.hashnode.com")
        .header("Authorization", api_key.as_str())
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "query": query,
            "variables": variables,
        }))
        .send()
        .await
        .map_err(|e| MarketingError::Channel {
            channel: "hashnode".into(),
            message: e.to_string(),
        })?;

    let status = resp.status();
    let body: serde_json::Value = resp.json().await.unwrap_or_default();

    if status.is_success() && body.get("errors").is_none() {
        let post = &body["data"]["publishPost"]["post"];
        let post_id = post["id"].as_str().map(|s| s.to_string());
        let post_url = post["url"].as_str().map(|s| s.to_string());
        info!(
            "Hashnode: published article {}",
            post_url.as_deref().unwrap_or("?")
        );
        Ok(PublishResult::success("hashnode", post_id, post_url))
    } else if status.as_u16() == 429 {
        warn!("Hashnode: rate limited");
        Err(MarketingError::RateLimited {
            channel: "hashnode".into(),
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
        warn!("Hashnode: HTTP {} — {}", status, err_msg);
        Ok(PublishResult::failure(
            "hashnode",
            format!("HTTP {}: {}", status, err_msg),
        ))
    }
}
