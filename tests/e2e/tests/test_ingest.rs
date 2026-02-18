// =============================================================================
// Ingest Service E2E Tests (service-ingest, port 8087)
// Covers: /internal/connectors, /internal/connectors/webhook,
//         /internal/connectors/bulk, /health
// =============================================================================

use crate::harness::*;
use reqwest::StatusCode;

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ingest_health_returns_200() {
    let resp = client()
        .get(format!("{}/health", ingest_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// List connectors
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ingest_list_connectors_returns_array() {
    let resp = internal_get(&ingest_url(), "/internal/connectors").await;
    assert!(resp.status().is_success());
    let connectors: Vec<ConnectorInfo> = resp.json().await.unwrap();
    assert!(!connectors.is_empty(), "Should have at least one connector");
}

#[tokio::test]
async fn ingest_list_connectors_includes_webhook() {
    let resp = internal_get(&ingest_url(), "/internal/connectors").await;
    let connectors: Vec<ConnectorInfo> = resp.json().await.unwrap();
    let webhook = connectors.iter().find(|c| c.id == "webhook");
    assert!(webhook.is_some(), "Webhook connector should be listed");
    assert_eq!(webhook.unwrap().status, "available");
}

#[tokio::test]
async fn ingest_list_connectors_includes_coming_soon() {
    let resp = internal_get(&ingest_url(), "/internal/connectors").await;
    let connectors: Vec<ConnectorInfo> = resp.json().await.unwrap();
    let coming_soon: Vec<&ConnectorInfo> = connectors.iter().filter(|c| c.status == "coming_soon").collect();
    assert!(
        !coming_soon.is_empty(),
        "Should have at least one coming_soon connector (slack, github, email)"
    );
}

#[tokio::test]
async fn ingest_list_connectors_has_valid_fields() {
    let resp = internal_get(&ingest_url(), "/internal/connectors").await;
    let connectors: Vec<ConnectorInfo> = resp.json().await.unwrap();
    for c in &connectors {
        assert!(!c.id.is_empty(), "Connector id should not be empty");
        assert!(!c.name.is_empty(), "Connector name should not be empty");
        assert!(!c.description.is_empty(), "Connector description should not be empty");
        assert!(
            c.status == "available" || c.status == "coming_soon",
            "Invalid connector status: {}",
            c.status
        );
    }
}

// ---------------------------------------------------------------------------
// Webhook ingest
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ingest_webhook_single_item() {
    let body = WebhookPayload {
        source: "test-webhook".into(),
        items: vec![WebhookItem {
            content: unique_content("webhook-single"),
            user_id: None,
            role: Some("user".into()),
            session_id: None,
            agent_id: None,
            metadata: None,
        }],
    };
    let resp = internal_post(&ingest_url(), "/internal/connectors/webhook", &body).await;
    assert!(resp.status().is_success());
    let result: IngestResponse = resp.json().await.unwrap();
    assert_eq!(result.ingested, 1);
    assert_eq!(result.total, 1);
    assert_eq!(result.source, "test-webhook");
}

#[tokio::test]
async fn ingest_webhook_multiple_items() {
    let body = WebhookPayload {
        source: "slack-integration".into(),
        items: vec![
            WebhookItem {
                content: unique_content("wh-multi-1"),
                user_id: Some(uuid::Uuid::new_v4()),
                role: Some("user".into()),
                session_id: Some("slack-channel-1".into()),
                agent_id: None,
                metadata: Some(serde_json::json!({"channel": "#general"})),
            },
            WebhookItem {
                content: unique_content("wh-multi-2"),
                user_id: Some(uuid::Uuid::new_v4()),
                role: Some("user".into()),
                session_id: Some("slack-channel-1".into()),
                agent_id: None,
                metadata: Some(serde_json::json!({"channel": "#general"})),
            },
            WebhookItem {
                content: unique_content("wh-multi-3"),
                user_id: None,
                role: None,
                session_id: None,
                agent_id: None,
                metadata: None,
            },
        ],
    };
    let resp = internal_post(&ingest_url(), "/internal/connectors/webhook", &body).await;
    assert!(resp.status().is_success());
    let result: IngestResponse = resp.json().await.unwrap();
    assert_eq!(result.total, 3);
    assert!(result.ingested <= 3);
}

#[tokio::test]
async fn ingest_webhook_empty_items() {
    let body = WebhookPayload {
        source: "empty-test".into(),
        items: vec![],
    };
    let resp = internal_post(&ingest_url(), "/internal/connectors/webhook", &body).await;
    let status = resp.status();
    assert!(
        status.is_success() || status == StatusCode::BAD_REQUEST,
        "Empty webhook: {}",
        status
    );
}

#[tokio::test]
async fn ingest_webhook_with_metadata() {
    let body = WebhookPayload {
        source: "github".into(),
        items: vec![WebhookItem {
            content: unique_content("github-pr-comment"),
            user_id: None,
            role: Some("user".into()),
            session_id: None,
            agent_id: None,
            metadata: Some(serde_json::json!({
                "repo": "knol-dev/knol",
                "pr_number": 42,
                "author": "octocat",
                "type": "pr_comment"
            })),
        }],
    };
    let resp = internal_post(&ingest_url(), "/internal/connectors/webhook", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn ingest_webhook_rejects_missing_tenant() {
    let resp = client()
        .post(format!("{}/internal/connectors/webhook", ingest_url()))
        .json(&serde_json::json!({
            "source": "test",
            "items": [{"content": "test"}]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// Bulk ingest
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ingest_bulk_multiple_texts() {
    let body = BulkIngestRequest {
        texts: vec![
            unique_content("bulk-1"),
            unique_content("bulk-2"),
            unique_content("bulk-3"),
            unique_content("bulk-4"),
            unique_content("bulk-5"),
        ],
        user_id: Some(uuid::Uuid::new_v4()),
        session_id: Some("bulk-session".into()),
        agent_id: None,
        metadata: Some(serde_json::json!({"source": "bulk-import"})),
    };
    let resp = internal_post(&ingest_url(), "/internal/connectors/bulk", &body).await;
    assert!(resp.status().is_success());
    let result: IngestResponse = resp.json().await.unwrap();
    assert_eq!(result.total, 5);
}

#[tokio::test]
async fn ingest_bulk_single_text() {
    let body = BulkIngestRequest {
        texts: vec![unique_content("bulk-single")],
        user_id: None,
        session_id: None,
        agent_id: None,
        metadata: None,
    };
    let resp = internal_post(&ingest_url(), "/internal/connectors/bulk", &body).await;
    assert!(resp.status().is_success());
    let result: IngestResponse = resp.json().await.unwrap();
    assert_eq!(result.total, 1);
}

#[tokio::test]
async fn ingest_bulk_empty_texts() {
    let body = BulkIngestRequest {
        texts: vec![],
        user_id: None,
        session_id: None,
        agent_id: None,
        metadata: None,
    };
    let resp = internal_post(&ingest_url(), "/internal/connectors/bulk", &body).await;
    let status = resp.status();
    assert!(
        status.is_success() || status == StatusCode::BAD_REQUEST,
        "Empty bulk: {}",
        status
    );
}

#[tokio::test]
async fn ingest_bulk_large_batch() {
    let texts: Vec<String> = (0..50).map(|i| format!("Bulk item number {} - {}", i, unique_content("bulk-large"))).collect();
    let body = BulkIngestRequest {
        texts,
        user_id: None,
        session_id: None,
        agent_id: None,
        metadata: None,
    };
    let resp = internal_post(&ingest_url(), "/internal/connectors/bulk", &body).await;
    assert!(resp.status().is_success());
    let result: IngestResponse = resp.json().await.unwrap();
    assert_eq!(result.total, 50);
}

#[tokio::test]
async fn ingest_bulk_with_all_optional_fields() {
    let body = BulkIngestRequest {
        texts: vec![unique_content("bulk-full")],
        user_id: Some(uuid::Uuid::new_v4()),
        session_id: Some("session-bulk".into()),
        agent_id: Some("agent-bulk".into()),
        metadata: Some(serde_json::json!({"import": true, "batch_id": "abc-123"})),
    };
    let resp = internal_post(&ingest_url(), "/internal/connectors/bulk", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn ingest_bulk_rejects_missing_tenant() {
    let resp = client()
        .post(format!("{}/internal/connectors/bulk", ingest_url()))
        .json(&serde_json::json!({"texts": ["test"]}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
