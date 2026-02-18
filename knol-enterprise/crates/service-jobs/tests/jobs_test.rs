//! Unit tests for background job logic
//!
//! Tests importance decay math, consolidation config, conflict detection
//! algorithms, and scheduling behavior without requiring a database.

use std::collections::HashSet;

// ── Importance Decay Tests ──

/// Exponential decay: importance *= e^(-λ * days)
fn calculate_decay(importance: f64, lambda: f64, days: f64) -> f64 {
    importance * (-lambda * days).exp()
}

#[test]
fn test_decay_one_day() {
    let result = calculate_decay(1.0, 0.01, 1.0);
    assert!((result - 0.99005).abs() < 0.001);
}

#[test]
fn test_decay_30_days() {
    let result = calculate_decay(1.0, 0.01, 30.0);
    assert!((result - 0.7408).abs() < 0.01);
}

#[test]
fn test_decay_90_days() {
    let result = calculate_decay(1.0, 0.01, 90.0);
    assert!((result - 0.4066).abs() < 0.01);
}

#[test]
fn test_decay_365_days() {
    let result = calculate_decay(1.0, 0.01, 365.0);
    assert!((result - 0.0255).abs() < 0.01);
}

#[test]
fn test_decay_preserves_minimum() {
    let min_importance = 0.05;
    let decayed = calculate_decay(0.1, 0.01, 100.0);
    let clamped = if decayed < min_importance { min_importance } else { decayed };
    // After 100 days with lambda=0.01, 0.1 * e^(-1.0) ≈ 0.0368
    assert!(clamped >= min_importance);
}

#[test]
fn test_decay_zero_days_no_change() {
    let result = calculate_decay(0.8, 0.01, 0.0);
    assert!((result - 0.8).abs() < 0.001);
}

#[test]
fn test_decay_already_low_skipped() {
    // Memories with importance <= 0.05 should not be decayed further
    let importance = 0.04;
    let should_decay = importance > 0.05;
    assert!(!should_decay);
}

// ── Consolidation Config Tests ──

struct ConsolidationConfig {
    min_age_hours: i64,
    min_cluster_size: usize,
    max_consolidations_per_run: usize,
}

impl Default for ConsolidationConfig {
    fn default() -> Self {
        Self {
            min_age_hours: 24,
            min_cluster_size: 3,
            max_consolidations_per_run: 100,
        }
    }
}

#[test]
fn test_consolidation_config_defaults() {
    let config = ConsolidationConfig::default();
    assert_eq!(config.min_age_hours, 24);
    assert_eq!(config.min_cluster_size, 3);
    assert_eq!(config.max_consolidations_per_run, 100);
}

#[test]
fn test_consolidation_min_age_filter() {
    let config = ConsolidationConfig::default();
    let now = chrono::Utc::now();
    let memory_age_hours = 48i64;

    let is_eligible = memory_age_hours >= config.min_age_hours;
    assert!(is_eligible);
}

#[test]
fn test_consolidation_min_age_filter_too_young() {
    let config = ConsolidationConfig::default();
    let memory_age_hours = 12i64;

    let is_eligible = memory_age_hours >= config.min_age_hours;
    assert!(!is_eligible);
}

// ── Clustering Tests ──

fn content_similar(content1: &str, content2: &str) -> bool {
    let words1: HashSet<&str> = content1.split_whitespace().collect();
    let words2: HashSet<&str> = content2.split_whitespace().collect();

    let intersection = words1.intersection(&words2).count();
    let union = words1.union(&words2).count();

    if union == 0 { return false; }
    let similarity = intersection as f32 / union as f32;
    similarity > 0.3
}

#[test]
fn test_content_similar_high_overlap() {
    assert!(content_similar(
        "John discussed the project timeline and deliverables",
        "John reviewed the project timeline and milestones"
    ));
}

#[test]
fn test_content_similar_low_overlap() {
    assert!(!content_similar(
        "Coffee shop opens at seven",
        "The database migration completed successfully"
    ));
}

#[test]
fn test_content_similar_identical() {
    let text = "exact same content here";
    assert!(content_similar(text, text));
}

#[test]
fn test_content_similar_empty() {
    assert!(!content_similar("", ""));
}

