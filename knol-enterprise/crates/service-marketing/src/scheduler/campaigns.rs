//! Campaign definitions aligned with the Zero-Cost Marketing Plan.
//!
//! Campaigns are organized by plan phases:
//!
//! **Phase 3 (Content Engine):**
//! - **daily_twitter** (2pm UTC): Day-of-week rotation (Mon=tip, Tue=benchmark,
//!   Wed=showcase, Thu=architecture, Fri=community)
//! - **weekly_content** (Tue 3pm UTC): Blog + cross-post to Dev.to, Hashnode, Medium +
//!   LinkedIn + Reddit (rotating subs)
//!
//! **Phase 2 (Launch Week) — one-time campaigns:**
//! - **launch_hn**: Hacker News Show HN post
//! - **launch_reddit**: Reddit blitz across 4 subreddits
//! - **launch_devto**: Dev.to technical article
//! - **launch_twitter**: Twitter thread
//! - **launch_producthunt**: Product Hunt listing
//!
//! **Phase 4 & 5 (Community & Conversion):**
//! - **monthly_newsletter**: Email + GitHub metadata + community tweet
//! - **mcp_content**: MCP ecosystem content (bi-weekly)
//! - **seo_content**: SEO-targeted blog posts (weekly)

use chrono::Datelike;
use std::sync::Arc;
use tokio::time::Duration;
use tracing::{error, info, warn};

use crate::channels;
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
    /// Which phase of the zero-cost marketing plan this campaign belongs to.
    pub phase: MarketingPhase,
    /// Human-readable description of the campaign's purpose.
    pub description: String,
}

/// A single publish task within a campaign.
#[derive(Debug, Clone)]
pub struct ChannelTask {
    pub channel: String,
    pub template_category: String,
}

/// Marketing plan phase for campaign categorization.
#[derive(Debug, Clone, serde::Serialize)]
pub enum MarketingPhase {
    /// Phase 2: Launch Week (one-time campaigns)
    Launch,
    /// Phase 3: Content Engine (recurring)
    ContentEngine,
    /// Phase 4: Community & Ecosystem
    Community,
    /// Phase 5: Conversion & Growth
    Conversion,
}

impl std::fmt::Display for MarketingPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketingPhase::Launch => write!(f, "launch"),
            MarketingPhase::ContentEngine => write!(f, "content_engine"),
            MarketingPhase::Community => write!(f, "community"),
            MarketingPhase::Conversion => write!(f, "conversion"),
        }
    }
}

impl MarketingPhase {
    fn from_str_loose(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "launch" => MarketingPhase::Launch,
            "community" => MarketingPhase::Community,
            "conversion" => MarketingPhase::Conversion,
            _ => MarketingPhase::ContentEngine,
        }
    }
}

