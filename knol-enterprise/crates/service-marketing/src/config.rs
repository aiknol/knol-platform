//! Channel configuration and rate limit definitions.
//! Safety margin: 90% of actual API limits to prevent accidental overages.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Rate limit window type.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum WindowType {
    Minute,
    Hourly,
    Daily,
    Monthly,
}

impl std::fmt::Display for WindowType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowType::Minute => write!(f, "minute"),
            WindowType::Hourly => write!(f, "hourly"),
            WindowType::Daily => write!(f, "daily"),
            WindowType::Monthly => write!(f, "monthly"),
        }
    }
}

impl WindowType {
    /// Duration of the window in seconds.
    pub fn duration_secs(&self) -> i64 {
        match self {
            WindowType::Minute => 60,
            WindowType::Hourly => 3600,
            WindowType::Daily => 86400,
            WindowType::Monthly => 86400 * 30,
        }
    }
}

/// Rate limit for a specific window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateWindow {
    pub window: WindowType,
    pub limit: u64,
}

/// Channel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub name: String,
    pub enabled: bool,
    pub rate_limits: Vec<RateWindow>,
    pub cooldown_between_posts_secs: u64,
}

/// Load all channel configurations with 90% safety margins.
pub fn default_channel_configs() -> HashMap<String, ChannelConfig> {
    let mut configs = HashMap::new();

    configs.insert("twitter".into(), ChannelConfig {
        name: "twitter".into(),
        enabled: true,
        rate_limits: vec![
            RateWindow { window: WindowType::Daily, limit: 45 },   // actual: 50
            RateWindow { window: WindowType::Monthly, limit: 1350 }, // actual: 1500
        ],
        cooldown_between_posts_secs: 2,
    });

    configs.insert("linkedin".into(), ChannelConfig {
        name: "linkedin".into(),
        enabled: true,
        rate_limits: vec![
            RateWindow { window: WindowType::Daily, limit: 22 }, // actual: 25
        ],
        cooldown_between_posts_secs: 5,
    });

    configs.insert("reddit".into(), ChannelConfig {
        name: "reddit".into(),
        enabled: true,
        rate_limits: vec![
            RateWindow { window: WindowType::Daily, limit: 9 },    // actual: 10
            RateWindow { window: WindowType::Minute, limit: 54 },  // actual: 60
        ],
        cooldown_between_posts_secs: 5,
    });

    configs.insert("devto".into(), ChannelConfig {
        name: "devto".into(),
        enabled: true,
        rate_limits: vec![
            RateWindow { window: WindowType::Daily, limit: 27 }, // actual: 30
        ],
        cooldown_between_posts_secs: 3,
    });

    configs.insert("github".into(), ChannelConfig {
        name: "github".into(),
        enabled: true,
        rate_limits: vec![
            RateWindow { window: WindowType::Hourly, limit: 4500 }, // actual: 5000
        ],
        cooldown_between_posts_secs: 1,
    });

    configs.insert("email".into(), ChannelConfig {
        name: "email".into(),
        enabled: true,
        rate_limits: vec![
            RateWindow { window: WindowType::Daily, limit: 400 }, // actual: 450 (Gmail 500)
        ],
        cooldown_between_posts_secs: 0,
    });

    configs.insert("blog".into(), ChannelConfig {
        name: "blog".into(),
        enabled: true,
        rate_limits: vec![], // No rate limit (self-hosted)
        cooldown_between_posts_secs: 0,
    });

    configs.insert("hackernews".into(), ChannelConfig {
        name: "hackernews".into(),
        enabled: true,
        rate_limits: vec![], // Manual only — monitoring via Algolia
        cooldown_between_posts_secs: 0,
    });

    configs
}

/// Twitter API credentials.
#[derive(Debug, Clone)]
pub struct TwitterCredentials {
    pub api_key: String,
    pub api_secret: String,
    pub access_token: String,
    pub access_token_secret: String,
}

/// All channel credentials loaded from environment.
#[derive(Debug, Clone)]
pub struct ChannelCredentials {
    pub twitter: Option<TwitterCredentials>,
    pub linkedin_token: Option<String>,
    pub linkedin_person_urn: Option<String>,
    pub reddit_client_id: Option<String>,
    pub reddit_client_secret: Option<String>,
    pub reddit_username: Option<String>,
    pub reddit_password: Option<String>,
    pub devto_api_key: Option<String>,
    pub github_token: Option<String>,
    pub smtp_host: Option<String>,
    pub smtp_port: u16,
    pub smtp_user: Option<String>,
    pub smtp_pass: Option<String>,
    pub anthropic_api_key: Option<String>,
}

