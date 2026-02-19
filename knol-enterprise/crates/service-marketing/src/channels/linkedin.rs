//! LinkedIn channel adapter — Bearer token → POST /v2/ugcPosts.

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
        .linkedin_token
        .as_ref()
        .ok_or_else(|| MarketingError::Channel {
            channel: "linkedin".into(),
            message: "LinkedIn credentials not configured".into(),
        })?;
    let person_urn = credentials
        .linkedin_person_urn
        .as_deref()
        .unwrap_or("urn:li:person:unknown");

    let body = serde_json::json!({
        "author": person_urn,
        "lifecycleState": "PUBLISHED",
        "specificContent": {
            "com.linkedin.ugc.ShareContent": {
                "shareCommentary": { "text": content.text },
                "shareMediaCategory": "NONE"
            }
        },
        "visibility": {
            "com.linkedin.ugc.MemberNetworkVisibility": "PUBLIC"
        }
    });

    let resp = http_client
        .post("https://api.linkedin.com/v2/ugcPosts")
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Restli-Protocol-Version", "2.0.0")
        .json(&body)
        .send()
        .await
        .map_err(|e| MarketingError::Channel {
            channel: "linkedin".into(),
            message: e.to_string(),
        })?;

    let status = resp.status();
    let post_id = resp
        .headers()
        .get("x-restli-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if status.as_u16() == 201 {
        info!("LinkedIn: posted update");
        Ok(PublishResult::success("linkedin", post_id, None))
    } else {
        let text = resp.text().await.unwrap_or_default();
        warn!(
            "LinkedIn: HTTP {} — {}",
            status,
            &text[..text.len().min(200)]
        );
        Ok(PublishResult::failure(
            "linkedin",
            format!("HTTP {}: {}", status, &text[..text.len().min(200)]),
        ))
    }
}