/// Get all campaign definitions aligned with the Zero-Cost Marketing Plan.
pub fn all_campaigns() -> Vec<Campaign> {
    vec![
        // ── Phase 3: Content Engine (recurring) ──────────────────
        Campaign {
            name: "daily_twitter".to_string(),
            cron: "0 0 14 * * *".to_string(), // 2pm UTC daily
            channels: vec![ChannelTask {
                channel: "twitter".to_string(),
                template_category: "tweet_tip".to_string(), // rotated by day-of-week at runtime
            }],
            enabled: true,
            phase: MarketingPhase::ContentEngine,
            description: "Daily tweet with day-of-week rotation: Mon=tip, Tue=benchmark, Wed=showcase, Thu=architecture, Fri=community".to_string(),
        },
        Campaign {
            name: "weekly_content".to_string(),
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
                    channel: "hashnode".to_string(),
                    template_category: "devto_tutorial".to_string(), // cross-post
                },
                ChannelTask {
                    channel: "medium".to_string(),
                    template_category: "devto_tutorial".to_string(), // cross-post
                },
                ChannelTask {
                    channel: "linkedin".to_string(),
                    template_category: "linkedin_technical".to_string(),
                },
                ChannelTask {
                    channel: "reddit".to_string(),
                    template_category: "reddit_rust".to_string(), // rotated at runtime
                },
            ],
            enabled: true,
            phase: MarketingPhase::ContentEngine,
            description: "Weekly blog + cross-post to Dev.to/Hashnode/Medium + LinkedIn + Reddit".to_string(),
        },

        // ── Phase 2: Launch Week (one-time, disabled by default) ─
        Campaign {
            name: "launch_hn".to_string(),
            cron: "0 0 13 * * *".to_string(), // 8am ET = 1pm UTC (peak HN)
            channels: vec![ChannelTask {
                channel: "hackernews".to_string(),
                template_category: "hn_show".to_string(),
            }],
            enabled: false, // Enable manually for launch day
            phase: MarketingPhase::Launch,
            description: "Launch Day 1: Show HN post (8am ET Tuesday/Wednesday)".to_string(),
        },
        Campaign {
            name: "launch_reddit".to_string(),
            cron: "0 0 15 * * *".to_string(),
            channels: vec![
                ChannelTask {
                    channel: "reddit".to_string(),
                    template_category: "reddit_rust".to_string(),
                },
                ChannelTask {
                    channel: "reddit".to_string(),
                    template_category: "reddit_local_llama".to_string(),
                },
                ChannelTask {
                    channel: "reddit".to_string(),
                    template_category: "reddit_ml".to_string(),
                },
                ChannelTask {
                    channel: "reddit".to_string(),
                    template_category: "reddit_selfhosted".to_string(),
                },
            ],
            enabled: false,
            phase: MarketingPhase::Launch,
            description: "Launch Day 2: Reddit blitz — r/rust, r/LocalLLaMA, r/MachineLearning, r/selfhosted".to_string(),
        },
        Campaign {
            name: "launch_devto".to_string(),
            cron: "0 0 14 * * *".to_string(),
            channels: vec![
                ChannelTask {
                    channel: "devto".to_string(),
                    template_category: "devto_rust_rewrite".to_string(),
                },
                ChannelTask {
                    channel: "hashnode".to_string(),
                    template_category: "devto_rust_rewrite".to_string(),
                },
            ],
            enabled: false,
            phase: MarketingPhase::Launch,
            description: "Launch Day 3: Dev.to + Hashnode article — 'Why We Rewrote in Rust'".to_string(),
        },
        Campaign {
            name: "launch_twitter".to_string(),
            cron: "0 0 14 * * *".to_string(),
            channels: vec![ChannelTask {
                channel: "twitter".to_string(),
                template_category: "tweet_thread_launch".to_string(),
            }],
            enabled: false,
            phase: MarketingPhase::Launch,
            description: "Launch Day 4: Twitter/X launch thread".to_string(),
        },
        Campaign {
            name: "launch_producthunt".to_string(),
            cron: "0 0 12 * * *".to_string(), // Midnight PT = noon UTC
            channels: vec![ChannelTask {
                channel: "producthunt".to_string(),
                template_category: "producthunt_launch".to_string(),
            }],
            enabled: false,
            phase: MarketingPhase::Launch,
            description: "Launch Day 5: Product Hunt listing (Category: Developer Tools > AI)".to_string(),
        },

        // ── Phase 4: Community & Ecosystem ───────────────────────
        Campaign {
            name: "mcp_content".to_string(),
            cron: "0 0 15 * * WED".to_string(), // Every Wed
            channels: vec![
                ChannelTask {
                    channel: "devto".to_string(),
                    template_category: "devto_mcp".to_string(),
                },
                ChannelTask {
                    channel: "blog".to_string(),
                    template_category: "blog_integration".to_string(),
                },
            ],
            enabled: true,
            phase: MarketingPhase::Community,
            description: "MCP ecosystem content: tutorials, integration guides".to_string(),
        },

        // ── Phase 5: Conversion & Growth ─────────────────────────
        Campaign {
            name: "monthly_newsletter".to_string(),
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
            phase: MarketingPhase::Conversion,
            description: "Monthly newsletter + GitHub metadata update + summary tweet".to_string(),
        },
        Campaign {
            name: "seo_content".to_string(),
            cron: "0 0 14 * * THU".to_string(), // Every Thursday
            channels: vec![
                ChannelTask {
                    channel: "blog".to_string(),
                    template_category: "blog_seo".to_string(),
                },
            ],
            enabled: true,
            phase: MarketingPhase::Conversion,
            description: "SEO-targeted blog posts: 'AI memory layer', 'Mem0 alternative', 'context engineering'".to_string(),
        },
        Campaign {
            name: "weekly_digest".to_string(),
            cron: "0 0 10 * * FRI".to_string(), // Fri 10am UTC
            channels: vec![
                ChannelTask {
                    channel: "email".to_string(),
                    template_category: "email_digest".to_string(),
                },
            ],
            enabled: true,
            phase: MarketingPhase::Conversion,
            description: "Weekly usage digest email for self-hosted users (opt-in)".to_string(),
        },
    ]
}

