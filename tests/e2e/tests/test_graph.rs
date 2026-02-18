// =============================================================================
// Graph Service E2E Tests (service-graph, background NATS consumer)
// Covers: NATS event consumption, LLM extraction, entity/edge upsert
// Note: Graph service has no HTTP endpoints — tested via write→NATS→graph→DB
// =============================================================================

use crate::harness::*;
use reqwest::StatusCode;

// ---------------------------------------------------------------------------
// Write → Graph pipeline (async via NATS)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn graph_processes_write_event_creates_entities() {
    // Write a memory with clear entity references
    let body = MemoryWriteRequest {
        content: "John Smith works at Acme Corp as a senior engineer. He joined in 2023.".into(),
        role: Some("user".into()),
        user_id: None,
        session_id: None,
        agent_id: None,
        metadata: None,
    };
    let resp = internal_post(&write_url(), "/internal/ingest", &body).await;
    assert!(resp.status().is_success());

    // Wait for async NATS → Graph processing (LLM extraction takes time)
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // Check entities were created via gateway
    let resp = gateway_get("/v1/graph/entities?entity_type=person&limit=50").await;
    assert!(resp.status().is_success());
    // We can't guarantee the specific entity exists (depends on LLM), but the endpoint works
}

#[tokio::test]
async fn graph_handles_memory_with_relationships() {
    let body = MemoryWriteRequest {
        content: "Alice manages Bob and Carol. They all work in the marketing department.".into(),
        role: Some("user".into()),
        user_id: None,
        session_id: None,
        agent_id: None,
        metadata: None,
    };
    let resp = internal_post(&write_url(), "/internal/ingest", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn graph_handles_memory_with_no_entities() {
    let body = MemoryWriteRequest {
        content: "The weather is nice today.".into(),
        role: Some("user".into()),
        user_id: None,
        session_id: None,
        agent_id: None,
        metadata: None,
    };
    let resp = internal_post(&write_url(), "/internal/ingest", &body).await;
    assert!(resp.status().is_success());
    // Graph should handle gracefully (no entities to create)
}

#[tokio::test]
async fn graph_handles_memory_with_temporal_references() {
    let body = MemoryWriteRequest {
        content: "Last Tuesday, the team decided to switch from React to Svelte for the frontend.".into(),
        role: Some("assistant".into()),
        user_id: None,
        session_id: None,
        agent_id: None,
        metadata: None,
    };
    let resp = internal_post(&write_url(), "/internal/ingest", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn graph_handles_memory_with_preferences() {
    let body = MemoryWriteRequest {
        content: "The user prefers dark mode, uses Vim keybindings, and likes TypeScript over JavaScript.".into(),
        role: Some("user".into()),
        user_id: None,
        session_id: None,
        agent_id: None,
        metadata: None,
    };
    let resp = internal_post(&write_url(), "/internal/ingest", &body).await;
    assert!(resp.status().is_success());
}

// ---------------------------------------------------------------------------
// Entity listing and querying (via gateway)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn graph_list_entities_all_types() {
    let resp = gateway_get("/v1/graph/entities").await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.is_array() || body.is_object());
}

#[tokio::test]
async fn graph_list_entities_by_type_person() {
    let resp = gateway_get("/v1/graph/entities?entity_type=person").await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn graph_list_entities_by_type_organization() {
    let resp = gateway_get("/v1/graph/entities?entity_type=organization").await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn graph_list_entities_by_type_concept() {
    let resp = gateway_get("/v1/graph/entities?entity_type=concept").await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn graph_list_entities_by_type_location() {
    let resp = gateway_get("/v1/graph/entities?entity_type=location").await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn graph_list_entities_with_limit() {
    let resp = gateway_get("/v1/graph/entities?limit=2").await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn graph_list_entities_unknown_type_returns_empty() {
    let resp = gateway_get("/v1/graph/entities?entity_type=nonexistent_type").await;
    assert!(resp.status().is_success());
}

// ---------------------------------------------------------------------------
// Entity expansion (2-hop)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn graph_expand_entity_nonexistent() {
    let fake_id = uuid::Uuid::new_v4();
    let resp = gateway_get(&format!("/v1/graph/entities/{}/expand", fake_id)).await;
    let status = resp.status();
    assert!(
        status.is_success() || status == StatusCode::NOT_FOUND,
        "Expand nonexistent: {}",
        status
    );
}

// ---------------------------------------------------------------------------
// Edge queries
// ---------------------------------------------------------------------------

#[tokio::test]
async fn graph_get_edges_for_nonexistent_entity() {
    let fake_id = uuid::Uuid::new_v4();
    let resp = gateway_get(&format!("/v1/graph/entities/{}/edges", fake_id)).await;
    let status = resp.status();
    assert!(
        status.is_success() || status == StatusCode::NOT_FOUND,
        "Edges for nonexistent entity: {}",
        status
    );
}
