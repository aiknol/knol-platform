//! Unit tests for admin service types and logic
//!
//! Tests request/response serialization, merge logic, and simulation parameters.

use chrono::{Duration, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Replicated types for testing (mirrors main.rs) ──

#[derive(Debug, Deserialize, Serialize)]
struct UpdateMemoryRequest {
    content: Option<String>,
    status: Option<String>,
    importance: Option<f32>,
}

#[derive(Debug, Deserialize, Serialize)]
struct MergeRequest {
    source_ids: Vec<Uuid>,
    merged_content: String,
    user_id: Option<Uuid>,
    scope: Option<String>,
    kind: Option<String>,
    confidence: Option<f32>,
    importance: Option<f32>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SimulateRequest {
    point_in_time: chrono::DateTime<Utc>,
    user_id: Option<Uuid>,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CreatePolicyRequest {
    name: String,
    rule_type: String,
    config: serde_json::Value,
}

// ── UpdateMemoryRequest Tests ──

#[test]
fn test_update_request_partial_content() {
    let json = r#"{"content": "Updated memory content"}"#;
    let req: UpdateMemoryRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.content.unwrap(), "Updated memory content");
    assert!(req.status.is_none());
    assert!(req.importance.is_none());
}

#[test]
fn test_update_request_partial_status() {
    let json = r#"{"status": "archived"}"#;
    let req: UpdateMemoryRequest = serde_json::from_str(json).unwrap();
    assert!(req.content.is_none());
    assert_eq!(req.status.unwrap(), "archived");
}

#[test]
fn test_update_request_partial_importance() {
    let json = r#"{"importance": 0.95}"#;
    let req: UpdateMemoryRequest = serde_json::from_str(json).unwrap();
    assert!(req.content.is_none());
    assert!((req.importance.unwrap() - 0.95).abs() < 0.01);
}

#[test]
fn test_update_request_full() {
    let json = r#"{"content": "new content", "status": "active", "importance": 0.8}"#;
    let req: UpdateMemoryRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.content.unwrap(), "new content");
    assert_eq!(req.status.unwrap(), "active");
    assert!((req.importance.unwrap() - 0.8).abs() < 0.01);
}

#[test]
fn test_update_request_empty() {
    let json = r#"{}"#;
    let req: UpdateMemoryRequest = serde_json::from_str(json).unwrap();
    assert!(req.content.is_none());
    assert!(req.status.is_none());
    assert!(req.importance.is_none());
}

// ── MergeRequest Tests ──

#[test]
fn test_merge_request_deserialization() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let json = serde_json::json!({
        "source_ids": [id1, id2],
        "merged_content": "Combined memory content",
        "confidence": 0.9,
        "importance": 0.7,
    });

    let req: MergeRequest = serde_json::from_value(json).unwrap();
    assert_eq!(req.source_ids.len(), 2);
    assert_eq!(req.merged_content, "Combined memory content");
    assert!(req.user_id.is_none());
    assert!(req.scope.is_none());
    assert!((req.confidence.unwrap() - 0.9).abs() < 0.01);
}

#[test]
fn test_merge_request_with_all_fields() {
    let user_id = Uuid::new_v4();
    let json = serde_json::json!({
        "source_ids": [Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()],
        "merged_content": "Synthesized knowledge about the project",
        "user_id": user_id,
        "scope": "agent",
        "kind": "summary",
        "confidence": 0.85,
        "importance": 0.6,
    });

    let req: MergeRequest = serde_json::from_value(json).unwrap();
    assert_eq!(req.source_ids.len(), 3);
    assert_eq!(req.scope.unwrap(), "agent");
    assert_eq!(req.kind.unwrap(), "summary");
    assert_eq!(req.user_id.unwrap(), user_id);
}

#[test]
fn test_merge_request_defaults() {
    // scope defaults to "user", kind defaults to "fact" in the handler
    let json = serde_json::json!({
        "source_ids": [Uuid::new_v4()],
        "merged_content": "Single source merge",
    });

    let req: MergeRequest = serde_json::from_value(json).unwrap();
    let scope = req.scope.as_deref().unwrap_or("user");
    let kind = req.kind.as_deref().unwrap_or("fact");
    assert_eq!(scope, "user");
    assert_eq!(kind, "fact");
}

// ── Merge Metadata Tests ──

#[test]
fn test_merge_metadata_construction() {
    let source_ids = vec![Uuid::new_v4(), Uuid::new_v4()];
    let metadata = serde_json::json!({ "merged_from": source_ids });
    let merged_from = metadata["merged_from"].as_array().unwrap();
    assert_eq!(merged_from.len(), 2);
}

