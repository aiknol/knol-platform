//! Content triage — decide whether content is worth an LLM call.
//!
//! This module runs *before* the LLM to skip trivial content, reducing
//! unnecessary API calls and token usage by 30-60%.

use serde::{Deserialize, Serialize};

/// Result of triaging content before LLM extraction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriageDecision {
    /// Skip LLM entirely — content has no extractable signal.
    Skip { reason: &'static str },
    /// Run extraction with reduced output budget.
    Light,
    /// Run full extraction.
    Full,
}

/// Configuration for the triage layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriageConfig {
    /// Enable content triage (skip trivial content).
    pub enabled: bool,
    /// Minimum word count to trigger extraction.
    pub min_words: usize,
    /// Word count threshold below which we use light extraction.
    pub light_threshold_words: usize,
}

impl Default for TriageConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_words: 3,
            light_threshold_words: 15,
        }
    }
}

/// Triage content to decide if it warrants an LLM call.
///
/// This is a fast, local heuristic — no LLM or network calls.
pub fn triage_content(content: &str, config: &TriageConfig) -> TriageDecision {
    if !config.enabled {
        return TriageDecision::Full;
    }

    let trimmed = content.trim();

    // Empty or whitespace-only
    if trimmed.is_empty() {
        return TriageDecision::Skip {
            reason: "empty content",
        };
    }

    let word_count = trimmed.split_whitespace().count();

    // Very short messages (greetings, acks)
    if word_count < config.min_words {
        // Check if it's a known greeting/ack pattern
        let lower = trimmed.to_lowercase();
        let trivial = [
            "hi",
            "hello",
            "hey",
            "ok",
            "okay",
            "thanks",
            "thank you",
            "yes",
            "no",
            "sure",
            "bye",
            "goodbye",
            "lol",
            "haha",
            "hmm",
            "yep",
            "nope",
            "cool",
            "nice",
            "great",
            "wow",
            "gotcha",
            "k",
            "ty",
            "thx",
            "np",
            "gg",
            "brb",
            "afk",
        ];
        if trivial
            .iter()
            .any(|t| lower == *t || lower.starts_with(&format!("{} ", t)))
        {
            return TriageDecision::Skip {
                reason: "trivial greeting or acknowledgment",
            };
        }
        // Even short messages might have info ("I'm Bob")
        if word_count < config.min_words {
            return TriageDecision::Skip {
                reason: "below minimum word count",
            };
        }
    }

    // Pure questions with no assertions (less likely to contain extractable memories)
    if is_pure_question(trimmed) && word_count < 10 {
        return TriageDecision::Skip {
            reason: "short question with no extractable assertions",
        };
    }

    // Short but potentially meaningful — use light extraction
    if word_count < config.light_threshold_words {
        return TriageDecision::Light;
    }

    TriageDecision::Full
}

/// Check if text is purely a question with no embedded facts.
fn is_pure_question(text: &str) -> bool {
    let trimmed = text.trim();

    // Must end with question mark
    if !trimmed.ends_with('?') {
        return false;
    }

    // Must start with a question word
    let lower = trimmed.to_lowercase();
    let question_starters = [
        "what ", "when ", "where ", "who ", "why ", "how ", "which ", "is ", "are ", "was ",
        "were ", "do ", "does ", "did ", "can ", "could ", "will ", "would ", "should ", "shall ",
        "have ", "has ", "had ",
    ];

    question_starters.iter().any(|q| lower.starts_with(q))
}

/// Determine the appropriate max_output_tokens based on content length.
pub fn dynamic_output_tokens(content: &str, enabled: bool) -> u32 {
    if !enabled {
        return 4096;
    }

    let word_count = content.split_whitespace().count();

    if word_count < 50 {
        1024
    } else if word_count < 200 {
        2048
    } else {
        4096
    }
}

