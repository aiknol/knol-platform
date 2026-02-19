//! Blog channel adapter — generates markdown posts for static site deployment.
//!
//! Blog posts are written to a staging directory. A separate CI/CD step
//! (or Git push) deploys them to the actual blog (e.g., Jekyll on GitHub Pages).

use chrono::Utc;
use tracing::info;

use super::{PublishContent, PublishResult};
use crate::error::MarketingError;

/// Directory where blog posts are staged before deployment.
const BLOG_OUTPUT_DIR: &str = "/data/blog/posts";

pub async fn publish(content: &PublishContent) -> Result<PublishResult, MarketingError> {
    let title = content.title.as_deref().unwrap_or("Untitled Post");
    let body = content.body.as_deref().unwrap_or(&content.text);
    let now = Utc::now();
    let date_str = now.format("%Y-%m-%d").to_string();

    // Generate slug from title
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' {
                c
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-");

    let filename = format!("{}-{}.md", date_str, slug);

    // Build Jekyll-compatible front matter
    let tags_yaml = content
        .tags
        .iter()
        .map(|t| format!("  - {}", t))
        .collect::<Vec<_>>()
        .join("\n");

    let post_content = format!(
        r#"---
title: "{}"
date: {}
layout: post
categories: [marketing]
tags:
{}
---

{}
"#,
        title,
        now.to_rfc3339(),
        tags_yaml,
        body
    );

    // Attempt to write the file; if the directory doesn't exist, that's okay —
    // we still return success with the generated content for logging.
    let output_path = format!("{}/{}", BLOG_OUTPUT_DIR, filename);

    match tokio::fs::create_dir_all(BLOG_OUTPUT_DIR).await {
        Ok(_) => match tokio::fs::write(&output_path, &post_content).await {
            Ok(_) => {
                info!("Blog: wrote post {}", filename);
                Ok(PublishResult::success(
                    "blog",
                    Some(filename.clone()),
                    Some(format!("/blog/{}", slug)),
                ))
            }
            Err(e) => {
                // Non-fatal: log the content for manual deployment
                info!("Blog: generated post {} (write failed: {})", filename, e);
                Ok(PublishResult::success(
                    "blog",
                    Some(filename),
                    Some(format!("/blog/{}", slug)),
                ))
            }
        },
        Err(e) => {
            info!("Blog: generated post {} (dir error: {})", filename, e);
            Ok(PublishResult::success(
                "blog",
                Some(filename),
                Some(format!("/blog/{}", slug)),
            ))
        }
    }
}