/// Backward-compatible alias: returns all enabled campaigns.
pub fn enabled_campaigns() -> Vec<Campaign> {
    all_campaigns().into_iter().filter(|c| c.enabled).collect()
}

fn campaign_alias(name: &str) -> &str {
    match name {
        "daily" => "daily_twitter",
        "weekly" => "weekly_content",
        "monthly" => "monthly_newsletter",
        _ => name,
    }
}

fn default_template_category(campaign_name: &str, channel: &str) -> String {
    match (campaign_alias(campaign_name), channel) {
        ("daily_twitter", "twitter") => "tweet_tip".to_string(),
        ("weekly_content", "blog") => "blog_technical".to_string(),
        ("weekly_content", "devto")
        | ("weekly_content", "hashnode")
        | ("weekly_content", "medium") => "devto_tutorial".to_string(),
        ("weekly_content", "linkedin") => "linkedin_technical".to_string(),
        ("weekly_content", "reddit") => "reddit_rust".to_string(),
        ("monthly_newsletter", "email") => "email_weekly".to_string(),
        ("monthly_newsletter", "github") => "blog_launch".to_string(),
        ("monthly_newsletter", "twitter") => "tweet_technical".to_string(),
        (_, "twitter") => "tweet_launch".to_string(),
        (_, "blog") => "blog_technical".to_string(),
        (_, "reddit") => "reddit_rust".to_string(),
        (_, "linkedin") => "linkedin_technical".to_string(),
        (_, "devto") => "devto_tutorial".to_string(),
        (_, "email") => "email_weekly".to_string(),
        (_, "hackernews") => "hn_show".to_string(),
        (_, "producthunt") => "producthunt_launch".to_string(),
        (_, "github") => "blog_launch".to_string(),
        (_, _) => "tweet_launch".to_string(),
    }
}

#[derive(sqlx::FromRow)]
struct DbCampaignRow {
    name: String,
    cron: String,
    channels: Vec<String>,
    enabled: bool,
    phase: String,
    description: String,
}

async fn has_column(
    pool: &sqlx::PgPool,
    table: &str,
    column: &str,
) -> Result<bool, MarketingError> {
    let exists = sqlx::query_scalar::<_, i64>(
        r#"SELECT 1
           FROM information_schema.columns
           WHERE table_schema = 'public'
             AND table_name = $1
             AND column_name = $2
           LIMIT 1"#,
    )
    .bind(table)
    .bind(column)
    .fetch_optional(pool)
    .await?;

    Ok(exists.is_some())
}

