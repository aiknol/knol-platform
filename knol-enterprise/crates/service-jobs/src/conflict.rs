//! Memory Conflict Detection and Resolution Pipeline
//!
//! This module detects and resolves conflicts between memories using:
//! 1. Content similarity analysis (word overlap)
//! 2. Entity overlap detection (shared entities)
//! 3. Temporal analysis (age differences)
//! 4. Conflict type classification
//! 5. Resolution strategies based on conflict type

use chrono::{Duration, Utc};
use sqlx::PgPool;
use std::collections::HashSet;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Configuration for conflict detection behavior
#[derive(Debug, Clone)]
pub struct ConflictDetectionConfig {
    /// Minimum days difference to consider memory "much newer" (default: 7)
    pub supersede_days_threshold: i64,
    /// Word overlap percentage threshold for duplicates (default: 0.80)
    pub duplicate_similarity_threshold: f32,
    /// Word overlap percentage threshold for contradictions (default: 0.50)
    pub contradiction_similarity_threshold: f32,
    /// Maximum conflicts to process per run (default: 100)
    pub max_conflicts_per_run: usize,
}

impl Default for ConflictDetectionConfig {
    fn default() -> Self {
        Self {
            supersede_days_threshold: 7,
            duplicate_similarity_threshold: 0.80,
            contradiction_similarity_threshold: 0.50,
            max_conflicts_per_run: 100,
        }
    }
}

/// Types of conflicts that can be detected between memories
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum ConflictType {
    /// Two memories express opposite or contradictory facts
    #[sqlx(rename = "contradiction")]
    Contradiction,
    /// One memory supersedes another due to recency
    #[sqlx(rename = "superseded")]
    Superseded,
    /// Two memories are essentially duplicates
    #[sqlx(rename = "duplicate")]
    Duplicate,
    /// Conflict type cannot be determined
    #[sqlx(rename = "ambiguous")]
    Ambiguous,
}

impl ConflictType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Contradiction => "contradiction",
            Self::Superseded => "superseded",
            Self::Duplicate => "duplicate",
            Self::Ambiguous => "ambiguous",
        }
    }
}

/// Resolution strategies for different conflict types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Keep the newer memory, archive the older
    KeepNewer,
    /// Flag both for human review
    Flag,
    /// No action taken
    NoAction,
}

/// Represents a detected conflict between two memories
#[derive(Debug, Clone)]
struct MemoryConflict {
    /// First memory ID
    memory_id_1: Uuid,
    /// Second memory ID
    memory_id_2: Uuid,
    /// Content of first memory
    content_1: String,
    /// Content of second memory
    content_2: String,
    /// Creation time of first memory
    created_at_1: chrono::DateTime<Utc>,
    /// Creation time of second memory
    created_at_2: chrono::DateTime<Utc>,
    /// Number of shared entities
    shared_entity_count: i64,
    /// Tenant ID
    tenant_id: Uuid,
    /// Detected conflict type
    conflict_type: ConflictType,
    /// Word overlap ratio
    word_overlap: f32,
}

/// Main conflict detection and resolution engine
pub struct ConflictDetector {
    db_pool: PgPool,
    config: ConflictDetectionConfig,
}

