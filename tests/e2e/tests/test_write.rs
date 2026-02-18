// =============================================================================
// Write Service E2E Tests (service-write, port 8081)
// Covers: /internal/ingest, /internal/ingest/batch, /health
// =============================================================================

use crate::harness::*;
use reqwest::StatusCode;

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

#[tokio::test]
async fn write_health_returns_200() {
    let resp = client()
        .get(format!("{}/health", write_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// Internal ingest endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn write_ingest_accepts_valid_memory() {
    let body = MemoryWriteRequest {
        content: unique_content("write-ingest"),
        role: Some("user".into()),
        user_id: None,
        session_id: Some("test-session".into()),
        agent_id: None,
        metadata: None,
    };
    let resp = internal_post(&write_url(), "/internal/ingest", &body).await;
    assert!(resp.status().is_success());
    let result: MemoryWriteResponse = resp.json().await.unwrap();
    assert_eq!(result.status, "accepted");
    assert!(!result.episode_id.is_nil());
}

#[tokio::test]
async fn write_ingest_with_all_optional_fields() {
    let body = MemoryWriteRequest {
        content: unique_content("write-full"),
        role: Some("assistant".into()),
        user_id: Some(uuid::Uuid::new_v4()),
        session_id: Some("session-full".into()),
        agent_id: Some("agent-full".into()),
        metadata: Some(serde_json::json!({"source": "e2e", "tags": ["test"]})),
    };
    let resp = internal_post(&write_url(), "/internal/ingest", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn write_ingest_with_minimal_fields() {
    let body = serde_json::json!({"content": unique_content("write-minimal")});
    let resp = internal_post(&write_url(), "/internal/ingest", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn write_ingest_rejects_missing_tenant_header() {
    let resp = client()
        .post(format!("{}/internal/ingest", write_url()))
        .json(&serde_json::json!({"content": "test"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn write_ingest_generates_unique_episode_ids() {
    let body1 = MemoryWriteRequest {
        content: unique_content("unique-1"),
        role: None, user_id: None, session_id: None, agent_id: None, metadata: None,
    };
    let body2 = MemoryWriteRequest {
        content: unique_content("unique-2"),
        role: None, user_id: None, session_id: None, agent_id: None, metadata: None,
    };

    let resp1 = internal_post(&write_url(), "/internal/ingest", &body1).await;
    let resp2 = internal_post(&write_url(), "/internal/ingest", &body2).await;

    let r1: MemoryWriteResponse = resp1.json().await.unwrap();
    let r2: MemoryWriteResponse = resp2.json().await.unwrap();
    assert_ne!(r1.episode_id, r2.episode_id, "Episode IDs must be unique");
}

#[tokio::test]
async fn write_ingest_with_large_content() {
    let large_content = "a".repeat(10_000);
    let body = serde_json::json!({"content": large_content});
    let resp = internal_post(&write_url(), "/internal/ingest", &body).await;
    assert!(
        resp.status().is_success() || resp.status() == StatusCode::PAYLOAD_TOO_LARGE,
        "Large content: {}",
        resp.status()
    );
}

#[tokio::test]
async fn write_ingest_with_unicode_content() {
    let body = serde_json::json!({
        "content": "用户偏好: 他喜欢中文书法 🎨 и русский язык",
        "role": "user"
    });
    let resp = internal_post(&write_url(), "/internal/ingest", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn write_ingest_with_special_characters() {
    let body = serde_json::json!({
        "content": "Test with <html> & \"quotes\" and 'apostrophes' and\nnewlines\ttabs",
        "role": "user"
    });
    let resp = internal_post(&write_url(), "/internal/ingest", &body).await;
    assert!(resp.status().is_success());
}

// ---------------------------------------------------------------------------
// Batch ingest
// ---------------------------------------------------------------------------

#[tokio::test]
async fn write_batch_ingest_multiple_items() {
    let body = serde_json::json!([
        {"content": unique_content("batch-a"), "role": "user"},
        {"content": unique_content("batch-b"), "role": "assistant"},
        {"content": unique_content("batch-c"), "role": "user"},
    ]);
    let resp = internal_post(&write_url(), "/internal/ingest/batch", &body).await;
    assert!(resp.status().is_success(), "Batch ingest: {}", resp.status());
}

#[tokio::test]
async fn write_batch_ingest_single_item() {
    let body = serde_json::json!([
        {"content": unique_content("batch-single"), "role": "user"},
    ]);
    let resp = internal_post(&write_url(), "/internal/ingest/batch", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn write_batch_ingest_empty_array() {
    let body: Vec<serde_json::Value> = vec![];
    let resp = internal_post(&write_url(), "/internal/ingest/batch", &body).await;
    let status = resp.status();
    assert!(
        status.is_success() || status == StatusCode::BAD_REQUEST,
        "Empty batch: {}",
        status
    );
}

// ---------------------------------------------------------------------------
// Content hash deduplication (write should still accept, dedup is search-time)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn write_ingest_accepts_duplicate_content() {
    let content = unique_content("dedup-test");
    let body = serde_json::json!({"content": content});

    let resp1 = internal_post(&write_url(), "/internal/ingest", &body).await;
    let resp2 = internal_post(&write_url(), "/internal/ingest", &body).await;

    assert!(resp1.status().is_success());
    assert!(resp2.status().is_success());

    let r1: MemoryWriteResponse = resp1.json().await.unwrap();
    let r2: MemoryWriteResponse = resp2.json().await.unwrap();
    // Both accepted with different episode IDs (dedup happens later)
    assert_ne!(r1.episode_id, r2.episode_id);
}

// ---------------------------------------------------------------------------
// Role values
// ---------------------------------------------------------------------------

#[tokio::test]
async fn write_ingest_role_user() {
    let body = serde_json::json!({"content": unique_content("role-user"), "role": "user"});
    let resp = internal_post(&write_url(), "/internal/ingest", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn write_ingest_role_assistant() {
    let body = serde_json::json!({"content": unique_content("role-assistant"), "role": "assistant"});
    let resp = internal_post(&write_url(), "/internal/ingest", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn write_ingest_role_system() {
    let body = serde_json::json!({"content": unique_content("role-system"), "role": "system"});
    let resp = internal_post(&write_url(), "/internal/ingest", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn write_ingest_default_role() {
    // No role field → should default to "user"
    let body = serde_json::json!({"content": unique_content("role-default")});
    let resp = internal_post(&write_url(), "/internal/ingest", &body).await;
    assert!(resp.status().is_success());
}