/// Load campaigns from DB; fallback to compiled defaults if table is unavailable/empty.
pub async fn load_campaigns(state: &Arc<AppState>) -> Vec<Campaign> {
    let pool = &state.db_pool;
    let has_phase = match has_column(pool, "marketing_campaigns", "phase").await {
        Ok(v) => v,
        Err(e) => {
            warn!(
                "Failed to inspect marketing_campaigns.phase: {}; using defaults",
                e
            );
            return all_campaigns();
        }
    };
    let has_description = match has_column(pool, "marketing_campaigns", "description").await {
        Ok(v) => v,
        Err(e) => {
            warn!(
                "Failed to inspect marketing_campaigns.description: {}; using defaults",
                e
            );
            return all_campaigns();
        }
    };

    let phase_expr = if has_phase {
        "phase"
    } else {
        "'content_engine'::text AS phase"
    };
    let description_expr = if has_description {
        "description"
    } else {
        "''::text AS description"
    };

    let sql = format!(
        "SELECT name, cron, channels, enabled, {}, {} FROM marketing_campaigns ORDER BY name",
        phase_expr, description_expr
    );

    match sqlx::query_as::<_, DbCampaignRow>(&sql)
        .fetch_all(pool)
        .await
    {
        Ok(rows) if !rows.is_empty() => rows
            .into_iter()
            .map(|r| Campaign {
                name: r.name.clone(),
                cron: r.cron,
                channels: r
                    .channels
                    .into_iter()
                    .map(|ch| ChannelTask {
                        template_category: default_template_category(&r.name, &ch),
                        channel: ch,
                    })
                    .collect(),
                enabled: r.enabled,
                phase: MarketingPhase::from_str_loose(&r.phase),
                description: r.description,
            })
            .collect(),
        Ok(_) => {
            warn!("No DB campaigns found; using compiled campaign defaults");
            all_campaigns()
        }
        Err(e) => {
            warn!("Failed to load campaigns from DB: {}; using defaults", e);
            all_campaigns()
        }
    }
}

/// Day-of-week tweet category rotation per the zero-cost plan:
/// Monday=tip, Tuesday=benchmark, Wednesday=showcase, Thursday=architecture, Friday=community.
/// Weekends fall back to engagement tweets.
fn day_of_week_tweet_category() -> &'static str {
    let weekday = chrono::Utc::now().weekday();
    match weekday {
        chrono::Weekday::Mon => "tweet_tip",
        chrono::Weekday::Tue => "tweet_benchmark",
        chrono::Weekday::Wed => "tweet_showcase",
        chrono::Weekday::Thu => "tweet_architecture",
        chrono::Weekday::Fri => "tweet_community",
        chrono::Weekday::Sat | chrono::Weekday::Sun => "tweet_engagement",
    }
}

/// Rotate Reddit subreddit for weekly content campaign.
/// Cycles through: rust → local_llama → ml → selfhosted.
fn rotate_reddit_category() -> &'static str {
    let week = chrono::Utc::now().iso_week().week0();
    match week % 4 {
        0 => "reddit_rust",
        1 => "reddit_local_llama",
        2 => "reddit_ml",
        _ => "reddit_selfhosted",
    }
}

/// Rotate blog template for weekly content: technical → SEO → integration.
fn rotate_blog_category() -> &'static str {
    let week = chrono::Utc::now().iso_week().week0();
    match week % 3 {
        0 => "blog_technical",
        1 => "blog_seo",
        _ => "blog_integration",
    }
}

/// Rotate Dev.to template: tutorial → MCP → rust_rewrite.
fn rotate_devto_category() -> &'static str {
    let week = chrono::Utc::now().iso_week().week0();
    match week % 3 {
        0 => "devto_tutorial",
        1 => "devto_mcp",
        _ => "devto_rust_rewrite",
    }
}

/// Resolve the actual template category for a task, applying rotation logic.
fn resolve_category(campaign_name: &str, task: &ChannelTask) -> String {
    match (campaign_alias(campaign_name), task.channel.as_str()) {
        ("daily_twitter", "twitter") => day_of_week_tweet_category().to_string(),
        ("weekly_content", "reddit") => rotate_reddit_category().to_string(),
        ("weekly_content", "blog") => rotate_blog_category().to_string(),
        ("weekly_content", "devto")
        | ("weekly_content", "hashnode")
        | ("weekly_content", "medium") => rotate_devto_category().to_string(),
        _ => task.template_category.clone(),
    }
}

