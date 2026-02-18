//! Unit tests for ingest service types and payload validation
//!
//! Tests connector info, webhook payloads, and bulk ingestion request handling.

use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

// ── Replicated types for testing (mirrors main.rs) ──

#[derive(Debug, Serialize, Deserialize)]
struct ConnectorInfo {
    id: String,
    name: String,
    description: String,
    status: String,
}

#[derive(Debug, Deserialize)]
struct WebhookPayload {
    source: String,
    items: Vec<WebhookItem>,
}

#[derive(Debug, Deserialize)]
struct WebhookItem {
    content: String,
    user_id: Option<Uuid>,
    role: Option<String>,
    session_id: Option<String>,
    agent_id: Option<String>,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct BulkIngestRequest {
    texts: Vec<String>,
    user_id: Option<Uuid>,
    session_id: Option<String>,
    agent_id: Option<String>,
    metadata: Option<serde_json::Value>,
}

// ── Connector Info Tests ──

#[test]
fn test_connectors_list() {
    let connectors = vec![
        ConnectorInfo {
            id: "webhook".into(),
            name: "Webhook".into(),
            description: "Generic webhook connector".into(),
            status: "available".into(),
        },
        ConnectorInfo {
            id: "slack".into(),
            name: "Slack".into(),
            description: "Slack channel connector".into(),
            status: "coming_soon".into(),
        },
        ConnectorInfo {
            id: "github".into(),
            name: "GitHub".into(),
            description: "GitHub issues/PRs connector".into(),
            status: "coming_soon".into(),
        },
        ConnectorInfo {
            id: "email".into(),
            name: "Email".into(),
            description: "Email IMAP connector".into(),
            status: "coming_soon".into(),
        },
    ];

    assert_eq!(connectors.len(), 4);
    assert_eq!(connectors.iter().filter(|c| c.status == "available").count(), 1);
    assert_eq!(connectors.iter().filter(|c| c.status == "coming_soon").count(), 3);
}

#[test]
fn test_connector_info_serialization() {
    let connector = ConnectorInfo {
        id: "webhook".into(),
        name: "Webhook".into(),
        description: "Generic webhook".into(),
        status: "available".into(),
    };

    let json = serde_json::to_string(&connector).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["id"], "webhook");
    assert_eq!(parsed["status"], "available");
}

// ── Webhook Payload Tests ──

#[test]
fn test_webhook_payload_single_item() {
    let json = serde_json::json!({
        "source": "custom-crm",
        "items": [{
            "content": "Customer John called about order #1234",
            "user_id": Uuid::new_v4(),
            "role": "user",
        }],
    });

    let payload: WebhookPayload = serde_json::from_value(json).unwrap();
    assert_eq!(payload.source, "custom-crm");
    assert_eq!(payload.items.len(), 1);
    assert_eq!(payload.items[0].role.as_deref().unwrap(), "user");
}

#[test]
fn test_webhook_payload_multiple_items() {
    let user_id = Uuid::new_v4();
    let json = serde_json::json!({
        "source": "slack-bot",
        "items": [
            { "content": "First message in conversation", "user_id": user_id },
            { "content": "Second message about the project", "user_id": user_id },
            { "content": "Third message with action items", "user_id": user_id },
        ],
    });

    let payload: WebhookPayload = serde_json::from_value(json).unwrap();
    assert_eq!(payload.items.len(), 3);
    // All items should have same user_id
    for item in &payload.items {
        assert_eq!(item.user_id.unwrap(), user_id);
    }
}

#[test]
fn test_webhook_payload_minimal_item() {
    let json = serde_json::json!({
        "source": "test",
        "items": [{
            "content": "minimal content",
        }],
    });

    let payload: WebhookPayload = serde_json::from_value(json).unwrap();
    let item = &payload.items[0];
    assert!(item.user_id.is_none());
    assert!(item.role.is_none());
    assert!(item.session_id.is_none());
    assert!(item.agent_id.is_none());
    assert!(item.metadata.is_none());
}

