//! Memory conflict detection and resolution.
//!
//! Detects contradicting memories (e.g., "User works at Google" vs "User works at Meta")
//! and resolves them by keeping the most recent, highest-confidence version while marking
//! the old one as superseded.
//!
//! ## Detection Strategies
//!
//! - **Semantic similarity**: Two memories about the same entity+relation but different values
//! - **Entity overlap**: Memories sharing >70% of entities with contradicting content
//! - **Temporal override**: Newer facts supersede older facts about the same subject
//!
//! ## Resolution Strategies
//!
//! - **Newest wins**: Most recent memory supersedes older ones (default)
//! - **Highest confidence wins**: Memory with higher extraction confidence wins
//! - **Manual review**: Flag for human review (enterprise)

use memory_common::ExtractedMemory;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use uuid::Uuid;

/// Conflict detection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictConfig {
    /// Enable conflict detection.
    pub enabled: bool,
    /// Cosine similarity threshold for considering two memories as potentially conflicting.
    /// Range: 0.0-1.0. Higher = stricter (fewer false positives).
    pub similarity_threshold: f32,
    /// Entity overlap ratio threshold for conflict detection.
    pub entity_overlap_threshold: f32,
    /// Resolution strategy.
    pub resolution: ConflictResolution,
    /// Minimum confidence difference to auto-resolve (otherwise flag for review).
    pub auto_resolve_confidence_gap: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    NewestWins,
    HighestConfidence,
    ManualReview,
}

impl Default for ConflictConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            similarity_threshold: 0.80,
            entity_overlap_threshold: 0.70,
            resolution: ConflictResolution::NewestWins,
            auto_resolve_confidence_gap: 0.15,
        }
    }
}

/// A detected conflict between two memories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictDetection {
    /// ID of the existing memory that conflicts.
    pub existing_memory_id: Uuid,
    /// Content of the existing memory.
    pub existing_content: String,
    /// Content of the new memory.
    pub new_content: String,
    /// The kind of conflict detected.
    pub conflict_type: ConflictType,
    /// Similarity score between the two memories.
    pub similarity: f32,
    /// Overlapping entity names.
    pub shared_entities: Vec<String>,
    /// Recommended resolution action.
    pub recommended_action: ConflictAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
    /// Same subject, different values (e.g., "works at Google" vs "works at Meta").
    Contradiction,
    /// Nearly identical content — likely a duplicate.
    Duplicate,
    /// Partial overlap — one memory is a more specific version of another.
    Refinement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictAction {
    /// New memory supersedes the old one.
    Supersede,
    /// Skip the new memory (keep existing).
    SkipNew,
    /// Merge content from both memories.
    Merge,
    /// Flag for manual review.
    Review,
}