/// Execute a campaign by name.
pub async fn execute_campaign(
    state: &Arc<AppState>,
    campaign_name: &str,
    dry_run: bool,
) -> Result<Vec<CampaignResult>, MarketingError> {
    execute_campaign_with_options(state, campaign_name, dry_run, false).await
}

pub async fn execute_campaign_with_options(
    state: &Arc<AppState>,
    campaign_name: &str,
    dry_run: bool,
    force_disabled: bool,
) -> Result<Vec<CampaignResult>, MarketingError> {
    let campaigns = load_campaigns(state).await;
    let campaign = campaigns
        .iter()
        .find(|c| {
            c.name == campaign_name || campaign_alias(&c.name) == campaign_alias(campaign_name)
        })
        .ok_or_else(|| MarketingError::CampaignNotFound(campaign_name.to_string()))?;

    if !campaign.enabled && !force_disabled {
        return Err(MarketingError::CampaignPaused(campaign_name.to_string()));
    }

    info!(
        "Campaign '{}' (phase={}): starting (dry_run={})",
        campaign_name, campaign.phase, dry_run
    );

    let mut results = Vec::new();

    for task in &campaign.channels {
        // Resolve category with rotation logic
        let category = resolve_category(campaign_name, task);

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
                "Campaign '{}': [DRY RUN] would publish to {} ({}) — '{}'",
                campaign_name,
                task.channel,
                category,
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
                info!(
                    "Campaign '{}': no rate limit for {} ({})",
                    campaign_name, task.channel, e
                );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_campaigns_have_unique_names() {
        let campaigns = all_campaigns();
        let mut names: Vec<&str> = campaigns.iter().map(|c| c.name.as_str()).collect();
        names.sort();
        names.dedup();
        assert_eq!(
            names.len(),
            campaigns.len(),
            "Duplicate campaign names found"
        );
    }

    #[test]
    fn enabled_campaigns_are_content_engine_or_later() {
        // Launch campaigns should be disabled by default
        let campaigns = all_campaigns();
        for c in &campaigns {
            if matches!(c.phase, MarketingPhase::Launch) {
                assert!(
                    !c.enabled,
                    "Launch campaign '{}' should be disabled by default",
                    c.name
                );
            }
        }
    }

    #[test]
    fn daily_twitter_uses_dow_rotation() {
        let task = ChannelTask {
            channel: "twitter".to_string(),
            template_category: "tweet_tip".to_string(),
        };
        let category = resolve_category("daily_twitter", &task);
        let valid = [
            "tweet_tip",
            "tweet_benchmark",
            "tweet_showcase",
            "tweet_architecture",
            "tweet_community",
            "tweet_engagement",
        ];
        assert!(
            valid.contains(&category.as_str()),
            "Unexpected category: {}",
            category
        );
    }

    #[test]
    fn weekly_reddit_rotates_subs() {
        let task = ChannelTask {
            channel: "reddit".to_string(),
            template_category: "reddit_rust".to_string(),
        };
        let category = resolve_category("weekly_content", &task);
        let valid = [
            "reddit_rust",
            "reddit_local_llama",
            "reddit_ml",
            "reddit_selfhosted",
        ];
        assert!(
            valid.contains(&category.as_str()),
            "Unexpected Reddit category: {}",
            category
        );
    }

    #[test]
    fn non_rotating_campaign_keeps_original_category() {
        let task = ChannelTask {
            channel: "email".to_string(),
            template_category: "email_weekly".to_string(),
        };
        let category = resolve_category("monthly_newsletter", &task);
        assert_eq!(category, "email_weekly");
    }
}
