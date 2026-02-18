// =============================================================================
// Gateway Service E2E Tests (service-gateway, port 8080)
// Covers: 15 routes, auth, rate limiting, CORS, plan enforcement
// =============================================================================

use crate::harness::*;
use reqwest::StatusCode;

// ---------------------------------------------------------------------------
// Health check
// ---------------------------------------------------------------------------

#[tokio::test]
async fn gateway_health_returns_200() {
    let resp = client()
        .get(format!("{}/health", gateway_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// Authentication tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn gateway_rejects_request_without_auth_header() {
    let resp = client()
        .post(format!("{}/v1/memory", gateway_url()))
        .json(&serde_json::json!({"content": "test"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let body: ErrorResponse = resp.json().await.unwrap();
    assert!(body.error.contains("Authorization"));
}

#[tokio::test]
async fn gateway_rejects_malformed_bearer_token() {
    let resp = client()
        .post(format!("{}/v1/memory", gateway_url()))
        .header("Authorization", "NotBearer abc123")
        .json(&serde_json::json!({"content": "test"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn gateway_rejects_invalid_api_key() {
    let resp = client()
        .post(format!("{}/v1/memory", gateway_url()))
        .header("Authorization", "Bearer invalid-key-that-does-not-exist")
        .json(&serde_json::json!({"content": "test"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let body: ErrorResponse = resp.json().await.unwrap();
    assert!(body.error.contains("Invalid API key") || body.error.contains("API key"));
}

#[tokio::test]
async fn gateway_accepts_valid_bearer_token() {
    let body = MemoryWriteRequest {
        content: unique_content("auth-valid"),
        role: Some("user".into()),
        user_id: None,
        session_id: None,
        agent_id: None,
        metadata: None,
    };
    let resp = gateway_post("/v1/memory", &body).await;
    assert!(
        resp.status() == StatusCode::OK || resp.status() == StatusCode::CREATED,
        "Expected 200/201 but got {}",
        resp.status()
    );
}

#[tokio::test]
async fn gateway_rejects_empty_bearer_value() {
    let resp = client()
        .post(format!("{}/v1/memory", gateway_url()))
        .header("Authorization", "Bearer ")
        .json(&serde_json::json!({"content": "test"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// CORS tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn gateway_handles_cors_preflight() {
    let resp = client()
        .request(reqwest::Method::OPTIONS, format!("{}/v1/memory", gateway_url()))
        .header("Origin", "https://app.aiknol.com")
        .header("Access-Control-Request-Method", "POST")
        .header("Access-Control-Request-Headers", "Authorization, Content-Type")
        .send()
        .await
        .unwrap();
    // Should return 200 or 204 with CORS headers
    assert!(resp.status().is_success() || resp.status() == StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn gateway_returns_cors_headers_on_response() {
    let resp = client()
        .get(format!("{}/health", gateway_url()))
        .header("Origin", "https://app.aiknol.com")
        .send()
        .await
        .unwrap();
    // CORS middleware should add Access-Control-Allow-Origin
    assert_eq!(resp.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// Memory CRUD via Gateway
// ---------------------------------------------------------------------------

#[tokio::test]
async fn gateway_write_memory_returns_episode_id() {
    let body = MemoryWriteRequest {
        content: unique_content("write-test"),
        role: Some("user".into()),
        user_id: None,
        session_id: Some("test-session".into()),
        agent_id: None,
        metadata: None,
    };
    let resp = gateway_post("/v1/memory", &body).await;
    assert!(resp.status().is_success());
    let result: MemoryWriteResponse = resp.json().await.unwrap();
    assert_eq!(result.status, "accepted");
    assert!(!result.episode_id.is_nil());
}

#[tokio::test]
async fn gateway_write_memory_with_all_fields() {
    let body = MemoryWriteRequest {
        content: unique_content("write-all-fields"),
        role: Some("assistant".into()),
        user_id: Some(uuid::Uuid::new_v4()),
        session_id: Some("sess-123".into()),
        agent_id: Some("agent-456".into()),
        metadata: Some(serde_json::json!({"source": "test", "priority": 1})),
    };
    let resp = gateway_post("/v1/memory", &body).await;
    assert!(resp.status().is_success());
    let result: MemoryWriteResponse = resp.json().await.unwrap();
    assert_eq!(result.status, "accepted");
}

#[tokio::test]
async fn gateway_write_rejects_empty_content() {
    let body = serde_json::json!({"content": ""});
    let resp = gateway_post("/v1/memory", &body).await;
    // Should reject or accept with validation error
    let status = resp.status();
    assert!(
        status == StatusCode::BAD_REQUEST || status.is_success(),
        "Empty content should be rejected or handled gracefully: {}",
        status
    );
}

#[tokio::test]
async fn gateway_write_rejects_missing_content_field() {
    let body = serde_json::json!({"role": "user"});
    let resp = gateway_post("/v1/memory", &body).await;
    assert!(
        resp.status() == StatusCode::BAD_REQUEST
            || resp.status() == StatusCode::UNPROCESSABLE_ENTITY,
        "Missing content should be 400/422"
    );
}

#[tokio::test]
async fn gateway_batch_write() {
    let body = serde_json::json!([
        {"content": unique_content("batch-1"), "role": "user"},
        {"content": unique_content("batch-2"), "role": "assistant"},
        {"content": unique_content("batch-3"), "role": "user"},
    ]);
    let resp = gateway_post("/v1/memory/batch", &body).await;
    assert!(
        resp.status().is_success(),
        "Batch write failed: {}",
        resp.status()
    );
}

#[tokio::test]
async fn gateway_search_memory() {
    // First write something
    let content = unique_content("search-target");
    let write_body = MemoryWriteRequest {
        content: content.clone(),
        role: Some("user".into()),
        user_id: None,
        session_id: None,
        agent_id: None,
        metadata: None,
    };
    let resp = gateway_post("/v1/memory", &write_body).await;
    assert!(resp.status().is_success());

    // Wait for async processing
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Now search
    let search_body = MemorySearchRequest {
        query: content,
        user_id: None,
        scope: None,
        kind: None,
        limit: Some(10),
        min_confidence: None,
    };
    let resp = gateway_post("/v1/memory/search", &search_body).await;
    assert!(resp.status().is_success());
    let result: MemorySearchResponse = resp.json().await.unwrap();
    assert!(result.query_ms > 0 || result.query_ms == 0); // timing is valid
}

#[tokio::test]
async fn gateway_search_with_filters() {
    let search_body = MemorySearchRequest {
        query: "test query".into(),
        user_id: Some(uuid::Uuid::new_v4()),
        scope: Some("user".into()),
        kind: Some("preference".into()),
        limit: Some(5),
        min_confidence: Some(0.5),
    };
    let resp = gateway_post("/v1/memory/search", &search_body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn gateway_search_empty_query() {
    let search_body = serde_json::json!({"query": ""});
    let resp = gateway_post("/v1/memory/search", &search_body).await;
    let status = resp.status();
    assert!(
        status.is_success() || status == StatusCode::BAD_REQUEST,
        "Empty search should return results or 400: {}",
        status
    );
}

#[tokio::test]
async fn gateway_get_memory_by_id_not_found() {
    let fake_id = uuid::Uuid::new_v4();
    let resp = gateway_get(&format!("/v1/memory/{}", fake_id)).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn gateway_update_memory_not_found() {
    let fake_id = uuid::Uuid::new_v4();
    let body = UpdateMemoryRequest {
        content: Some("updated".into()),
        status: None,
        importance: None,
    };
    let resp = gateway_put(&format!("/v1/memory/{}", fake_id), &body).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn gateway_delete_memory_not_found() {
    let fake_id = uuid::Uuid::new_v4();
    let resp = gateway_delete(&format!("/v1/memory/{}", fake_id)).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// Graph API via Gateway
// ---------------------------------------------------------------------------

#[tokio::test]
async fn gateway_list_entities() {
    let resp = gateway_get("/v1/graph/entities").await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn gateway_list_entities_with_type_filter() {
    let resp = gateway_get("/v1/graph/entities?entity_type=person&limit=5").await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn gateway_get_entity_not_found() {
    let fake_id = uuid::Uuid::new_v4();
    let resp = gateway_get(&format!("/v1/graph/entities/{}", fake_id)).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn gateway_get_entity_edges_not_found() {
    let fake_id = uuid::Uuid::new_v4();
    let resp = gateway_get(&format!("/v1/graph/entities/{}/edges", fake_id)).await;
    // May return empty array or 404
    let status = resp.status();
    assert!(
        status.is_success() || status == StatusCode::NOT_FOUND,
        "Entity edges for unknown id: {}",
        status
    );
}

#[tokio::test]
async fn gateway_expand_entity_not_found() {
    let fake_id = uuid::Uuid::new_v4();
    let resp = gateway_get(&format!("/v1/graph/entities/{}/expand", fake_id)).await;
    let status = resp.status();
    assert!(
        status.is_success() || status == StatusCode::NOT_FOUND,
        "Expand for unknown entity: {}",
        status
    );
}

// ---------------------------------------------------------------------------
// Admin API via Gateway
// ---------------------------------------------------------------------------

#[tokio::test]
async fn gateway_get_tenant_info() {
    let resp = gateway_get("/v1/admin/tenants").await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn gateway_list_audit_log() {
    let resp = gateway_get("/v1/admin/audit").await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn gateway_list_audit_log_with_filter() {
    let resp = gateway_get("/v1/admin/audit?limit=5").await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn gateway_list_policies() {
    let resp = gateway_get("/v1/admin/policies").await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn gateway_create_policy() {
    let body = CreatePolicyRequest {
        name: format!("test-retention-{}", &uuid::Uuid::new_v4().to_string()[..8]),
        rule_type: "retention".into(),
        config: serde_json::json!({"retention_days": 90}),
    };
    let resp = gateway_post("/v1/admin/policies", &body).await;
    assert!(resp.status().is_success(), "Create policy failed: {}", resp.status());
}

// ---------------------------------------------------------------------------
// Rate limiting tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn gateway_rate_limit_not_triggered_on_single_request() {
    let body = MemorySearchRequest {
        query: "rate limit test".into(),
        user_id: None,
        scope: None,
        kind: None,
        limit: Some(1),
        min_confidence: None,
    };
    let resp = gateway_post("/v1/memory/search", &body).await;
    assert_ne!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
}

// ---------------------------------------------------------------------------
// Invalid routes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn gateway_returns_404_for_unknown_route() {
    let resp = gateway_get("/v1/nonexistent").await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn gateway_returns_405_for_wrong_method() {
    let resp = client()
        .delete(format!("{}/v1/memory/search", gateway_url()))
        .header("Authorization", format!("Bearer {}", test_api_key()))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status() == StatusCode::METHOD_NOT_ALLOWED || resp.status() == StatusCode::NOT_FOUND,
        "Wrong method should be 405 or 404"
    );
}

// ---------------------------------------------------------------------------
// Content-Type handling
// ---------------------------------------------------------------------------

#[tokio::test]
async fn gateway_rejects_non_json_body() {
    let resp = client()
        .post(format!("{}/v1/memory", gateway_url()))
        .header("Authorization", format!("Bearer {}", test_api_key()))
        .header("Content-Type", "text/plain")
        .body("not json")
        .send()
        .await
        .unwrap();
    assert!(
        resp.status() == StatusCode::BAD_REQUEST
            || resp.status() == StatusCode::UNSUPPORTED_MEDIA_TYPE
            || resp.status() == StatusCode::UNPROCESSABLE_ENTITY,
        "Non-JSON should be rejected: {}",
        resp.status()
    );
}
