//! Marketing service error types.

use axum::{http::StatusCode, response::IntoResponse, Json};

#[derive(Debug, thiserror::Error)]
pub enum MarketingError {
    #[error("Rate limited: {channel} ({current}/{limit} in {window})")]
    RateLimited {
        channel: String,
        current: u64,
        limit: u64,
        window: String,
    },

    #[error("Channel error ({channel}): {message}")]
    Channel { channel: String, message: String },

    #[error("Content generation error: {0}")]
    ContentGeneration(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Campaign not found: {0}")]
    CampaignNotFound(String),

    #[error("Campaign paused: {0}")]
    CampaignPaused(String),

    #[error("HTTP client error: {0}")]
    Http(String),
}

impl IntoResponse for MarketingError {
    fn into_response(self) -> axum::response::Response {
        let (status, msg) = match &self {
            MarketingError::RateLimited { .. } => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            MarketingError::CampaignNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            MarketingError::CampaignPaused(_) => (StatusCode::CONFLICT, self.to_string()),
            MarketingError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        (status, Json(serde_json::json!({"error": msg}))).into_response()
    }
}

impl From<sqlx::Error> for MarketingError {
    fn from(e: sqlx::Error) -> Self {
        MarketingError::Database(e.to_string())
    }
}

impl From<reqwest::Error> for MarketingError {
    fn from(e: reqwest::Error) -> Self {
        MarketingError::Http(e.to_string())
    }
}
