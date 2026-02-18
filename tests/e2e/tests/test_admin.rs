// =============================================================================
// Admin Service E2E Tests (service-admin, port 8084)
// Covers: update, delete, merge, audit, policies, simulate/replay
// =============================================================================

use crate::harness::*;
use reqwest::StatusCode;

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

#[tokio::test]
async fn admin_health_returns_200() {
    let resp = client()
        .get(format!("{}/health", admin_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// Update memory
// ---------------------------------------------------------------------------

#[tokio::test]
async fn admin_update_memory_not_found() {
    let fake_id = uuid::Uuid::new_v4();
    let body = UpdateMemoryRequest {
        content: Some("updated content".into()),
        status: None,
        importance: None,
    };
    let resp = internal_put(&admin_url(), &format!("/internal/memory/{}", fake_id), &body).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn admin_update_memory_content_only() {
    let fake_id = uuid::Uuid::new_v4();
    let body = UpdateMemoryRequest {
        content: Some("new content value".into()),
        status: None,
        importance: None,
    };
    let resp = internal_put(&admin_url(), &format!("/internal/memory/{}", fake_id), &body).await;
    // Not found is expected (no pre-existing memory), but the endpoint must handle it
    assert!(resp.status() == StatusCode::NOT_FOUND || resp.status().is_success());
}

#[tokio::test]
async fn admin_update_memory_importance() {
    let fake_id = uuid::Uuid::new_v4();
    let body = UpdateMemoryRequest {
        content: None,
        status: None,
        importance: Some(0.85),
    };
    let resp = internal_put(&admin_url(), &format!("/internal/memory/{}", fake_id), &body).await;
    assert!(resp.status() == StatusCode::NOT_FOUND || resp.status().is_success());
}

#[tokio::test]
async fn admin_update_memory_status_to_archived() {
    let fake_id = uuid::Uuid::new_v4();
    let body = UpdateMemoryRequest {
        content: None,
        status: Some("archived".into()),
        importance: None,
    };
    let resp = internal_put(&admin_url(), &format!("/internal/memory/{}", fake_id), &body).await;
    assert!(resp.status() == StatusCode::NOT_FOUND || resp.status().is_success());
}

#[tokio::test]
async fn admin_update_rejects_missing_tenant() {
    let fake_id = uuid::Uuid::new_v4();
    let resp = client()
        .put(format!("{}/internal/memory/{}", admin_url(), fake_id))
        .json(&serde_json::json!({"content": "test"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// Delete memory (soft delete)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn admin_delete_memory_not_found() {
    let fake_id = uuid::Uuid::new_v4();
    let resp = internal_delete(&admin_url(), &format!("/internal/memory/{}", fake_id)).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn admin_delete_rejects_missing_tenant() {
    let fake_id = uuid::Uuid::new_v4();
    let resp = client()
        .delete(format!("{}/internal/memory/{}", admin_url(), fake_id))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// Merge memories
// ---------------------------------------------------------------------------

#[tokio::test]
async fn admin_merge_with_nonexistent_sources() {
    let body = MergeRequest {
        source_ids: vec![uuid::Uuid::new_v4(), uuid::Uuid::new_v4()],
        merged_content: "Merged content from two sources".into(),
        user_id: Some(uuid::Uuid::new_v4()),
        scope: Some("user".into()),
        kind: Some("fact".into()),
        confidence: Some(0.9),
        importance: Some(0.8),
    };
    let resp = internal_post(&admin_url(), "/internal/memory/merge", &body).await;
    // May fail because sources don't exist, or succeed creating the merged memory
    let status = resp.status();
    assert!(
        status.is_success() || status == StatusCode::NOT_FOUND || status == StatusCode::BAD_REQUEST,
        "Merge with nonexistent: {}",
        status
    );
}

#[tokio::test]
async fn admin_merge_with_minimal_fields() {
    let body = MergeRequest {
        source_ids: vec![uuid::Uuid::new_v4()],
        merged_content: "Minimal merge".into(),
        user_id: None,
        scope: None,
        kind: None,
        confidence: None,
        importance: None,
    };
    let resp = internal_post(&admin_url(), "/internal/memory/merge", &body).await;
    let status = resp.status();
    assert!(
        status.is_success() || status == StatusCode::NOT_FOUND || status == StatusCode::BAD_REQUEST,
        "Minimal merge: {}",
        status
    );
}

#[tokio::test]
async fn admin_merge_with_empty_source_ids() {
    let body = MergeRequest {
        source_ids: vec![],
        merged_content: "Empty sources merge".into(),
        user_id: None, scope: None, kind: None, confidence: None, importance: None,
    };
    let resp = internal_post(&admin_url(), "/internal/memory/merge", &body).await;
    let status = resp.status();
    assert!(
        status == StatusCode::BAD_REQUEST || status.is_success(),
        "Empty sources merge: {}",
        status
    );
}

// ---------------------------------------------------------------------------
// Audit log
// ---------------------------------------------------------------------------

#[tokio::test]
async fn admin_list_audit_log() {
    let resp = internal_get(&admin_url(), "/internal/audit").await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn admin_list_audit_log_with_limit() {
    let resp = internal_get(&admin_url(), "/internal/audit?limit=5").await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn admin_list_audit_log_by_memory_id() {
    let fake_id = uuid::Uuid::new_v4();
    let resp = internal_get(
        &admin_url(),
        &format!("/internal/audit?memory_id={}", fake_id),
    )
    .await;
    assert!(resp.status().is_success());
}

// ---------------------------------------------------------------------------
// Policies
// ---------------------------------------------------------------------------

#[tokio::test]
async fn admin_list_policies() {
    let resp = internal_get(&admin_url(), "/internal/policies").await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn admin_create_retention_policy() {
    let body = CreatePolicyRequest {
        name: format!("retention-{}", &uuid::Uuid::new_v4().to_string()[..8]),
        rule_type: "retention".into(),
        config: serde_json::json!({"retention_days": 90, "scope": "user"}),
    };
    let resp = internal_post(&admin_url(), "/internal/policies", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn admin_create_governance_policy() {
    let body = CreatePolicyRequest {
        name: format!("governance-{}", &uuid::Uuid::new_v4().to_string()[..8]),
        rule_type: "governance".into(),
        config: serde_json::json!({"require_pii_redaction": true, "max_retention_days": 365}),
    };
    let resp = internal_post(&admin_url(), "/internal/policies", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn admin_create_policy_rejects_missing_tenant() {
    let resp = client()
        .post(format!("{}/internal/policies", admin_url()))
        .json(&serde_json::json!({
            "name": "test",
            "rule_type": "retention",
            "config": {}
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// Simulate / Replay (point-in-time)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn admin_simulate_replay_current_time() {
    let body = SimulateRequest {
        point_in_time: chrono::Utc::now(),
        user_id: None,
        limit: Some(10),
    };
    let resp = internal_post(&admin_url(), "/internal/simulate/replay", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn admin_simulate_replay_past_time() {
    let past = chrono::Utc::now() - chrono::Duration::days(30);
    let body = SimulateRequest {
        point_in_time: past,
        user_id: None,
        limit: Some(10),
    };
    let resp = internal_post(&admin_url(), "/internal/simulate/replay", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn admin_simulate_replay_with_user_filter() {
    let body = SimulateRequest {
        point_in_time: chrono::Utc::now(),
        user_id: Some(uuid::Uuid::new_v4()),
        limit: Some(5),
    };
    let resp = internal_post(&admin_url(), "/internal/simulate/replay", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn admin_simulate_replay_far_future() {
    let future = chrono::Utc::now() + chrono::Duration::days(365 * 10);
    let body = SimulateRequest {
        point_in_time: future,
        user_id: None,
        limit: Some(10),
    };
    let resp = internal_post(&admin_url(), "/internal/simulate/replay", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn admin_simulate_replay_with_limit_zero() {
    let body = SimulateRequest {
        point_in_time: chrono::Utc::now(),
        user_id: None,
        limit: Some(0),
    };
    let resp = internal_post(&admin_url(), "/internal/simulate/replay", &body).await;
    assert!(resp.status().is_success());
}