impl ChannelCredentials {
    /// Load credentials from environment variables.
    pub fn from_env() -> Self {
        let twitter = match (
            std::env::var("TWITTER_API_KEY").ok(),
            std::env::var("TWITTER_API_SECRET").ok(),
            std::env::var("TWITTER_ACCESS_TOKEN").ok(),
            std::env::var("TWITTER_ACCESS_TOKEN_SECRET").ok(),
        ) {
            (Some(k), Some(s), Some(t), Some(ts)) if !k.is_empty() => {
                Some(TwitterCredentials {
                    api_key: k,
                    api_secret: s,
                    access_token: t,
                    access_token_secret: ts,
                })
            }
            _ => None,
        };

        Self {
            twitter,
            linkedin_token: std::env::var("LINKEDIN_ACCESS_TOKEN").ok().filter(|s| !s.is_empty()),
            linkedin_person_urn: std::env::var("LINKEDIN_PERSON_URN").ok().filter(|s| !s.is_empty()),
            reddit_client_id: std::env::var("REDDIT_CLIENT_ID").ok().filter(|s| !s.is_empty()),
            reddit_client_secret: std::env::var("REDDIT_CLIENT_SECRET").ok().filter(|s| !s.is_empty()),
            reddit_username: std::env::var("REDDIT_USERNAME").ok().filter(|s| !s.is_empty()),
            reddit_password: std::env::var("REDDIT_PASSWORD").ok().filter(|s| !s.is_empty()),
            devto_api_key: std::env::var("DEVTO_API_KEY").ok().filter(|s| !s.is_empty()),
            github_token: std::env::var("GITHUB_TOKEN").ok().filter(|s| !s.is_empty()),
            smtp_host: std::env::var("SMTP_HOST").ok().filter(|s| !s.is_empty()),
            smtp_port: std::env::var("SMTP_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(587),
            smtp_user: std::env::var("SMTP_USER").ok().filter(|s| !s.is_empty()),
            smtp_pass: std::env::var("SMTP_PASS").ok().filter(|s| !s.is_empty()),
            anthropic_api_key: std::env::var("ANTHROPIC_API_KEY").ok().filter(|s| !s.is_empty()),
        }
    }

    pub fn has_twitter(&self) -> bool { self.twitter.is_some() }
    pub fn has_linkedin(&self) -> bool { self.linkedin_token.is_some() }
    pub fn has_reddit(&self) -> bool { self.reddit_client_id.is_some() && self.reddit_username.is_some() }
    pub fn has_devto(&self) -> bool { self.devto_api_key.is_some() }
    pub fn has_github(&self) -> bool { self.github_token.is_some() }
    pub fn has_email(&self) -> bool { self.smtp_host.is_some() && self.smtp_user.is_some() }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── WindowType Tests ──

    #[test]
    fn test_window_type_display() {
        assert_eq!(WindowType::Minute.to_string(), "minute");
        assert_eq!(WindowType::Hourly.to_string(), "hourly");
        assert_eq!(WindowType::Daily.to_string(), "daily");
        assert_eq!(WindowType::Monthly.to_string(), "monthly");
    }

    #[test]
    fn test_window_type_duration_secs() {
        assert_eq!(WindowType::Minute.duration_secs(), 60);
        assert_eq!(WindowType::Hourly.duration_secs(), 3600);
        assert_eq!(WindowType::Daily.duration_secs(), 86400);
        assert_eq!(WindowType::Monthly.duration_secs(), 86400 * 30);
    }

    #[test]
    fn test_window_type_ordering() {
        assert!(WindowType::Minute.duration_secs() < WindowType::Hourly.duration_secs());
        assert!(WindowType::Hourly.duration_secs() < WindowType::Daily.duration_secs());
        assert!(WindowType::Daily.duration_secs() < WindowType::Monthly.duration_secs());
    }

    #[test]
    fn test_window_type_serde_roundtrip() {
        let w = WindowType::Daily;
        let json = serde_json::to_string(&w).unwrap();
        assert_eq!(json, "\"daily\"");
        let parsed: WindowType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, w);
    }

    // ── Channel Config Tests ──

    #[test]
    fn test_default_channel_configs_has_all_channels() {
        let configs = default_channel_configs();
        let expected = ["twitter", "linkedin", "reddit", "devto", "github", "email", "blog", "hackernews"];
        for ch in &expected {
            assert!(configs.contains_key(*ch), "Missing channel: {}", ch);
        }
        assert_eq!(configs.len(), expected.len());
    }

    #[test]
    fn test_all_channels_enabled_by_default() {
        let configs = default_channel_configs();
        for (name, config) in &configs {
            assert!(config.enabled, "Channel {} should be enabled", name);
        }
    }