// ── SimulateRequest Tests ──

#[test]
fn test_simulate_request_point_in_time() {
    let past = Utc::now() - Duration::days(30);
    let json = serde_json::json!({
        "point_in_time": past.to_rfc3339(),
        "limit": 50,
    });

    let req: SimulateRequest = serde_json::from_value(json).unwrap();
    assert_eq!(req.limit.unwrap(), 50);
    assert!(req.user_id.is_none());
    // Point in time should be roughly 30 days ago
    let diff = Utc::now() - req.point_in_time;
    assert!(diff.num_days() >= 29 && diff.num_days() <= 31);
}

#[test]
fn test_simulate_request_default_limit() {
    let json = serde_json::json!({
        "point_in_time": Utc::now().to_rfc3339(),
    });

    let req: SimulateRequest = serde_json::from_value(json).unwrap();
    let limit = req.limit.unwrap_or(100) as i64;
    assert_eq!(limit, 100);
}

// ── CreatePolicyRequest Tests ──

#[test]
fn test_retention_policy_creation() {
    let json = serde_json::json!({
        "name": "90-day retention",
        "rule_type": "retention",
        "config": { "days": 90, "scope": "all" },
    });

    let req: CreatePolicyRequest = serde_json::from_value(json).unwrap();
    assert_eq!(req.name, "90-day retention");
    assert_eq!(req.rule_type, "retention");
    assert_eq!(req.config["days"], 90);
}

#[test]
fn test_governance_policy_creation() {
    let json = serde_json::json!({
        "name": "PII scrubbing",
        "rule_type": "governance",
        "config": { "redact_pii": true, "pii_types": ["email", "ssn"] },
    });

    let req: CreatePolicyRequest = serde_json::from_value(json).unwrap();
    assert_eq!(req.rule_type, "governance");
    assert!(req.config["redact_pii"].as_bool().unwrap());
}

// ── Audit Diff Construction Tests ──

#[test]
fn test_audit_diff_captures_changes() {
    let before = serde_json::json!({
        "content": "old content",
        "status": "active",
        "importance": 0.5,
    });

    let after = serde_json::json!({
        "content": "new content",
        "status": "active",
        "importance": 0.8,
    });

    let diff = serde_json::json!({ "before": before, "after": after });
    assert_ne!(diff["before"]["content"], diff["after"]["content"]);
    assert_ne!(diff["before"]["importance"], diff["after"]["importance"]);
    assert_eq!(diff["before"]["status"], diff["after"]["status"]);
}

// ── Tenant ID Extraction Tests ──

#[test]
fn test_valid_uuid_parsing() {
    let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
    let parsed = Uuid::parse_str(uuid_str);
    assert!(parsed.is_ok());
}

#[test]
fn test_invalid_uuid_parsing() {
    let invalid = "not-a-uuid";
    let parsed = Uuid::parse_str(invalid);
    assert!(parsed.is_err());
}

// ── Point-in-Time Query Logic Tests ──

#[test]
fn test_validity_window_logic() {
    let valid_from = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    let valid_to = Some(Utc.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap());
    let query_time = Utc.with_ymd_and_hms(2025, 3, 15, 0, 0, 0).unwrap();

    // Memory should be visible at query_time
    let is_valid = query_time >= valid_from && valid_to.map(|vt| query_time < vt).unwrap_or(true);
    assert!(is_valid);
}

#[test]
fn test_validity_window_before_start() {
    let valid_from = Utc.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap();
    let query_time = Utc.with_ymd_and_hms(2025, 3, 15, 0, 0, 0).unwrap();

    let is_valid = query_time >= valid_from;
    assert!(!is_valid);
}

#[test]
fn test_validity_window_after_end() {
    let valid_from = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    let valid_to = Some(Utc.with_ymd_and_hms(2025, 3, 1, 0, 0, 0).unwrap());
    let query_time = Utc.with_ymd_and_hms(2025, 6, 15, 0, 0, 0).unwrap();

    let is_valid = query_time >= valid_from && valid_to.map(|vt| query_time < vt).unwrap_or(true);
    assert!(!is_valid);
}

#[test]
fn test_validity_window_no_end_always_valid() {
    let valid_from = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    let valid_to: Option<chrono::DateTime<Utc>> = None;
    let query_time = Utc.with_ymd_and_hms(2099, 12, 31, 0, 0, 0).unwrap();

    let is_valid = query_time >= valid_from && valid_to.map(|vt| query_time < vt).unwrap_or(true);
    assert!(is_valid);
}