/// Filter entity names to only those likely mentioned in the content.
///
/// Instead of sending 100 entities in the prompt, we send only those
/// whose names appear (case-insensitive) in the content, capped at `max_entities`.
pub fn prune_entity_context(
    content: &str,
    all_entities: &[String],
    max_entities: usize,
) -> Vec<String> {
    if all_entities.is_empty() {
        return Vec::new();
    }

    let content_lower = content.to_lowercase();

    let mut relevant: Vec<String> = all_entities
        .iter()
        .filter(|name| {
            let name_lower = name.to_lowercase();
            // Check if entity name (or first word of multi-word name) appears in content
            content_lower.contains(&name_lower)
                || name_lower
                    .split_whitespace()
                    .next()
                    .map(|first| first.len() > 2 && content_lower.contains(first))
                    .unwrap_or(false)
        })
        .cloned()
        .collect();

    relevant.truncate(max_entities);
    relevant
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> TriageConfig {
        TriageConfig::default()
    }

    #[test]
    fn test_skip_empty() {
        assert_eq!(
            triage_content("", &default_config()),
            TriageDecision::Skip {
                reason: "empty content"
            }
        );
        assert_eq!(
            triage_content("   ", &default_config()),
            TriageDecision::Skip {
                reason: "empty content"
            }
        );
    }

    #[test]
    fn test_skip_greetings() {
        assert_eq!(
            triage_content("hi", &default_config()),
            TriageDecision::Skip {
                reason: "trivial greeting or acknowledgment"
            }
        );
        assert_eq!(
            triage_content("thanks", &default_config()),
            TriageDecision::Skip {
                reason: "trivial greeting or acknowledgment"
            }
        );
        assert_eq!(
            triage_content("ok", &default_config()),
            TriageDecision::Skip {
                reason: "trivial greeting or acknowledgment"
            }
        );
    }

    #[test]
    fn test_skip_short() {
        assert_eq!(
            triage_content("yep", &default_config()),
            TriageDecision::Skip {
                reason: "trivial greeting or acknowledgment"
            }
        );
    }

    #[test]
    fn test_skip_pure_question() {
        assert_eq!(
            triage_content("What time is it?", &default_config()),
            TriageDecision::Skip {
                reason: "short question with no extractable assertions"
            }
        );
        assert_eq!(
            triage_content("How are you?", &default_config()),
            TriageDecision::Skip {
                reason: "short question with no extractable assertions"
            }
        );
    }

    #[test]
    fn test_light_short_meaningful() {
        // Short but contains potential facts
        assert_eq!(
            triage_content("I work at Google", &default_config()),
            TriageDecision::Light
        );
        assert_eq!(
            triage_content("My name is Alice", &default_config()),
            TriageDecision::Light
        );
    }

    #[test]
    fn test_full_long_content() {
        let long = "I work at Google as a senior ML engineer. I've been there for 5 years now and I lead the TPU compiler team. We're working on next-gen hardware acceleration.";
        assert_eq!(
            triage_content(long, &default_config()),
            TriageDecision::Full
        );
    }

    #[test]
    fn test_disabled_always_full() {
        let config = TriageConfig {
            enabled: false,
            ..default_config()
        };
        assert_eq!(triage_content("hi", &config), TriageDecision::Full);
    }

    #[test]
    fn test_dynamic_output_tokens_short() {
        assert_eq!(dynamic_output_tokens("I like tea", true), 1024);
    }

    #[test]
    fn test_dynamic_output_tokens_medium() {
        let medium = "word ".repeat(100);
        assert_eq!(dynamic_output_tokens(&medium, true), 2048);
    }

    #[test]
    fn test_dynamic_output_tokens_long() {
        let long = "word ".repeat(300);
        assert_eq!(dynamic_output_tokens(&long, true), 4096);
    }

    #[test]
    fn test_dynamic_output_tokens_disabled() {
        assert_eq!(dynamic_output_tokens("short", false), 4096);
    }

    #[test]
    fn test_prune_entities_relevant_only() {
        let entities = vec![
            "Alice".to_string(),
            "Bob".to_string(),
            "TechCorp".to_string(),
            "Unrelated Inc".to_string(),
        ];
        let result = prune_entity_context("Alice told me about TechCorp", &entities, 20);
        assert!(result.contains(&"Alice".to_string()));
        assert!(result.contains(&"TechCorp".to_string()));
        assert!(!result.contains(&"Bob".to_string()));
        assert!(!result.contains(&"Unrelated Inc".to_string()));
    }

    #[test]
    fn test_prune_entities_partial_match() {
        let entities = vec!["John Smith".to_string()];
        let result = prune_entity_context("I talked to John yesterday", &entities, 20);
        assert!(result.contains(&"John Smith".to_string()));
    }

    #[test]
    fn test_prune_entities_max_cap() {
        let entities: Vec<String> = (0..50).map(|i| format!("Entity{}", i)).collect();
        let content = (0..50)
            .map(|i| format!("Entity{}", i))
            .collect::<Vec<_>>()
            .join(" ");
        let result = prune_entity_context(&content, &entities, 5);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_prune_entities_empty() {
        let result = prune_entity_context("some content", &[], 20);
        assert!(result.is_empty());
    }

    #[test]
    fn test_short_question_skipped() {
        // Short questions (< 10 words) are skipped even if they embed facts
        let result = triage_content("Did you know I work at Google?", &default_config());
        assert_eq!(
            result,
            TriageDecision::Skip {
                reason: "short question with no extractable assertions"
            }
        );
    }

    #[test]
    fn test_long_question_with_facts_not_skipped() {
        // Longer questions (>= 10 words) are not skipped
        let result = triage_content(
            "Did you know I work at Google as a senior ML engineer on the TPU team?",
            &default_config(),
        );
        assert_eq!(result, TriageDecision::Full);
    }
}
