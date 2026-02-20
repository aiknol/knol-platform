//! Channel adapters for publishing content to various platforms.
//!
//! Channels organized by the Zero-Cost Marketing Plan:
//! - Core: Twitter, LinkedIn, Reddit, Dev.to, GitHub, Email, Blog, HackerNews
//! - Cross-post (Phase 3): Hashnode, Medium
//! - Launch (Phase 2): Product Hunt

pub mod blog;
pub mod devto;
pub mod email;
pub mod github;
pub mod hashnode;
pub mod linkedin;
pub mod medium;
pub mod producthunt;
pub mod reddit;
pub mod twitter;

use serde::{Deserialize, Serialize};

use crate::error::MarketingError;

/// Result of publishing to a channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResult {
    pub channel: String,
    pub success: bool,
    pub message_id: Option<String>,
    pub url: Option<String>,
    pub error: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl PublishResult {
    pub fn success(channel: &str, message_id: Option<String>, url: Option<String>) -> Self {
        Self {
            channel: channel.to_string(),
            success: true,
            message_id,
            url,
            error: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn failure(channel: &str, error: String) -> Self {
        Self {
            channel: channel.to_string(),
            success: false,
            message_id: None,
            url: None,
            error: Some(error),
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn manual(channel: &str, note: &str) -> Self {
        Self {
            channel: channel.to_string(),
            success: true,
            message_id: None,
            url: None,
            error: Some(format!("manual: {}", note)),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Content to publish (varies by channel).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishContent {
    /// Short text (tweets, LinkedIn posts)
    pub text: String,
    /// Title (Reddit, Dev.to, blog)
    pub title: Option<String>,
    /// Long-form body (Dev.to articles, blog posts, emails)
    pub body: Option<String>,
    /// Tags
    pub tags: Vec<String>,
    /// Target subreddit for Reddit
    pub subreddit: Option<String>,
    /// Email subject
    pub subject: Option<String>,
}

/// Dispatch content to the appropriate channel adapter.
pub async fn publish_to_channel(
    channel: &str,
    content: &PublishContent,
    http_client: &reqwest::Client,
    credentials: &crate::config::ChannelCredentials,
) -> Result<PublishResult, MarketingError> {
    match channel {
        "twitter" => twitter::publish(content, http_client, credentials).await,
        "linkedin" => linkedin::publish(content, http_client, credentials).await,
        "reddit" => reddit::publish(content, http_client, credentials).await,
        "devto" => devto::publish(content, http_client, credentials).await,
        "github" => github::publish(content, http_client, credentials).await,
        "email" => email::publish(content, credentials).await,
        "blog" => blog::publish(content).await,
        "hashnode" => hashnode::publish(content, http_client, credentials).await,
        "medium" => medium::publish(content, http_client, credentials).await,
        "producthunt" => producthunt::publish(content, http_client, credentials).await,
        "hackernews" => Ok(PublishResult::manual(
            "hackernews",
            "HN requires manual submission",
        )),
        _ => Err(MarketingError::Channel {
            channel: channel.to_string(),
            message: "Unknown channel".into(),
        }),
    }
}