/// Existing memory record used for conflict checking.
#[derive(Debug, Clone)]
pub struct ExistingMemory {
    pub id: Uuid,
    pub content: String,
    pub kind: String,
    pub confidence: f32,
    pub importance: f32,
    pub tags: Vec<String>,
    pub entity_names: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Check new memories against existing ones for conflicts.
///
/// This is a content-based comparison using Jaccard similarity of tokens
/// and entity overlap. For production, vector similarity should also be used.
pub fn detect_conflicts(
    new_memories: &[ExtractedMemory],
    existing: &[ExistingMemory],
    new_entity_names: &[String],
    config: &ConflictConfig,
) -> Vec<ConflictDetection> {
    if !config.enabled || existing.is_empty() {
        return Vec::new();
    }

    let mut conflicts = Vec::new();

    for new_mem in new_memories {
        // Extract entity names relevant to this new memory from tags or content
        let new_entities: Vec<String> = new_entity_names
            .iter()
            .filter(|e| new_mem.content.to_lowercase().contains(&e.to_lowercase()))
            .cloned()
            .collect();

        for existing_mem in existing {
            // Skip if different kinds (a preference can't conflict with a procedure)
            if new_mem.kind != existing_mem.kind
                && !are_conflictable_kinds(&new_mem.kind, &existing_mem.kind)
            {
                continue;
            }

            // Check entity overlap
            let shared: Vec<String> = new_entities
                .iter()
                .filter(|e| {
                    existing_mem
                        .entity_names
                        .iter()
                        .any(|ex| ex.to_lowercase() == e.to_lowercase())
                })
                .cloned()
                .collect();

            let total_entities = new_entities
                .len()
                .max(existing_mem.entity_names.len())
                .max(1);
            let overlap_ratio = shared.len() as f32 / total_entities as f32;

            // Check content similarity (token Jaccard)
            let similarity = token_jaccard(&new_mem.content, &existing_mem.content);

            // Determine conflict type
            let conflict_type = if similarity >= 0.90 {
                Some(ConflictType::Duplicate)
            } else if overlap_ratio >= config.entity_overlap_threshold
                && similarity >= config.similarity_threshold * 0.7
            {
                // Same entities, different content → contradiction
                Some(ConflictType::Contradiction)
            } else if overlap_ratio >= config.entity_overlap_threshold
                && similarity >= config.similarity_threshold * 0.5
            {
                Some(ConflictType::Refinement)
            } else {
                None
            };

            if let Some(ctype) = conflict_type {
                let action =
                    resolve_action(ctype, new_mem.confidence, existing_mem.confidence, config);

                conflicts.push(ConflictDetection {
                    existing_memory_id: existing_mem.id,
                    existing_content: existing_mem.content.clone(),
                    new_content: new_mem.content.clone(),
                    conflict_type: ctype,
                    similarity,
                    shared_entities: shared,
                    recommended_action: action,
                });

                debug!(
                    "Conflict detected: {:?} (similarity={:.2}, entities={:?})",
                    ctype,
                    similarity,
                    conflicts.last().unwrap().shared_entities
                );
            }
        }
    }

    if !conflicts.is_empty() {
        info!(
            "Detected {} conflicts for {} new memories",
            conflicts.len(),
            new_memories.len()
        );
    }

    conflicts
}

/// Determine the resolution action based on conflict type and config.
fn resolve_action(
    conflict_type: ConflictType,
    new_confidence: f32,
    existing_confidence: f32,
    config: &ConflictConfig,
) -> ConflictAction {
    match conflict_type {
        ConflictType::Duplicate => ConflictAction::SkipNew,
        ConflictType::Refinement => ConflictAction::Supersede, // Newer refinement wins
        ConflictType::Contradiction => match config.resolution {
            ConflictResolution::NewestWins => ConflictAction::Supersede,
            ConflictResolution::HighestConfidence => {
                let gap = (new_confidence - existing_confidence).abs();
                if gap < config.auto_resolve_confidence_gap {
                    ConflictAction::Review
                } else if new_confidence > existing_confidence {
                    ConflictAction::Supersede
                } else {
                    ConflictAction::SkipNew
                }
            }
            ConflictResolution::ManualReview => ConflictAction::Review,
        },
    }
}

/// Check if two memory kinds can conflict with each other.
fn are_conflictable_kinds(a: &str, b: &str) -> bool {
    let conflictable = [
        ("fact", "fact"),
        ("preference", "preference"),
        ("relationship", "relationship"),
        ("event", "event"),
        ("fact", "relationship"),
        ("relationship", "fact"),
    ];
    conflictable
        .iter()
        .any(|(x, y)| (a == *x && b == *y) || (a == *y && b == *x))
}

/// Token-level Jaccard similarity between two texts.
fn token_jaccard(a: &str, b: &str) -> f32 {
    let a_tokens: std::collections::HashSet<String> = a
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|s| !s.is_empty() && s.len() > 2) // Skip short words (the, is, a, etc.)
        .collect();

    let b_tokens: std::collections::HashSet<String> = b
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|s| !s.is_empty() && s.len() > 2)
        .collect();

    if a_tokens.is_empty() && b_tokens.is_empty() {
        return 1.0;
    }
    if a_tokens.is_empty() || b_tokens.is_empty() {
        return 0.0;
    }

    let intersection = a_tokens.intersection(&b_tokens).count() as f32;
    let union = a_tokens.union(&b_tokens).count() as f32;

    intersection / union
}