impl ConflictDetector {
    /// Create a new conflict detector
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool,
            config: ConflictDetectionConfig::default(),
        }
    }

    /// Main entry point: run full conflict detection and resolution pipeline
    pub async fn run_conflict_detection(&self) -> anyhow::Result<u64> {
        info!("Starting memory conflict detection pipeline");

        // Find potential conflicts
        let potential_conflicts = self.find_potential_conflicts().await.map_err(|e| {
            error!("Failed to find potential conflicts: {}", e);
            e
        })?;

        if potential_conflicts.is_empty() {
            info!("No potential conflicts found");
            return Ok(0);
        }

        info!(
            "Found {} potential conflict pairs to analyze",
            potential_conflicts.len()
        );

        let mut total_resolved = 0u64;

        // Process each conflict pair
        for conflict in potential_conflicts {
            match self.process_conflict(conflict).await {
                Ok(()) => total_resolved += 1,
                Err(e) => {
                    warn!("Failed to process conflict: {}", e);
                }
            }
        }

        info!(
            "Memory conflict detection complete: {} conflicts resolved",
            total_resolved
        );
        Ok(total_resolved)
    }

    /// Find potential conflict pairs within same tenant/scope
    async fn find_potential_conflicts(&self) -> anyhow::Result<Vec<MemoryConflict>> {
        let max_limit = self.config.max_conflicts_per_run as i64;

        #[derive(sqlx::FromRow)]
        struct ConflictRow {
            memory_id_1: Uuid,
            memory_id_2: Uuid,
            content_1: String,
            content_2: String,
            created_at_1: chrono::DateTime<Utc>,
            created_at_2: chrono::DateTime<Utc>,
            shared_entity_count: Option<i64>,
            tenant_id: Uuid,
        }

        let rows = sqlx::query_as::<_, ConflictRow>(
            r#"
            SELECT
                m1.id as memory_id_1,
                m2.id as memory_id_2,
                m1.content as content_1,
                m2.content as content_2,
                m1.created_at as created_at_1,
                m2.created_at as created_at_2,
                COUNT(DISTINCT e.id) as shared_entity_count,
                m1.tenant_id
            FROM memories m1
            INNER JOIN memories m2 ON m1.tenant_id = m2.tenant_id
            LEFT JOIN memory_entities me1 ON m1.id = me1.memory_id
            LEFT JOIN memory_entities me2 ON m2.id = me2.memory_id
            LEFT JOIN entities e ON me1.entity_id = e.id AND me2.entity_id = e.id
            WHERE m1.status = 'active'
              AND m2.status = 'active'
              AND m1.id < m2.id
              AND me1.entity_id IS NOT NULL
              AND me2.entity_id IS NOT NULL
              AND NOT EXISTS (
                SELECT 1 FROM memory_conflicts
                WHERE (memory_id_1 = m1.id AND memory_id_2 = m2.id)
                   OR (memory_id_1 = m2.id AND memory_id_2 = m1.id)
              )
            GROUP BY m1.id, m2.id, m1.content, m2.content, m1.created_at, m2.created_at, m1.tenant_id
            HAVING COUNT(DISTINCT e.id) > 0
            LIMIT $1
            "#,
        )
        .bind(max_limit)
        .fetch_all(&self.db_pool)
        .await?;

        let conflicts = rows
            .into_iter()
            .map(|row| {
                let word_overlap = Self::calculate_word_overlap(&row.content_1, &row.content_2);

                MemoryConflict {
                    memory_id_1: row.memory_id_1,
                    memory_id_2: row.memory_id_2,
                    content_1: row.content_1,
                    content_2: row.content_2,
                    created_at_1: row.created_at_1,
                    created_at_2: row.created_at_2,
                    shared_entity_count: row.shared_entity_count.unwrap_or(0),
                    tenant_id: row.tenant_id,
                    conflict_type: ConflictType::Ambiguous, // Will be classified below
                    word_overlap,
                }
            })
            .collect();

        Ok(conflicts)
    }

    /// Process a single conflict: classify, resolve, and record
    async fn process_conflict(&self, mut conflict: MemoryConflict) -> anyhow::Result<()> {
        // Classify conflict type
        conflict.conflict_type = self.classify_conflict(&conflict);

        debug!(
            "Detected {} conflict between memories {} and {} (overlap: {:.2}%)",
            conflict.conflict_type.as_str(),
            conflict.memory_id_1,
            conflict.memory_id_2,
            conflict.word_overlap * 100.0
        );

        // Determine resolution strategy
        let resolution = Self::determine_resolution(conflict.conflict_type);

        // Apply resolution
        self.resolve_conflict(&conflict, resolution).await?;

        // Store conflict record for audit
        self.store_conflict_record(&conflict, resolution).await?;

        Ok(())
    }

    /// Classify conflict type based on memory characteristics
    fn classify_conflict(&self, conflict: &MemoryConflict) -> ConflictType {
        // If one is much newer: Superseded
        let age_diff = (conflict.created_at_1 - conflict.created_at_2).abs();
        if age_diff > Duration::days(self.config.supersede_days_threshold) {
            return ConflictType::Superseded;
        }

        // If very high word overlap: Duplicate
        if conflict.word_overlap > self.config.duplicate_similarity_threshold {
            return ConflictType::Duplicate;
        }

        // If they mention same entity with different facts: Contradiction
        if conflict.shared_entity_count > 0
            && conflict.word_overlap > self.config.contradiction_similarity_threshold
        {
            // Check if content expresses different sentiments/facts about same entities
            if Self::expresses_contradiction(&conflict.content_1, &conflict.content_2) {
                return ConflictType::Contradiction;
            }
        }

        // Otherwise: Ambiguous
        ConflictType::Ambiguous
    }

    /// Check if two memory contents express contradictory information
    fn expresses_contradiction(content1: &str, content2: &str) -> bool {
        // Simple heuristic: look for negation markers in opposite directions
        let negation_words = ["not", "no", "never", "cannot", "can't", "won't", "isn't"];

        let content1_lower = content1.to_lowercase();
        let content2_lower = content2.to_lowercase();

        let has_negation_1 = negation_words.iter().any(|w| content1_lower.contains(w));
        let has_negation_2 = negation_words.iter().any(|w| content2_lower.contains(w));

        // If one has negation and the other doesn't, they might contradict
        let overlap = Self::calculate_word_overlap(&content1_lower, &content2_lower);
        has_negation_1 != has_negation_2 && overlap > 0.3
    }

    /// Determine resolution strategy based on conflict type
    fn determine_resolution(conflict_type: ConflictType) -> ConflictResolution {
        match conflict_type {
            ConflictType::Duplicate => ConflictResolution::KeepNewer,
            ConflictType::Superseded => ConflictResolution::KeepNewer,
            ConflictType::Contradiction => ConflictResolution::Flag,
            ConflictType::Ambiguous => ConflictResolution::NoAction,
        }
    }

    /// Apply the resolution strategy to the conflict
    async fn resolve_conflict(
        &self,
        conflict: &MemoryConflict,
        resolution: ConflictResolution,
    ) -> anyhow::Result<()> {
        match resolution {
            ConflictResolution::KeepNewer => {
                self.resolve_keep_newer(conflict).await?;
            }
            ConflictResolution::Flag => {
                self.resolve_flag_for_review(conflict).await?;
            }
            ConflictResolution::NoAction => {
                debug!(
                    "No action taken for ambiguous conflict between {} and {}",
                    conflict.memory_id_1, conflict.memory_id_2
                );
            }
        }

        Ok(())
    }

    /// Archive the older memory, keep the newer one
    async fn resolve_keep_newer(&self, conflict: &MemoryConflict) -> anyhow::Result<()> {
        let (older_id, newer_id) = if conflict.created_at_1 < conflict.created_at_2 {
            (conflict.memory_id_1, conflict.memory_id_2)
        } else {
            (conflict.memory_id_2, conflict.memory_id_1)
        };

        let now = Utc::now();
        sqlx::query(
            r#"
            UPDATE memories
            SET status = 'archived', valid_to = $1, updated_at = $1
            WHERE id = $2
            "#,
        )
        .bind(now)
        .bind(older_id)
        .execute(&self.db_pool)
        .await?;

        debug!(
            "Archived older memory {} in favor of {} ({})",
            older_id,
            newer_id,
            conflict.conflict_type.as_str()
        );

        Ok(())
    }

    /// Flag both memories for human review
    async fn resolve_flag_for_review(&self, conflict: &MemoryConflict) -> anyhow::Result<()> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE memories
            SET metadata = jsonb_set(
                COALESCE(metadata, '{}'::jsonb),
                '{needs_review}',
                'true'::jsonb
            ),
            updated_at = $1
            WHERE id = $2 OR id = $3
            "#,
        )
        .bind(now)
        .bind(conflict.memory_id_1)
        .bind(conflict.memory_id_2)
        .execute(&self.db_pool)
        .await?;

        debug!(
            "Flagged memories {} and {} for human review ({} conflict)",
            conflict.memory_id_1,
            conflict.memory_id_2,
            conflict.conflict_type.as_str()
        );

        Ok(())
    }

    /// Store conflict record in audit table
    async fn store_conflict_record(
        &self,
        conflict: &MemoryConflict,
        resolution: ConflictResolution,
    ) -> anyhow::Result<()> {
        let now = Utc::now();

        // Try to insert, but don't fail if the table doesn't exist
        let result = sqlx::query(
            r#"
            INSERT INTO memory_conflicts (
                tenant_id,
                memory_id_1,
                memory_id_2,
                conflict_type,
                shared_entity_count,
                word_overlap,
                resolution_strategy,
                created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(conflict.tenant_id)
        .bind(conflict.memory_id_1)
        .bind(conflict.memory_id_2)
        .bind(conflict.conflict_type.as_str())
        .bind(conflict.shared_entity_count)
        .bind(conflict.word_overlap as f64)
        .bind(format!("{:?}", resolution))
        .bind(now)
        .execute(&self.db_pool)
        .await;

        if let Err(e) = result {
            // Log but don't fail if conflicts table doesn't exist
            debug!("Could not record conflict audit: {}", e);
        }

        Ok(())
    }

    /// Calculate word overlap ratio between two content strings
    fn calculate_word_overlap(content1: &str, content2: &str) -> f32 {
        let lower1 = content1.to_lowercase();
        let words1: HashSet<&str> = lower1
            .split_whitespace()
            .filter(|w| w.len() > 2) // Ignore very short words
            .collect();

        let lower2 = content2.to_lowercase();
        let words2: HashSet<&str> = lower2.split_whitespace().filter(|w| w.len() > 2).collect();

        if words1.is_empty() || words2.is_empty() {
            return 0.0;
        }

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            return 0.0;
        }

        intersection as f32 / union as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_word_overlap_identical() {
        let text = "I have a meeting with John tomorrow about the project";
        let overlap = ConflictDetector::calculate_word_overlap(text, text);
        assert!((overlap - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_word_overlap_high_similarity() {
        let text1 = "lunch john discussed project proposal deadline review";
        let text2 = "lunch john discussed project proposal timeline review";
        let overlap = ConflictDetector::calculate_word_overlap(text1, text2);
        assert!(overlap > 0.6, "Expected >0.6 but got {}", overlap);
    }

    #[test]
    fn test_calculate_word_overlap_low_similarity() {
        let text1 = "I like coffee in the morning";
        let text2 = "The weather was rainy yesterday afternoon";
        let overlap = ConflictDetector::calculate_word_overlap(text1, text2);
        assert!(overlap < 0.2);
    }

    #[test]
    fn test_calculate_word_overlap_no_overlap() {
        let text1 = "apple banana cherry";
        let text2 = "dog elephant fish";
        let overlap = ConflictDetector::calculate_word_overlap(text1, text2);
        assert_eq!(overlap, 0.0);
    }

    #[test]
    fn test_calculate_word_overlap_ignores_short_words() {
        let text1 = "I a the meeting";
        let text2 = "I a the conference";
        let overlap = ConflictDetector::calculate_word_overlap(text1, text2);
        // Should only count 'meeting' vs 'conference', which are different
        assert!(overlap < 0.5);
    }

    #[test]
    fn test_conflict_type_superseded_age_check() {
        // Test the time difference calculation logic
        let now = Utc::now();
        let age_diff = (now - Duration::days(10) - now).abs();

        let config = ConflictDetectionConfig::default();
        let exceeds_threshold = age_diff > Duration::days(config.supersede_days_threshold);

        assert!(exceeds_threshold, "10 days should exceed 7 day threshold");
    }

    #[test]
    fn test_conflict_type_duplicate_similarity_check() {
        // Test duplicate detection by high word overlap
        let overlap = 0.95;
        let config = ConflictDetectionConfig::default();

        let is_duplicate = overlap > config.duplicate_similarity_threshold;
        assert!(is_duplicate, "0.95 overlap should be > 0.80 threshold");
    }

    #[test]
    fn test_determine_resolution_duplicate() {
        let resolution = ConflictDetector::determine_resolution(ConflictType::Duplicate);
        assert_eq!(resolution, ConflictResolution::KeepNewer);
    }

    #[test]
    fn test_determine_resolution_superseded() {
        let resolution = ConflictDetector::determine_resolution(ConflictType::Superseded);
        assert_eq!(resolution, ConflictResolution::KeepNewer);
    }

    #[test]
    fn test_determine_resolution_contradiction() {
        let resolution = ConflictDetector::determine_resolution(ConflictType::Contradiction);
        assert_eq!(resolution, ConflictResolution::Flag);
    }

    #[test]
    fn test_determine_resolution_ambiguous() {
        let resolution = ConflictDetector::determine_resolution(ConflictType::Ambiguous);
        assert_eq!(resolution, ConflictResolution::NoAction);
    }

    #[test]
    fn test_expresses_contradiction_opposite_negation() {
        let text1 = "John is not available for the meeting";
        let text2 = "John is available for the meeting";
        let result = ConflictDetector::expresses_contradiction(text1, text2);
        assert!(result);
    }

    #[test]
    fn test_no_contradiction_without_negation_difference() {
        let text1 = "John is available for the meeting";
        let text2 = "John is free for the meeting";
        let result = ConflictDetector::expresses_contradiction(text1, text2);
        // Should be false because both are positive
        assert!(!result);
    }
}
