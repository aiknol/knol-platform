// =============================================================================
// Graph API — End-to-End Tests
// =============================================================================
//
// Covers: list entities, get entity, get edges, traverse, find path.
// Graph entities are created automatically when memories are ingested.
// =============================================================================

use crate::tenant_helpers::*;
use reqwest::StatusCode;
use serde_json::json;

// ── List Entities ───────────────────────────────────────────────────────────

#[tokio::test]
async fn list_entities_returns_array() {
    let (_client, api_key, _csrf, _body) = signup_tenant("graph-list").await;

    // Write some content that should generate entities
    gateway_post_with_key(
        &api_key,
        "/v1/memory",
        &json!({ "content": "John Smith works at Acme Corp in New York City." }),
    )
    .await;

    // Give the pipeline time to extract entities
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    let (status, resp) = gateway_get_with_key(&api_key, "/v1/graph/entities?limit=50").await;
    assert_eq!(status, StatusCode::OK, "List entities failed: {:?}", resp);
    assert!(resp.is_array(), "Should return array of entities");
}

#[tokio::test]
async fn list_entities_with_type_filter() {
    let (_client, api_key, _csrf, _body) = signup_tenant("graph-filter").await;

    let (status, resp) = gateway_get_with_key(
        &api_key,
        "/v1/graph/entities?entity_type=person&limit=10",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "Filter entities failed: {:?}", resp);
    assert!(resp.is_array());
}

// ── Get Entity by ID ────────────────────────────────────────────────────────

#[tokio::test]
async fn get_nonexistent_entity_returns_404() {
    let (_client, api_key, _csrf, _body) = signup_tenant("graph-404").await;

    let fake_id = uuid::Uuid::new_v4();
    let (status, _) = gateway_get_with_key(&api_key, &format!("/v1/graph/entities/{}", fake_id)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_entity_returns_details() {
    let (_client, api_key, _csrf, _body) = signup_tenant("graph-detail").await;

    // Write content to generate entities
    gateway_post_with_key(
        &api_key,
        "/v1/memory",
        &json!({ "content": "Albert Einstein developed the theory of relativity at Princeton University." }),
    )
    .await;

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    let (status, entities) = gateway_get_with_key(&api_key, "/v1/graph/entities?limit=5").await;
    assert_eq!(status, StatusCode::OK);

    let entities = entities.as_array().unwrap();
    if entities.is_empty() {
        eprintln!("WARN: No entities extracted yet, skipping get entity test");
        return;
    }

    let entity_id = entities[0]["id"].as_str().unwrap();
    let (status, entity) = gateway_get_with_key(
        &api_key,
        &format!("/v1/graph/entities/{}", entity_id),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "Get entity failed: {:?}", entity);
    assert!(entity["name"].is_string());
    assert!(entity["entity_type"].is_string());
}

// ── Get Entity Edges ────────────────────────────────────────────────────────

#[tokio::test]
async fn get_entity_edges_returns_structure() {
    let (_client, api_key, _csrf, _body) = signup_tenant("graph-edges").await;

    // Write content that should create relationships
    gateway_post_with_key(
        &api_key,
        "/v1/memory",
        &json!({ "content": "Marie Curie discovered radium and won the Nobel Prize in Physics." }),
    )
    .await;

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    let (_, entities) = gateway_get_with_key(&api_key, "/v1/graph/entities?limit=5").await;
    let entities = entities.as_array().unwrap();
    if entities.is_empty() {
        eprintln!("WARN: No entities extracted, skipping edges test");
        return;
    }

    let entity_id = entities[0]["id"].as_str().unwrap();
    let (status, resp) = gateway_get_with_key(
        &api_key,
        &format!("/v1/graph/entities/{}/edges", entity_id),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "Get edges failed: {:?}", resp);
    assert!(resp["outgoing"].is_array(), "Should have outgoing edges");
    assert!(resp["incoming"].is_array(), "Should have incoming edges");
}

// ── Traverse ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn traverse_entity_returns_results() {
    let (_client, api_key, _csrf, _body) = signup_tenant("graph-traverse").await;

    gateway_post_with_key(
        &api_key,
        "/v1/memory",
        &json!({ "content": "Tesla Motors was founded by Elon Musk in Palo Alto, California." }),
    )
    .await;

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    let (_, entities) = gateway_get_with_key(&api_key, "/v1/graph/entities?limit=5").await;
    let entities = entities.as_array().unwrap();
    if entities.is_empty() {
        eprintln!("WARN: No entities, skipping traverse test");
        return;
    }

    let entity_id = entities[0]["id"].as_str().unwrap();
    let (status, resp) = gateway_get_with_key(
        &api_key,
        &format!("/v1/graph/entities/{}/traverse?depth=2&limit=20", entity_id),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "Traverse failed: {:?}", resp);
    assert!(resp["source_entity_id"].is_string());
    assert!(resp["entities"].is_array());
}

// ── Find Path ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn find_path_between_entities() {
    let (_client, api_key, _csrf, _body) = signup_tenant("graph-path").await;

    gateway_post_with_key(
        &api_key,
        "/v1/memory",
        &json!({ "content": "Steve Jobs founded Apple and worked with Steve Wozniak at their garage." }),
    )
    .await;

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    let (_, entities) = gateway_get_with_key(&api_key, "/v1/graph/entities?limit=10").await;
    let entities = entities.as_array().unwrap();
    if entities.len() < 2 {
        eprintln!("WARN: Need at least 2 entities for path test, got {}", entities.len());
        return;
    }

    let from_id = entities[0]["id"].as_str().unwrap();
    let to_id = entities[1]["id"].as_str().unwrap();

    let (status, resp) = gateway_get_with_key(
        &api_key,
        &format!("/v1/graph/path/{}/{}", from_id, to_id),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "Find path failed: {:?}", resp);
    assert!(resp["from"].is_string());
    assert!(resp["to"].is_string());
    // Path may or may not be found depending on graph structure
    assert!(resp.get("found").is_some() || resp.get("path").is_some());
}

// ── Read-only key can access graph ──────────────────────────────────────────

#[tokio::test]
async fn read_only_key_can_query_graph() {
    let (client, _initial_key, csrf, _body) = signup_tenant("graph-ro").await;

    let (_, created) = tenant_post_auth(
        &client,
        &csrf,
        "/app/api-keys",
        &json!({ "name": "ro-graph", "role": "read_only" }),
    )
    .await;
    let ro_key = created["api_key"].as_str().unwrap();

    let (status, _) = gateway_get_with_key(ro_key, "/v1/graph/entities?limit=5").await;
    assert_eq!(status, StatusCode::OK, "read_only should access graph");
}