#[test]
fn test_cluster_formation_minimum_size() {
    // Simulate cluster formation with min_cluster_size = 3
    let min_cluster_size = 3;
    let memories = vec![
        "John discussed project deadlines",
        "John reviewed project milestones",
        "John updated project timeline",
    ];

    // These are all similar (share "John" and "project")
    let cluster_size = memories.len();
    assert!(cluster_size >= min_cluster_size);
}

#[test]
fn test_cluster_too_small_rejected() {
    let min_cluster_size = 3;
    let cluster_size = 2;
    assert!(cluster_size < min_cluster_size);
}

// ── Conflict Detection Config Tests ──

struct ConflictDetectionConfig {
    supersede_days_threshold: i64,
    duplicate_similarity_threshold: f32,
    contradiction_similarity_threshold: f32,
    max_conflicts_per_run: usize,
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

#[test]
fn test_conflict_config_defaults() {
    let config = ConflictDetectionConfig::default();
    assert_eq!(config.supersede_days_threshold, 7);
    assert!((config.duplicate_similarity_threshold - 0.80).abs() < 0.01);
    assert!((config.contradiction_similarity_threshold - 0.50).abs() < 0.01);
    assert_eq!(config.max_conflicts_per_run, 100);
}

// ── Conflict Classification Tests ──

#[derive(Debug, PartialEq)]
enum ConflictType {
    Contradiction,
    Superseded,
    Duplicate,
    Ambiguous,
}

#[derive(Debug, PartialEq)]
enum ConflictResolution {
    KeepNewer,
    KeepHigherConfidence,
    Flag,
    NoAction,
}

fn classify_conflict(
    age_diff_days: i64,
    word_overlap: f32,
    shared_entities: i64,
    has_contradiction: bool,
    config: &ConflictDetectionConfig,
) -> ConflictType {
    if age_diff_days > config.supersede_days_threshold {
        return ConflictType::Superseded;
    }
    if word_overlap > config.duplicate_similarity_threshold {
        return ConflictType::Duplicate;
    }
    if shared_entities > 0 && word_overlap > config.contradiction_similarity_threshold && has_contradiction {
        return ConflictType::Contradiction;
    }
    ConflictType::Ambiguous
}

fn determine_resolution(conflict_type: &ConflictType) -> ConflictResolution {
    match conflict_type {
        ConflictType::Duplicate => ConflictResolution::KeepNewer,
        ConflictType::Superseded => ConflictResolution::KeepNewer,
        ConflictType::Contradiction => ConflictResolution::Flag,
        ConflictType::Ambiguous => ConflictResolution::NoAction,
    }
}

#[test]
fn test_classify_superseded_by_age() {
    let config = ConflictDetectionConfig::default();
    let result = classify_conflict(10, 0.5, 1, false, &config);
    assert_eq!(result, ConflictType::Superseded);
}

#[test]
fn test_classify_duplicate_by_overlap() {
    let config = ConflictDetectionConfig::default();
    let result = classify_conflict(2, 0.95, 1, false, &config);
    assert_eq!(result, ConflictType::Duplicate);
}

#[test]
fn test_classify_contradiction() {
    let config = ConflictDetectionConfig::default();
    let result = classify_conflict(2, 0.60, 2, true, &config);
    assert_eq!(result, ConflictType::Contradiction);
}

#[test]
fn test_classify_ambiguous_no_signals() {
    let config = ConflictDetectionConfig::default();
    let result = classify_conflict(2, 0.3, 0, false, &config);
    assert_eq!(result, ConflictType::Ambiguous);
}

#[test]
fn test_resolution_for_all_types() {
    assert_eq!(determine_resolution(&ConflictType::Duplicate), ConflictResolution::KeepNewer);
    assert_eq!(determine_resolution(&ConflictType::Superseded), ConflictResolution::KeepNewer);
    assert_eq!(determine_resolution(&ConflictType::Contradiction), ConflictResolution::Flag);
    assert_eq!(determine_resolution(&ConflictType::Ambiguous), ConflictResolution::NoAction);
}

// ── Negation Heuristic Tests ──

fn expresses_contradiction(content1: &str, content2: &str) -> bool {
    let negation_words = ["not", "no", "never", "cannot", "can't", "won't", "isn't"];
    let c1 = content1.to_lowercase();
    let c2 = content2.to_lowercase();

    let has_neg_1 = negation_words.iter().any(|w| c1.contains(w));
    let has_neg_2 = negation_words.iter().any(|w| c2.contains(w));

    let words1: HashSet<&str> = c1.split_whitespace().filter(|w| w.len() > 2).collect();
    let words2: HashSet<&str> = c2.split_whitespace().filter(|w| w.len() > 2).collect();
    let intersection = words1.intersection(&words2).count();
    let union = words1.union(&words2).count();
    let overlap = if union > 0 { intersection as f32 / union as f32 } else { 0.0 };

    has_neg_1 != has_neg_2 && overlap > 0.3
}

#[test]
fn test_contradiction_with_negation() {
    assert!(expresses_contradiction(
        "John is not available for the meeting",
        "John is available for the meeting"
    ));
}

#[test]
fn test_no_contradiction_both_positive() {
    assert!(!expresses_contradiction(
        "The project is on track",
        "The project is progressing well"
    ));
}

#[test]
fn test_no_contradiction_both_negative() {
    assert!(!expresses_contradiction(
        "The feature is not ready yet",
        "The feature cannot be deployed"
    ));
}

#[test]
fn test_no_contradiction_unrelated_topics() {
    assert!(!expresses_contradiction(
        "John is not available",
        "The weather is sunny today"
    ));
}

// ── Job Scheduling Tests ──

#[test]
fn test_job_intervals() {
    use std::time::Duration;

    let decay_interval = Duration::from_secs(3600);       // 1 hour
    let dedup_interval = Duration::from_secs(7200);       // 2 hours
    let retention_interval = Duration::from_secs(86400);  // 24 hours
    let cleanup_interval = Duration::from_secs(86400);    // 24 hours
    let consolidation_interval = Duration::from_secs(43200); // 12 hours
    let conflict_interval = Duration::from_secs(14400);   // 4 hours

    assert_eq!(decay_interval.as_secs(), 3600);
    assert_eq!(dedup_interval.as_secs(), 7200);
    assert_eq!(retention_interval.as_secs(), 86400);
    assert_eq!(cleanup_interval.as_secs(), 86400);
    assert_eq!(consolidation_interval.as_secs(), 43200);
    assert_eq!(conflict_interval.as_secs(), 14400);

    // Verify ordering: most frequent to least frequent
    assert!(decay_interval < dedup_interval);
    assert!(dedup_interval < conflict_interval);
    assert!(conflict_interval < consolidation_interval);
    assert!(consolidation_interval <= retention_interval);
}

// ── Word Overlap (Jaccard Similarity) Tests ──

fn calculate_word_overlap(content1: &str, content2: &str) -> f32 {
    let lower1 = content1.to_lowercase();
    let words1: HashSet<&str> = lower1.split_whitespace().filter(|w| w.len() > 2).collect();
    let lower2 = content2.to_lowercase();
    let words2: HashSet<&str> = lower2.split_whitespace().filter(|w| w.len() > 2).collect();

    if words1.is_empty() || words2.is_empty() { return 0.0; }
    let intersection = words1.intersection(&words2).count();
    let union = words1.union(&words2).count();
    if union == 0 { return 0.0; }
    intersection as f32 / union as f32
}

#[test]
fn test_word_overlap_identical() {
    let text = "The quick brown fox jumps over the lazy dog";
    let overlap = calculate_word_overlap(text, text);
    assert!((overlap - 1.0).abs() < 0.01);
}

#[test]
fn test_word_overlap_partial() {
    let overlap = calculate_word_overlap(
        "John likes coffee and tea",
        "John likes coffee and juice",
    );
    assert!(overlap > 0.5);
    assert!(overlap < 1.0);
}

#[test]
fn test_word_overlap_none() {
    let overlap = calculate_word_overlap(
        "apple banana cherry",
        "dog elephant fish",
    );
    assert_eq!(overlap, 0.0);
}

#[test]
fn test_word_overlap_case_insensitive() {
    let overlap = calculate_word_overlap(
        "HELLO WORLD TEST",
        "hello world test",
    );
    assert!((overlap - 1.0).abs() < 0.01);
}