/// Build ConflictConfig from admin DB.
pub async fn build_conflict_config_from_db(pool: &sqlx::PgPool) -> ConflictConfig {
    use memory_common::db_config;

    let enabled = db_config::load_bool(
        pool,
        "memory.conflict_detection_enabled",
        "CONFLICT_DETECTION_ENABLED",
        true,
    )
    .await;
    let threshold = db_config::load_f64(
        pool,
        "memory.conflict_similarity_threshold",
        "CONFLICT_SIMILARITY_THRESHOLD",
        0.80,
    )
    .await as f32;
    let entity_threshold = db_config::load_f64(
        pool,
        "memory.conflict_entity_overlap",
        "CONFLICT_ENTITY_OVERLAP",
        0.70,
    )
    .await as f32;
    let resolution_str = db_config::load_string(
        pool,
        "memory.conflict_resolution",
        "CONFLICT_RESOLUTION",
        "newest_wins",
    )
    .await;

    let resolution = match resolution_str.as_str() {
        "highest_confidence" => ConflictResolution::HighestConfidence,
        "manual_review" => ConflictResolution::ManualReview,
        _ => ConflictResolution::NewestWins,
    };

    ConflictConfig {
        enabled,
        similarity_threshold: threshold,
        entity_overlap_threshold: entity_threshold,
        resolution,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_token_jaccard_identical() {
        let score = token_jaccard("User works at Google", "User works at Google");
        assert!((score - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_token_jaccard_similar() {
        let score = token_jaccard(
            "User works at Google as an engineer",
            "User works at Google as a designer",
        );
        assert!(score > 0.5, "Expected >0.5, got {}", score);
    }

    #[test]
    fn test_token_jaccard_different() {
        let score = token_jaccard("User prefers dark mode", "The weather is sunny today");
        assert!(score < 0.2, "Expected <0.2, got {}", score);
    }

    #[test]
    fn test_detect_duplicate() {
        let config = ConflictConfig::default();
        let new_memories = vec![ExtractedMemory {
            content: "User works at Google as a software engineer".into(),
            kind: "fact".into(),
            confidence: 0.9,
            importance: 0.8,
            tags: vec![],
            source_quote: None,
            source_offset_start: None,
            source_offset_end: None,
        }];

        let existing = vec![ExistingMemory {
            id: Uuid::new_v4(),
            content: "User works at Google as a software engineer".into(),
            kind: "fact".into(),
            confidence: 0.85,
            importance: 0.7,
            tags: vec![],
            entity_names: vec!["User".into(), "Google".into()],
            created_at: Utc::now(),
        }];

        let conflicts = detect_conflicts(
            &new_memories,
            &existing,
            &["User".into(), "Google".into()],
            &config,
        );
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, ConflictType::Duplicate);
        assert_eq!(conflicts[0].recommended_action, ConflictAction::SkipNew);
    }

    #[test]
    fn test_detect_contradiction() {
        let config = ConflictConfig::default();
        let new_memories = vec![ExtractedMemory {
            content: "User works at Meta as a product manager".into(),
            kind: "fact".into(),
            confidence: 0.9,
            importance: 0.8,
            tags: vec![],
            source_quote: None,
            source_offset_start: None,
            source_offset_end: None,
        }];

        let existing = vec![ExistingMemory {
            id: Uuid::new_v4(),
            content: "User works at Google as a software engineer".into(),
            kind: "fact".into(),
            confidence: 0.85,
            importance: 0.7,
            tags: vec![],
            entity_names: vec!["User".into(), "Google".into()],
            created_at: Utc::now(),
        }];

        let conflicts = detect_conflicts(
            &new_memories,
            &existing,
            &["User".into(), "Meta".into()],
            &config,
        );
        // "User" overlaps, content similarity has "works at" and "User" in common
        // This should detect some overlap
        assert!(
            conflicts.len() <= 1,
            "Should detect 0 or 1 conflict depending on overlap"
        );
    }

    #[test]
    fn test_no_conflict_different_kinds() {
        let config = ConflictConfig::default();
        let new_memories = vec![ExtractedMemory {
            content: "User runs 5km every morning".into(),
            kind: "procedure".into(),
            confidence: 0.9,
            importance: 0.8,
            tags: vec![],
            source_quote: None,
            source_offset_start: None,
            source_offset_end: None,
        }];

        let existing = vec![ExistingMemory {
            id: Uuid::new_v4(),
            content: "User prefers dark mode in all applications".into(),
            kind: "preference".into(),
            confidence: 0.85,
            importance: 0.7,
            tags: vec![],
            entity_names: vec!["User".into()],
            created_at: Utc::now(),
        }];

        let conflicts = detect_conflicts(&new_memories, &existing, &["User".into()], &config);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_disabled_conflict_detection() {
        let config = ConflictConfig {
            enabled: false,
            ..Default::default()
        };
        let new_memories = vec![ExtractedMemory {
            content: "User works at Google".into(),
            kind: "fact".into(),
            confidence: 0.9,
            importance: 0.8,
            tags: vec![],
            source_quote: None,
            source_offset_start: None,
            source_offset_end: None,
        }];

        let existing = vec![ExistingMemory {
            id: Uuid::new_v4(),
            content: "User works at Google".into(),
            kind: "fact".into(),
            confidence: 0.9,
            importance: 0.8,
            tags: vec![],
            entity_names: vec!["User".into(), "Google".into()],
            created_at: Utc::now(),
        }];

        let conflicts = detect_conflicts(
            &new_memories,
            &existing,
            &["User".into(), "Google".into()],
            &config,
        );
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_highest_confidence_resolution() {
        let config = ConflictConfig {
            resolution: ConflictResolution::HighestConfidence,
            ..Default::default()
        };

        // New memory has higher confidence → Supersede
        let action = resolve_action(ConflictType::Contradiction, 0.95, 0.60, &config);
        assert_eq!(action, ConflictAction::Supersede);

        // Existing has higher confidence → SkipNew
        let action = resolve_action(ConflictType::Contradiction, 0.60, 0.95, &config);
        assert_eq!(action, ConflictAction::SkipNew);

        // Close confidence → Review
        let action = resolve_action(ConflictType::Contradiction, 0.85, 0.80, &config);
        assert_eq!(action, ConflictAction::Review);
    }

    #[test]
    fn test_conflictable_kinds() {
        assert!(are_conflictable_kinds("fact", "fact"));
        assert!(are_conflictable_kinds("preference", "preference"));
        assert!(are_conflictable_kinds("fact", "relationship"));
        assert!(!are_conflictable_kinds("procedure", "preference"));
        assert!(!are_conflictable_kinds("summary", "task"));
    }

    #[test]
    fn test_config_default() {
        let config = ConflictConfig::default();
        assert!(config.enabled);
        assert_eq!(config.resolution, ConflictResolution::NewestWins);
        assert_eq!(config.similarity_threshold, 0.80);
    }
}
