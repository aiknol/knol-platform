//! GitHub channel adapter — PAT → releases, metadata, stats.

use tracing::{info, warn};

use super::{PublishContent, PublishResult};
use crate::config::ChannelCredentials;
use crate::error::MarketingError;

const REPO_OWNER: &str = "knol-memory";
const REPO_NAME: &str = "memorylayer";

pub async fn publish(
    content: &PublishContent,
    http_client: &reqwest::Client,
    credentials: &ChannelCredentials,
) -> Result<PublishResult, MarketingError> {
    let token = credentials.github_token.as_ref().ok_or_else(|| {
        MarketingError::Channel {
            channel: "github".into(),
            message: "GitHub credentials not configured".into(),
        }
    })?;

    // GitHub adapter supports two modes:
    // 1. Create a release (if title looks like a version tag)
    // 2. Update repo metadata (description, topics)
    let title = content.title.as_deref().unwrap_or("");

    if title.starts_with('v') || title.starts_with("release") {
        create_release(http_client, token, content).await
    } else {
        update_repo_metadata(http_client, token, content).await
    }
}

async fn create_release(
    http_client: &reqwest::Client,
    token: &str,
    content: &PublishContent,
) -> Result<PublishResult, MarketingError> {
    let tag = content.title.as_deref().unwrap_or("v0.0.0");
    let body_text = content.body.as_deref().unwrap_or(&content.text);

    let payload = serde_json::json!({
        "tag_name": tag,
        "name": tag,
        "body": body_text,
        "draft": false,
        "prerelease": false,
    });

    let url = format!(
        "https://api.github.com/repos/{}/{}/releases",
        REPO_OWNER, REPO_NAME
    );

    let resp = http_client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "knol-marketing/0.1.0")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .json(&payload)
        .send()
        .await
        .map_err(|e| MarketingError::Channel {
            channel: "github".into(),
            message: e.to_string(),
        })?;

    let status = resp.status();
    let body: serde_json::Value = resp.json().await.unwrap_or_default();

    if status.as_u16() == 201 {
        let release_id = body["id"].as_i64().map(|id| id.to_string());
        let html_url = body["html_url"].as_str().map(|s| s.to_string());
        info!("GitHub: created release {}", tag);
        Ok(PublishResult::success("github", release_id, html_url))
    } else {
        let err_msg = body["message"]
            .as_str()
            .unwrap_or("Unknown error")
            .to_string();
        warn!("GitHub: HTTP {} — {}", status, err_msg);
        Ok(PublishResult::failure(
            "github",
            format!("HTTP {}: {}", status, err_msg),
        ))
    }
}

async fn update_repo_metadata(
    http_client: &reqwest::Client,
    token: &str,
    content: &PublishContent,
) -> Result<PublishResult, MarketingError> {
    let url = format!(
        "https://api.github.com/repos/{}/{}",
        REPO_OWNER, REPO_NAME
    );

    let mut payload = serde_json::Map::new();

    if !content.text.is_empty() {
        payload.insert(
            "description".to_string(),
            serde_json::Value::String(content.text.clone()),
        );
    }

    if !content.tags.is_empty() {
        // Update topics via separate endpoint
        let topics_url = format!("{}/topics", url);
        let topics_payload = serde_json::json!({
            "names": content.tags.iter().take(20).collect::<Vec<_>>()
        });

        let _ = http_client
            .put(&topics_url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "knol-marketing/0.1.0")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&topics_payload)
            .send()
            .await;
    }

    if payload.is_empty() {
        return Ok(PublishResult::success("github", None, None));
    }

    let resp = http_client
        .patch(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "knol-marketing/0.1.0")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .json(&serde_json::Value::Object(payload))
        .send()
        .await
        .map_err(|e| MarketingError::Channel {
            channel: "github".into(),
            message: e.to_string(),
        })?;

    let status = resp.status();
    if status.is_success() {
        info!("GitHub: updated repo metadata");
        Ok(PublishResult::success("github", None, None))
    } else {
        let text = resp.text().await.unwrap_or_default();
        warn!("GitHub: HTTP {} — {}", status, &text[..text.len().min(200)]);
        Ok(PublishResult::failure(
            "github",
            format!("HTTP {}", status),
        ))
    }
}