#[test]
fn test_webhook_payload_with_metadata() {
    let json = serde_json::json!({
        "source": "integration",
        "items": [{
            "content": "Meeting notes from standup",
            "metadata": {
                "channel": "#engineering",
                "thread_ts": "1234567890.123456",
                "tags": ["standup", "engineering"],
            },
        }],
    });

    let payload: WebhookPayload = serde_json::from_value(json).unwrap();
    let meta = payload.items[0].metadata.as_ref().unwrap();
    assert_eq!(meta["channel"], "#engineering");
    assert_eq!(meta["tags"].as_array().unwrap().len(), 2);
}

#[test]
fn test_webhook_role_defaults() {
    // When role is None, handler defaults to "system"
    let json = serde_json::json!({
        "source": "test",
        "items": [{ "content": "no role specified" }],
    });

    let payload: WebhookPayload = serde_json::from_value(json).unwrap();
    let role = payload.items[0].role.clone().unwrap_or_else(|| "system".into());
    assert_eq!(role, "system");
}

// ── Bulk Ingest Tests ──

#[test]
fn test_bulk_ingest_minimal() {
    let json = serde_json::json!({
        "texts": ["Document paragraph 1", "Document paragraph 2"],
    });

    let req: BulkIngestRequest = serde_json::from_value(json).unwrap();
    assert_eq!(req.texts.len(), 2);
    assert!(req.user_id.is_none());
    assert!(req.session_id.is_none());
}

#[test]
fn test_bulk_ingest_full() {
    let user_id = Uuid::new_v4();
    let json = serde_json::json!({
        "texts": ["Chapter 1 content", "Chapter 2 content", "Chapter 3 content"],
        "user_id": user_id,
        "session_id": "import-session-001",
        "agent_id": "doc-importer",
        "metadata": { "source_file": "book.pdf", "format": "pdf" },
    });

    let req: BulkIngestRequest = serde_json::from_value(json).unwrap();
    assert_eq!(req.texts.len(), 3);
    assert_eq!(req.user_id.unwrap(), user_id);
    assert_eq!(req.session_id.unwrap(), "import-session-001");
    assert_eq!(req.agent_id.unwrap(), "doc-importer");
    assert_eq!(req.metadata.unwrap()["source_file"], "book.pdf");
}

#[test]
fn test_bulk_ingest_empty_texts() {
    let json = serde_json::json!({
        "texts": [],
    });

    let req: BulkIngestRequest = serde_json::from_value(json).unwrap();
    assert!(req.texts.is_empty());
}

#[test]
fn test_bulk_ingest_large_batch() {
    let texts: Vec<String> = (0..100).map(|i| format!("Document chunk {}", i)).collect();
    let json = serde_json::json!({
        "texts": texts,
        "user_id": Uuid::new_v4(),
    });

    let req: BulkIngestRequest = serde_json::from_value(json).unwrap();
    assert_eq!(req.texts.len(), 100);
}

// ── Event Construction Tests ──

#[test]
fn test_write_event_construction_from_webhook() {
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // Simulate what webhook_ingest does
    let event = serde_json::json!({
        "episode_id": Uuid::new_v4(),
        "tenant_id": tenant_id,
        "user_id": user_id,
        "content": "User mentioned they prefer dark mode",
        "role": "user",
        "session_id": "session-123",
        "agent_id": "assistant-v2",
        "metadata": {},
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    assert_eq!(event["tenant_id"], tenant_id.to_string());
    assert_eq!(event["role"], "user");
}

#[test]
fn test_ingestion_count_tracking() {
    // Simulate the ingestion counting logic
    let items = vec!["item1", "item2", "item3", "item4", "item5"];
    let mut ingested = 0;
    let mut failed = 0;

    for (i, _item) in items.iter().enumerate() {
        // Simulate: 4 succeed, 1 fails
        if i == 2 {
            failed += 1;
        } else {
            ingested += 1;
        }
    }

    assert_eq!(ingested, 4);
    assert_eq!(failed, 1);

    let response = serde_json::json!({
        "ingested": ingested,
        "total": items.len(),
        "source": "test",
    });

    assert_eq!(response["ingested"], 4);
    assert_eq!(response["total"], 5);
}
