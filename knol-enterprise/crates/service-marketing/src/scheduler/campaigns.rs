//! Campaign definitions and execution logic.
//!
//! Three campaigns run on cron schedules:
//! - **Daily** (2pm UTC): 1 tweet, rotating category
//! - **Weekly** (Tue 3pm UTC): Blog + Dev.to + LinkedIn + Reddit
//! - **Monthly** (1st 4pm UTC): Newsletter + GitHub metadata + tweet thread

use std::sync::Arc;
use chrono::Datelike;
use tokio::time::Duration;
use tracing::{error, info, warn};

use crate::channels::{self, PublishContent};
use crate::content::generator;
use crate::error::MarketingError;
use crate::state::AppState;

/// Campaign definition.
#[derive(Debug, Clone)]
pub struct Campaign {
    pub name: String,
    pub cron: String,
    pub channels: Vec<ChannelTask>,
    pub enabled: bool,
}

/// A single publish task within a campaign.
#[derive(Debug, Clone)]
pub struct ChannelTask {
    pub channel: String,
    pub template_category: String,
}

/// Get all campaign definitions.
pub fn all_campaigns() -> Vec<Campaign> {
    vec![
        Campaign {
            name: "daily".to_string(),
            cron: "0 0 14 * * *".to_string(), // 2pm UTC daily
            channels: vec![ChannelTask {
                channel: "twitter".to_string(),
                template_category: "tweet_launch".to_string(), // rotates at runtime
            }],
            enabled: true,
        },
        Campaign {
            name: "weekly".to_string(),
            cron: "0 0 15 * * TUE".to_string(), // Tue 3pm UTC
            channels: vec![
                ChannelTask {
                    channel: "blog".to_string(),
                    template_category: "blog_technical".to_string(),
                },
                ChannelTask {
                    channel: "devto".to_string(),
                    template_category: "devto_tutorial".to_string(),
                },
                ChannelTask {
                    channel: "linkedin".to_string(),
                    template_category: "linkedin_technical".to_string(),
                },
                ChannelTask {
                    channel: "reddit".to_string(),
                    template_category: "reddit_rust".to_string(),
                },
            ],
            enabled: true,
        },
        Campaign {
            name: "monthly".to_string(),
            cron: "0 0 16 1 * *".to_string(), // 1st of month, 4pm UTC
            channels: vec![
                ChannelTask {
                    channel: "email".to_string(),
                    template_category: "email_weekly".to_string(),
                },
                ChannelTask {
                    channel: "github".to_string(),
                    template_category: "blog_launch".to_string(),
                },
                ChannelTask {
                    channel: "twitter".to_string(),
                    template_category: "tweet_technical".to_string(),
                },
            ],
            enabled: true,
        },
    ]
}

/// Rotate tweet category based on day of year.
fn rotate_tweet_category() -> &'static str {
    let day = chrono::Utc::now().ordinal0();
    match day % 3 {
        0 => "tweet_launch",
        1 => "tweet_technical",
        _ => "tweet_comparison",
    }
}

/// Execute a campaign by name.
pub async fn execute_campaign(
    state: &Arc<AppState>,
    campaign_name: &str,
    dry_run: bool,
) -> Result<Vec<CampaignResult>, MarketingError> {
    let campaigns = all_campaigns();
    let campaign = campaigns
        .iter()
        .find(|c| c.name == campaign_name)
        .ok_or_else(|| MarketingError::CampaignNotFound(campaign_name.to_string()))?;

    if !campaign.enabled {
        return Err(MarketingError::CampaignPaused(campaign_name.to_string()));
    }

    info!("Campaign '{}': starting (dry_run={})", campaign_name, dry_run);

    let mut results = Vec::new();

    for task in &campaign.channels {
        // Rotate tweet category for daily campaign
        let category = if task.channel == "twitter" && campaign_name == "daily" {
            rotate_tweet_category().to_string()
        } else {
            task.template_category.clone()
        };

        // 1. Check rate limit BEFORE generating content
        let statuses = state.rate_limiter.check_status(&task.channel).await;
        if !statuses.is_empty() {
            let blocked = statuses.iter().any(|s| !s.allowed);
            if blocked {
                warn!(
                    "Campaign '{}': skipping {} — rate limited",
                    campaign_name, task.channel
                );
                results.push(CampaignResult {
                    channel: task.channel.clone(),
                    category: category.clone(),
                    status: "skipped_rate_limited".to_string(),
                    error: None,
                });
                continue;
            }
        }

        // 2. Generate content
        let content = match generator::generate(&task.channel, &category) {
            Ok(c) => c,
            Err(e) => {
                error!(
                    "Campaign '{}': content generation failed for {} — {}",
                    campaign_name, task.channel, e
                );
                results.push(CampaignResult {
                    channel: task.channel.clone(),
                    category: category.clone(),
                    status: "error".to_string(),
                    error: Some(e.to_string()),
                });
                continue;
            }
        };

        if dry_run {
            info!(
                "Campaign '{}': [DRY RUN] would publish to {} — '{}'",
                campaign_name,
                task.channel,
                &content.text[..content.text.len().min(80)]
            );
            results.push(CampaignResult {
                channel: task.channel.clone(),
                category,
                status: "dry_run".to_string(),
                error: None,
            });
            continue;
        }

        // 3. Increment rate limit
        match state.rate_limiter.check_and_increment(&task.channel).await {
            Ok((true, _)) => {}
            Ok((false, _)) => {
                warn!(
                    "Campaign '{}': rate limit reached for {} at publish time",
                    campaign_name, task.channel
                );
                results.push(CampaignResult {
                    channel: task.channel.clone(),
                    category,
                    status: "skipped_rate_limited".to_string(),
                    error: None,
                });
                continue;
            }
            Err(e) => {
                // No limits configured — proceed
                info!("Campaign '{}': no rate limit for {} ({})", campaign_name, task.channel, e);
            }
        }

        // 4. Publish
        let result = channels::publish_to_channel(
            &task.channel,
            &content,
            &state.http_client,
            &state.credentials,
        )
        .await;

        match result {
            Ok(publish_result) => {
                // Log to database
                if let Err(e) = log_publish(&state.db_pool, campaign_name, &publish_result).await {
                    warn!("Failed to log publish result: {}", e);
                }

                results.push(CampaignResult {
                    channel: task.channel.clone(),
                    category,
                    status: if publish_result.success {
                        "success".to_string()
                    } else {
                        "failed".to_string()
                    },
                    error: publish_result.error,
                });
            }
            Err(e) => {
                error!(
                    "Campaign '{}': publish error for {} — {}",
                    campaign_name, task.channel, e
                );
                results.push(CampaignResult {
                    channel: task.channel.clone(),
                    category,
                    status: "error".to_string(),
                    error: Some(e.to_string()),
                });
            }
        }

        // 5. Delay between channels (3 seconds)
        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    info!(
        "Campaign '{}': completed — {} tasks",
        campaign_name,
        results.len()
    );
    Ok(results)
}

/// Result of a single channel task within a campaign.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CampaignResult {
    pub channel: String,
    pub category: String,
    pub status: String,
    pub error: Option<String>,
}

/// Log a publish result to the database.
async fn log_publish(
    pool: &sqlx::PgPool,
    campaign: &str,
    result: &channels::PublishResult,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO marketing_publish_log
            (campaign, channel, success, message_id, url, error, published_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(campaign)
    .bind(&result.channel)
    .bind(result.success)
    .bind(&result.message_id)
    .bind(&result.url)
    .bind(&result.error)
    .bind(result.timestamp)
    .execute(pool)
    .await?;

    Ok(())
}
