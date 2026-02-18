//! Content generator — picks templates and builds PublishContent.

use rand::Rng;
use tracing::info;

use crate::channels::PublishContent;
use crate::error::MarketingError;

use super::templates;

/// Generate content for a specific channel and template category.
pub fn generate(channel: &str, category: &str) -> Result<PublishContent, MarketingError> {
    let variants = templates::get_templates(category).ok_or_else(|| {
        MarketingError::ContentGeneration(format!("Unknown template category: {}", category))
    })?;

    let idx = rand::thread_rng().gen_range(0..variants.len());
    let template = variants[idx];

    info!(
        "Content: selected {}/{} variant {}/{}",
        channel,
        category,
        idx + 1,
        variants.len()
    );

    let content = match channel {
        "twitter" => PublishContent {
            text: template.to_string(),
            title: None,
            body: None,
            tags: default_tags(),
            subreddit: None,
            subject: None,
        },
        "linkedin" => PublishContent {
            text: template.to_string(),
            title: None,
            body: None,
            tags: default_tags(),
            subreddit: None,
            subject: None,
        },
        "reddit" => {
            // First line is the title, rest is body
            let (title, body) = split_title_body(template);
            let subreddit = if category.contains("rust") {
                "rust"
            } else {
                "MachineLearning"
            };
            PublishContent {
                text: title.to_string(),
                title: Some(title.to_string()),
                body: Some(body.to_string()),
                tags: default_tags(),
                subreddit: Some(subreddit.to_string()),
                subject: None,
            }
        }
        "devto" => {
            let (title, body) = split_title_body(template);
            PublishContent {
                text: title.to_string(),
                title: Some(title.to_string()),
                body: Some(body.to_string()),
                tags: vec![
                    "rust".into(),
                    "ai".into(),
                    "machinelearning".into(),
                    "opensource".into(),
                ],
                subreddit: None,
                subject: None,
            }
        }
        "github" => PublishContent {
            text: template.to_string(),
            title: None,
            body: Some(template.to_string()),
            tags: vec![
                "memory".into(),
                "ai".into(),
                "llm".into(),
                "rust".into(),
                "machine-learning".into(),
            ],
            subreddit: None,
            subject: None,
        },
        "email" => PublishContent {
            text: "Knol Newsletter".to_string(),
            title: None,
            body: Some(template.to_string()),
            tags: vec![],
            subreddit: None,
            subject: Some(if category.contains("welcome") {
                "Welcome to Knol".to_string()
            } else {
                format!("This Week at Knol — {}", chrono::Utc::now().format("%b %d"))
            }),
        },
        "blog" => {
            let (title, body) = split_title_body(template);
            PublishContent {
                text: title.to_string(),
                title: Some(title.to_string()),
                body: Some(body.to_string()),
                tags: default_tags(),
                subreddit: None,
                subject: None,
            }
        }
        "hackernews" => PublishContent {
            text: template.to_string(),
            title: Some(template.to_string()),
            body: None,
            tags: vec![],
            subreddit: None,
            subject: None,
        },
        _ => PublishContent {
            text: template.to_string(),
            title: None,
            body: None,
            tags: default_tags(),
            subreddit: None,
            subject: None,
        },
    };

    Ok(content)
}

/// Preview content without publishing — useful for admin preview endpoint.
pub fn preview(channel: &str, category: &str) -> Result<PublishContent, MarketingError> {
    generate(channel, category)
}

fn split_title_body(text: &str) -> (&str, &str) {
    // If starts with "# ", use the first line as title
    if let Some(stripped) = text.strip_prefix("# ") {
        if let Some(nl) = stripped.find('\n') {
            let title = &stripped[..nl];
            let body = stripped[nl..].trim_start();
            return (title, body);
        }
        return (stripped, "");
    }
    // Otherwise, first line is title
    if let Some(nl) = text.find('\n') {
        let title = &text[..nl];
        let body = text[nl..].trim_start();
        (title, body)
    } else {
        (text, "")
    }
}

fn default_tags() -> Vec<String> {
    vec![
        "rust".into(),
        "ai".into(),
        "memory".into(),
        "llm".into(),
        "opensource".into(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_tweet() {
        let content = generate("twitter", "tweet_launch").unwrap();
        assert!(!content.text.is_empty());
        assert!(content.title.is_none());
    }

    #[test]
    fn generate_reddit() {
        let content = generate("reddit", "reddit_rust").unwrap();
        assert!(content.title.is_some());
        assert!(content.body.is_some());
        assert_eq!(content.subreddit.as_deref(), Some("rust"));
    }

    #[test]
    fn generate_blog() {
        let content = generate("blog", "blog_launch").unwrap();
        assert!(content.title.is_some());
        assert!(content.body.is_some());
    }

    #[test]
    fn unknown_category_errors() {
        assert!(generate("twitter", "nonexistent").is_err());
    }
}
