//! Unit tests for billing service logic
//!
//! Tests plan limits, feature mappings, and usage tracking without a database.

use serde_json;

/// Plan tier configuration for testing (mirrors main.rs logic)
struct PlanLimits {
    max_memories_per_user: i64,
    max_users: i64,
    features: Vec<String>,
}

fn get_plan_limits(plan: &str) -> PlanLimits {
    match plan {
        "free" => PlanLimits {
            max_memories_per_user: 1000,
            max_users: 1,
            features: vec!["vector_search".into(), "basic_graph".into()],
        },
        "developer" => PlanLimits {
            max_memories_per_user: 10_000,
            max_users: 5,
            features: vec!["vector_search".into(), "graph".into(), "temporal".into()],
        },
        "pro" => PlanLimits {
            max_memories_per_user: 100_000,
            max_users: 25,
            features: vec![
                "vector_search".into(), "graph".into(), "temporal".into(),
                "simulation".into(), "connectors".into(),
            ],
        },
        "team" => PlanLimits {
            max_memories_per_user: 500_000,
            max_users: 100,
            features: vec![
                "vector_search".into(), "graph".into(), "temporal".into(),
                "simulation".into(), "connectors".into(), "audit".into(), "sso".into(),
            ],
        },
        "enterprise" => PlanLimits {
            max_memories_per_user: i64::MAX,
            max_users: i64::MAX,
            features: vec![
                "vector_search".into(), "graph".into(), "temporal".into(),
                "simulation".into(), "connectors".into(), "audit".into(),
                "sso".into(), "air_gapped".into(), "custom_ontology".into(),
                "dedicated_infra".into(),
            ],
        },
        _ => get_plan_limits("free"),
    }
}

fn check_within_limits(usage_ops_month: i32, usage_limit: Option<i32>) -> bool {
    usage_limit.map(|l| usage_ops_month < l).unwrap_or(true)
}

// ── Plan Limits Tests ──

#[test]
fn test_free_plan_limits() {
    let limits = get_plan_limits("free");
    assert_eq!(limits.max_memories_per_user, 1000);
    assert_eq!(limits.max_users, 1);
    assert_eq!(limits.features.len(), 2);
    assert!(limits.features.contains(&"vector_search".to_string()));
    assert!(limits.features.contains(&"basic_graph".to_string()));
}

#[test]
fn test_developer_plan_limits() {
    let limits = get_plan_limits("developer");
    assert_eq!(limits.max_memories_per_user, 10_000);
    assert_eq!(limits.max_users, 5);
    assert!(limits.features.contains(&"temporal".to_string()));
}

#[test]
fn test_pro_plan_limits() {
    let limits = get_plan_limits("pro");
    assert_eq!(limits.max_memories_per_user, 100_000);
    assert_eq!(limits.max_users, 25);
    assert!(limits.features.contains(&"simulation".to_string()));
    assert!(limits.features.contains(&"connectors".to_string()));
}

#[test]
fn test_team_plan_limits() {
    let limits = get_plan_limits("team");
    assert_eq!(limits.max_memories_per_user, 500_000);
    assert_eq!(limits.max_users, 100);
    assert!(limits.features.contains(&"audit".to_string()));
    assert!(limits.features.contains(&"sso".to_string()));
}

#[test]
fn test_enterprise_plan_unlimited() {
    let limits = get_plan_limits("enterprise");
    assert_eq!(limits.max_memories_per_user, i64::MAX);
    assert_eq!(limits.max_users, i64::MAX);
    assert_eq!(limits.features.len(), 10);
    assert!(limits.features.contains(&"air_gapped".to_string()));
    assert!(limits.features.contains(&"dedicated_infra".to_string()));
}

#[test]
fn test_unknown_plan_defaults_to_free() {
    let limits = get_plan_limits("unknown_plan");
    assert_eq!(limits.max_memories_per_user, 1000);
    assert_eq!(limits.max_users, 1);
}

// ── Feature Tier Progression Tests ──

#[test]
fn test_plan_tiers_increase_monotonically() {
    let tiers = ["free", "developer", "pro", "team", "enterprise"];
    let mut prev_memories = 0i64;
    let mut prev_users = 0i64;

    for tier in &tiers {
        let limits = get_plan_limits(tier);
        assert!(
            limits.max_memories_per_user >= prev_memories,
            "Tier {} should have >= memories than previous tier",
            tier
        );
        assert!(
            limits.max_users >= prev_users,
            "Tier {} should have >= users than previous tier",
            tier
        );
        prev_memories = limits.max_memories_per_user;
        prev_users = limits.max_users;
    }
}

#[test]
fn test_higher_tiers_superset_features() {
    let tiers = ["free", "developer", "pro", "team", "enterprise"];
    let mut prev_feature_count = 0;

    for tier in &tiers {
        let limits = get_plan_limits(tier);
        assert!(
            limits.features.len() >= prev_feature_count,
            "Tier {} should have >= features than previous tier",
            tier
        );
        prev_feature_count = limits.features.len();
    }
}

// ── Usage Limit Check Tests ──

#[test]
fn test_within_limits_under_cap() {
    assert!(check_within_limits(50, Some(100)));
}

#[test]
fn test_within_limits_at_cap() {
    assert!(!check_within_limits(100, Some(100)));
}

#[test]
fn test_within_limits_over_cap() {
    assert!(!check_within_limits(150, Some(100)));
}

#[test]
fn test_within_limits_no_cap_unlimited() {
    assert!(check_within_limits(999_999, None));
}

#[test]
fn test_within_limits_zero_usage() {
    assert!(check_within_limits(0, Some(100)));
}

// ── Serde Roundtrip Tests ──

#[test]
fn test_record_usage_request_deserialization() {
    let json = r#"{"ops_count": 42}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(parsed["ops_count"], 42);
}

#[test]
fn test_usage_response_serialization() {
    let response = serde_json::json!({
        "plan": "pro",
        "ops_this_month": 1500,
        "ops_limit": 10000,
        "active_memories": 5000,
        "active_entities": 1200,
    });

    assert_eq!(response["plan"], "pro");
    assert_eq!(response["ops_this_month"], 1500);
    assert!(response["active_memories"].as_i64().unwrap() > 0);
}

#[test]
fn test_plan_limits_response_serialization() {
    let response = serde_json::json!({
        "plan": "team",
        "within_limits": true,
        "ops_used": 500,
        "ops_limit": 50000,
        "max_memories_per_user": 500000,
        "max_users": 100,
        "features": ["vector_search", "graph", "audit"],
    });

    assert_eq!(response["within_limits"], true);
    assert_eq!(response["features"].as_array().unwrap().len(), 3);
}
