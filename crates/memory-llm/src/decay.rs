//! Memory importance decay and scoring.
//!
//! Implements time-based decay that gradually reduces memory importance over time,
//! ensuring recent memories are prioritized while old, unaccessed memories fade.
//! This is a key differentiator vs Mem0 (no decay) and matches Zep's temporal model.
//!
//! ## Decay Functions
//!
//! - **Exponential decay**: `score = importance * e^(-λt)` where t = hours since last access
//! - **Linear decay**: `score = importance * max(0, 1 - rate * t)`
//! - **Step decay**: importance drops at fixed intervals
//!
//! ## Access-Based Reinforcement
//!
//! Each time a memory is retrieved (search hit), its `last_accessed_at` is updated
//! and its importance gets a small boost, preventing frequently-used memories from decaying.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Decay configuration — loaded from system_config table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayConfig {
    /// Enable decay scoring in retrieval.
    pub enabled: bool,
    /// Decay function: "exponential", "linear", "step"
    pub function: DecayFunction,
    /// Half-life in hours (for exponential: time for score to halve).
    /// Default: 168 hours (7 days).
    pub half_life_hours: f64,
    /// Linear decay rate per hour (for linear mode only).
    pub linear_rate_per_hour: f64,
    /// Step intervals in hours and their multipliers (for step mode).
    pub step_intervals: Vec<(f64, f64)>,
    /// Minimum score floor — memories never decay below this.
    pub min_score: f32,
    /// Boost applied when a memory is accessed (retrieved in search).
    pub access_boost: f32,
    /// Maximum importance cap after boosting.
    pub max_importance: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecayFunction {
    Exponential,
    Linear,
    Step,
}

impl Default for DecayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            function: DecayFunction::Exponential,
            half_life_hours: 168.0, // 7 days
            linear_rate_per_hour: 0.001,
            step_intervals: vec![
                (24.0, 0.95),   // After 1 day: 95% of original
                (168.0, 0.80),  // After 1 week: 80%
                (720.0, 0.50),  // After 1 month: 50%
                (2160.0, 0.20), // After 3 months: 20%
            ],
            min_score: 0.05,
            access_boost: 0.05,
            max_importance: 1.0,
        }
    }
}

/// Calculate decayed importance score for a memory.
///
/// # Arguments
/// * `importance` - Original importance (0.0-1.0)
/// * `created_at` - When the memory was created
/// * `last_accessed_at` - When the memory was last retrieved (None = never accessed since creation)
/// * `now` - Current time
/// * `config` - Decay configuration
pub fn decayed_score(
    importance: f32,
    created_at: DateTime<Utc>,
    last_accessed_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
    config: &DecayConfig,
) -> f32 {
    if !config.enabled {
        return importance;
    }

    // Use the more recent of created_at or last_accessed_at as the reference point
    let reference_time = last_accessed_at.unwrap_or(created_at);
    let hours_elapsed = (now - reference_time).num_seconds() as f64 / 3600.0;

    if hours_elapsed <= 0.0 {
        return importance;
    }

    let decay_multiplier = match config.function {
        DecayFunction::Exponential => {
            // λ = ln(2) / half_life
            let lambda = (2.0_f64).ln() / config.half_life_hours;
            (-lambda * hours_elapsed).exp() as f32
        }
        DecayFunction::Linear => {
            let decayed = 1.0 - config.linear_rate_per_hour as f32 * hours_elapsed as f32;
            decayed.max(0.0)
        }
        DecayFunction::Step => {
            let mut multiplier = 1.0f32;
            for (threshold_hours, factor) in &config.step_intervals {
                if hours_elapsed >= *threshold_hours {
                    multiplier = *factor as f32;
                }
            }
            multiplier
        }
    };

    let score = importance * decay_multiplier;
    score.max(config.min_score)
}

/// Apply access boost to a memory's importance.
/// Called when a memory appears in search results.
pub fn apply_access_boost(current_importance: f32, config: &DecayConfig) -> f32 {
    let boosted = current_importance + config.access_boost;
    boosted.min(config.max_importance)
}

/// Batch-apply decay to multiple memories for efficient retrieval scoring.
/// Returns Vec of (memory_id, decayed_score).
pub fn batch_decay_scores(
    memories: &[(uuid::Uuid, f32, DateTime<Utc>, Option<DateTime<Utc>>)], // (id, importance, created_at, last_accessed)
    now: DateTime<Utc>,
    config: &DecayConfig,
) -> Vec<(uuid::Uuid, f32)> {
    memories
        .iter()
        .map(|(id, importance, created_at, last_accessed)| {
            let score = decayed_score(*importance, *created_at, *last_accessed, now, config);
            (*id, score)
        })
        .collect()
}