    #[test]
    fn test_twitter_config() {
        let configs = default_channel_configs();
        let twitter = &configs["twitter"];
        assert_eq!(twitter.name, "twitter");
        assert_eq!(twitter.rate_limits.len(), 2);
        assert_eq!(twitter.cooldown_between_posts_secs, 2);
        assert_eq!(twitter.rate_limits[0].limit, 45);
    }

    #[test]
    fn test_blog_has_no_rate_limit() {
        let configs = default_channel_configs();
        let blog = &configs["blog"];
        assert!(blog.rate_limits.is_empty());
        assert_eq!(blog.cooldown_between_posts_secs, 0);
    }

    #[test]
    fn test_hackernews_has_no_rate_limit() {
        let configs = default_channel_configs();
        let hn = &configs["hackernews"];
        assert!(hn.rate_limits.is_empty());
    }

    #[test]
    fn test_channel_config_serde_roundtrip() {
        let config = ChannelConfig {
            name: "test".into(),
            enabled: true,
            rate_limits: vec![RateWindow {
                window: WindowType::Daily,
                limit: 100,
            }],
            cooldown_between_posts_secs: 5,
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: ChannelConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "test");
        assert_eq!(parsed.rate_limits.len(), 1);
        assert_eq!(parsed.rate_limits[0].limit, 100);
    }

    #[test]
    fn test_rate_limits_safety_margin() {
        let configs = default_channel_configs();
        // Twitter daily: actual 50, configured 45 (90%)
        let twitter = &configs["twitter"];
        let daily = twitter.rate_limits.iter().find(|r| r.window == WindowType::Daily).unwrap();
        assert!(daily.limit <= 50 && daily.limit >= 40);
        // LinkedIn daily: actual 25, configured 22
        let linkedin = &configs["linkedin"];
        let daily = linkedin.rate_limits.iter().find(|r| r.window == WindowType::Daily).unwrap();
        assert!(daily.limit <= 25 && daily.limit >= 20);
    }

    // ── ChannelCredentials Tests ──

    fn empty_creds() -> ChannelCredentials {
        ChannelCredentials {
            twitter: None,
            linkedin_token: None,
            linkedin_person_urn: None,
            reddit_client_id: None,
            reddit_client_secret: None,
            reddit_username: None,
            reddit_password: None,
            devto_api_key: None,
            github_token: None,
            smtp_host: None,
            smtp_port: 587,
            smtp_user: None,
            smtp_pass: None,
            anthropic_api_key: None,
        }
    }

    #[test]
    fn test_empty_creds_has_nothing() {
        let c = empty_creds();
        assert!(!c.has_twitter());
        assert!(!c.has_linkedin());
        assert!(!c.has_reddit());
        assert!(!c.has_devto());
        assert!(!c.has_github());
        assert!(!c.has_email());
    }

    #[test]
    fn test_creds_with_all_channels() {
        let c = ChannelCredentials {
            twitter: Some(TwitterCredentials {
                api_key: "k".into(), api_secret: "s".into(),
                access_token: "t".into(), access_token_secret: "ts".into(),
            }),
            linkedin_token: Some("tok".into()),
            linkedin_person_urn: None,
            reddit_client_id: Some("id".into()),
            reddit_client_secret: Some("secret".into()),
            reddit_username: Some("user".into()),
            reddit_password: Some("pass".into()),
            devto_api_key: Some("key".into()),
            github_token: Some("gh-tok".into()),
            smtp_host: Some("smtp.example.com".into()),
            smtp_port: 587,
            smtp_user: Some("user@example.com".into()),
            smtp_pass: Some("pass".into()),
            anthropic_api_key: Some("sk-ant-xxx".into()),
        };
        assert!(c.has_twitter());
        assert!(c.has_linkedin());
        assert!(c.has_reddit());
        assert!(c.has_devto());
        assert!(c.has_github());
        assert!(c.has_email());
    }

    #[test]
    fn test_reddit_requires_client_and_username() {
        let mut c = empty_creds();
        c.reddit_client_id = Some("id".into());
        assert!(!c.has_reddit()); // missing username
        c.reddit_username = Some("user".into());
        assert!(c.has_reddit());
    }

    #[test]
    fn test_email_requires_host_and_user() {
        let mut c = empty_creds();
        c.smtp_host = Some("smtp.example.com".into());
        assert!(!c.has_email()); // missing user
        c.smtp_user = Some("user@example.com".into());
        assert!(c.has_email());
    }

    #[test]
    fn test_github_hourly_limit() {
        let configs = default_channel_configs();
        let gh = &configs["github"];
        let hourly = gh.rate_limits.iter().find(|r| r.window == WindowType::Hourly).unwrap();
        assert_eq!(hourly.limit, 4500); // 90% of 5000
    }
}
