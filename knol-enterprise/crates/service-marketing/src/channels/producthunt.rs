//! Product Hunt channel adapter.
//!
//! Product Hunt launches require manual submission through their website,
//! but the API can be used to prepare maker comments and track submissions.
//! This adapter currently logs the launch content for manual submission.
//!
//! Requires: PRODUCTHUNT_API_TOKEN env var (optional — for future API support).

use tracing::info;

use super::{PublishContent, PublishResult};
use crate::config::ChannelCredentials;
use crate::error::MarketingError;

pub async fn publish(
    content: &PublishContent,
    _http_client: &reqwest::Client,
    _credentials: &ChannelCredentials,
) -> Result<PublishResult, MarketingError> {
    // Product Hunt launches are best done manually through the website.
    // This adapter stages the content and returns a manual-action result.
    let tagline = &content.text;
    let description = content.body.as_deref().unwrap_or("");

    info!(
        "Product Hunt: staged launch content — tagline: '{}' ({} chars), description: {} chars",
        &tagline[..tagline.len().min(80)],
        tagline.len(),
        description.len()
    );

    Ok(PublishResult::manual(
        "producthunt",
        &format!(
            "Product Hunt launch staged. Submit manually at producthunt.com/posts/new. Tagline: '{}'",
            &tagline[..tagline.len().min(80)]
        ),
    ))
}