/// Build DecayConfig from admin DB.
pub async fn build_decay_config_from_db(pool: &sqlx::PgPool) -> DecayConfig {
    use memory_common::db_config;

    let enabled = db_config::load_bool(pool, "memory.decay_enabled", "MEMORY_DECAY_ENABLED", true).await;
    let function_str = db_config::load_string(pool, "memory.decay_function", "MEMORY_DECAY_FUNCTION", "exponential").await;
    let half_life = db_config::load_f64(pool, "memory.decay_half_life_hours", "MEMORY_DECAY_HALF_LIFE", 168.0).await;
    let min_score = db_config::load_f64(pool, "memory.decay_min_score", "MEMORY_DECAY_MIN_SCORE", 0.05).await as f32;
    let access_boost = db_config::load_f64(pool, "memory.access_boost", "MEMORY_ACCESS_BOOST", 0.05).await as f32;

    let function = match function_str.as_str() {
        "linear" => DecayFunction::Linear,
        "step" => DecayFunction::Step,
        _ => DecayFunction::Exponential,
    };

    DecayConfig {
        enabled,
        function,
        half_life_hours: half_life,
        min_score,
        access_boost,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use uuid::Uuid;

    fn default_config() -> DecayConfig {
        DecayConfig::default()
    }

    #[test]
    fn test_no_decay_when_just_created() {
        let config = default_config();
        let now = Utc::now();
        let score = decayed_score(0.8, now, None, now, &config);
        assert_eq!(score, 0.8);
    }

    #[test]
    fn test_exponential_decay_half_life() {
        let config = default_config(); // half_life = 168h (7 days)
        let now = Utc::now();
        let created = now - Duration::hours(168); // exactly one half-life ago

        let score = decayed_score(1.0, created, None, now, &config);
        // After one half-life, score should be ~0.5
        assert!((score - 0.5).abs() < 0.02, "Expected ~0.5, got {}", score);
    }

    #[test]
    fn test_exponential_decay_two_half_lives() {
        let config = default_config();
        let now = Utc::now();
        let created = now - Duration::hours(336); // 2 half-lives

        let score = decayed_score(1.0, created, None, now, &config);
        // After two half-lives, score should be ~0.25
        assert!((score - 0.25).abs() < 0.02, "Expected ~0.25, got {}", score);
    }

    #[test]
    fn test_min_score_floor() {
        let config = default_config();
        let now = Utc::now();
        let created = now - Duration::hours(10000); // very old

        let score = decayed_score(0.5, created, None, now, &config);
        assert!(score >= config.min_score, "Score {} should be >= min {}", score, config.min_score);
    }

    #[test]
    fn test_access_resets_decay_clock() {
        let config = default_config();
        let now = Utc::now();
        let created = now - Duration::hours(336); // old
        let accessed = now - Duration::hours(1); // recently accessed

        let score_no_access = decayed_score(0.8, created, None, now, &config);
        let score_with_access = decayed_score(0.8, created, Some(accessed), now, &config);

        assert!(
            score_with_access > score_no_access,
            "Accessed score ({}) should be > non-accessed ({})",
            score_with_access,
            score_no_access
        );
    }

    #[test]
    fn test_linear_decay() {
        let config = DecayConfig {
            function: DecayFunction::Linear,
            linear_rate_per_hour: 0.01,
            ..default_config()
        };
        let now = Utc::now();
        let created = now - Duration::hours(50);

        let score = decayed_score(1.0, created, None, now, &config);
        // 1.0 * (1 - 0.01 * 50) = 0.5
        assert!((score - 0.5).abs() < 0.02, "Expected ~0.5, got {}", score);
    }

    #[test]
    fn test_step_decay() {
        let config = DecayConfig {
            function: DecayFunction::Step,
            ..default_config()
        };
        let now = Utc::now();

        // After 2 hours: no step threshold crossed → 1.0
        let score_2h = decayed_score(1.0, now - Duration::hours(2), None, now, &config);
        assert_eq!(score_2h, 1.0);

        // After 25 hours: crossed 24h threshold → 0.95
        let score_25h = decayed_score(1.0, now - Duration::hours(25), None, now, &config);
        assert!((score_25h - 0.95).abs() < 0.01);

        // After 200 hours: crossed 168h threshold → 0.80
        let score_200h = decayed_score(1.0, now - Duration::hours(200), None, now, &config);
        assert!((score_200h - 0.80).abs() < 0.01);
    }

    #[test]
    fn test_disabled_decay() {
        let config = DecayConfig {
            enabled: false,
            ..default_config()
        };
        let now = Utc::now();
        let created = now - Duration::hours(10000);

        let score = decayed_score(0.8, created, None, now, &config);
        assert_eq!(score, 0.8); // No decay when disabled
    }

    #[test]
    fn test_access_boost() {
        let config = default_config();
        let boosted = apply_access_boost(0.5, &config);
        assert_eq!(boosted, 0.55);
    }

    #[test]
    fn test_access_boost_capped_at_max() {
        let config = default_config();
        let boosted = apply_access_boost(0.98, &config);
        assert_eq!(boosted, 1.0);
    }

    #[test]
    fn test_batch_decay_scores() {
        let config = default_config();
        let now = Utc::now();

        let memories = vec![
            (Uuid::new_v4(), 0.9, now, None),                           // just created
            (Uuid::new_v4(), 0.9, now - Duration::hours(168), None),    // 7 days old
            (Uuid::new_v4(), 0.9, now - Duration::hours(336), None),    // 14 days old
        ];

        let scores = batch_decay_scores(&memories, now, &config);
        assert_eq!(scores.len(), 3);

        // Scores should decrease with age
        assert!(scores[0].1 > scores[1].1, "Recent > 7 days");
        assert!(scores[1].1 > scores[2].1, "7 days > 14 days");
    }
}
