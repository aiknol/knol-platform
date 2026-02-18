// =============================================================================
// Retrieve Service E2E Tests (service-retrieve, port 8082)
// Covers: /internal/search, /health, intent classification, RRF fusion,
//         scope cascade, temporal filters, vector+BM25+graph search
// =============================================================================

use crate::harness::*;
use reqwest::StatusCode;

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

#[tokio::test]
async fn retrieve_health_returns_200() {
    let resp = client()
        .get(format!("{}/health", retrieve_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// Basic search
// ---------------------------------------------------------------------------

#[tokio::test]
async fn retrieve_search_returns_valid_response() {
    let body = MemorySearchRequest {
        query: "test search query".into(),
        user_id: None,
        scope: None,
        kind: None,
        limit: Some(10),
        min_confidence: None,
    };
    let resp = internal_post(&retrieve_url(), "/internal/search", &body).await;
    assert!(resp.status().is_success());
    let result: MemorySearchResponse = resp.json().await.unwrap();
    assert!(result.total >= 0); // can be zero if no matching memories
}

#[tokio::test]
async fn retrieve_search_respects_limit() {
    let body = MemorySearchRequest {
        query: "test".into(),
        user_id: None,
        scope: None,
        kind: None,
        limit: Some(3),
        min_confidence: None,
    };
    let resp = internal_post(&retrieve_url(), "/internal/search", &body).await;
    assert!(resp.status().is_success());
    let result: MemorySearchResponse = resp.json().await.unwrap();
    assert!(result.results.len() <= 3);
}

#[tokio::test]
async fn retrieve_search_rejects_missing_tenant() {
    let resp = client()
        .post(format!("{}/internal/search", retrieve_url()))
        .json(&serde_json::json!({"query": "test"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// Intent-based search routing
// ---------------------------------------------------------------------------

#[tokio::test]
async fn retrieve_search_preference_intent() {
    // Keywords: "prefer", "like", "favorite" → vector-first
    let body = MemorySearchRequest {
        query: "What does the user prefer for breakfast?".into(),
        user_id: None, scope: None, kind: None, limit: Some(5), min_confidence: None,
    };
    let resp = internal_post(&retrieve_url(), "/internal/search", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn retrieve_search_temporal_intent() {
    // Keywords: "when", "timeline", "history", "recently" → text+graph
    let body = MemorySearchRequest {
        query: "When was the last time we discussed the timeline?".into(),
        user_id: None, scope: None, kind: None, limit: Some(5), min_confidence: None,
    };
    let resp = internal_post(&retrieve_url(), "/internal/search", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn retrieve_search_relational_intent() {
    // Keywords: "who", "relationship", "connected" → graph-first
    let body = MemorySearchRequest {
        query: "Who is connected to the engineering team?".into(),
        user_id: None, scope: None, kind: None, limit: Some(5), min_confidence: None,
    };
    let resp = internal_post(&retrieve_url(), "/internal/search", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn retrieve_search_general_intent() {
    // No intent keywords → hybrid
    let body = MemorySearchRequest {
        query: "Information about the project status".into(),
        user_id: None, scope: None, kind: None, limit: Some(5), min_confidence: None,
    };
    let resp = internal_post(&retrieve_url(), "/internal/search", &body).await;
    assert!(resp.status().is_success());
}

// ---------------------------------------------------------------------------
// Scope filtering
// ---------------------------------------------------------------------------

#[tokio::test]
async fn retrieve_search_scope_user() {
    let body = MemorySearchRequest {
        query: "test".into(),
        user_id: Some(uuid::Uuid::new_v4()),
        scope: Some("user".into()),
        kind: None, limit: Some(5), min_confidence: None,
    };
    let resp = internal_post(&retrieve_url(), "/internal/search", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn retrieve_search_scope_team() {
    let body = MemorySearchRequest {
        query: "test".into(),
        user_id: None,
        scope: Some("team".into()),
        kind: None, limit: Some(5), min_confidence: None,
    };
    let resp = internal_post(&retrieve_url(), "/internal/search", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn retrieve_search_scope_org() {
    let body = MemorySearchRequest {
        query: "test".into(),
        user_id: None,
        scope: Some("org".into()),
        kind: None, limit: Some(5), min_confidence: None,
    };
    let resp = internal_post(&retrieve_url(), "/internal/search", &body).await;
    assert!(resp.status().is_success());
}

// ---------------------------------------------------------------------------
// Kind filtering
// ---------------------------------------------------------------------------

#[tokio::test]
async fn retrieve_search_kind_preference() {
    let body = MemorySearchRequest {
        query: "test".into(),
        user_id: None, scope: None,
        kind: Some("preference".into()),
        limit: Some(5), min_confidence: None,
    };
    let resp = internal_post(&retrieve_url(), "/internal/search", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn retrieve_search_kind_fact() {
    let body = MemorySearchRequest {
        query: "test".into(),
        user_id: None, scope: None,
        kind: Some("fact".into()),
        limit: Some(5), min_confidence: None,
    };
    let resp = internal_post(&retrieve_url(), "/internal/search", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn retrieve_search_kind_event() {
    let body = MemorySearchRequest {
        query: "test".into(),
        user_id: None, scope: None,
        kind: Some("event".into()),
        limit: Some(5), min_confidence: None,
    };
    let resp = internal_post(&retrieve_url(), "/internal/search", &body).await;
    assert!(resp.status().is_success());
}

// ---------------------------------------------------------------------------
// Confidence threshold
// ---------------------------------------------------------------------------

#[tokio::test]
async fn retrieve_search_min_confidence_high() {
    let body = MemorySearchRequest {
        query: "test".into(),
        user_id: None, scope: None, kind: None,
        limit: Some(10),
        min_confidence: Some(0.95),
    };
    let resp = internal_post(&retrieve_url(), "/internal/search", &body).await;
    assert!(resp.status().is_success());
    let result: MemorySearchResponse = resp.json().await.unwrap();
    // High threshold means fewer or no results
    assert!(result.results.len() <= 10);
}

#[tokio::test]
async fn retrieve_search_min_confidence_zero() {
    let body = MemorySearchRequest {
        query: "test".into(),
        user_id: None, scope: None, kind: None,
        limit: Some(10),
        min_confidence: Some(0.0),
    };
    let resp = internal_post(&retrieve_url(), "/internal/search", &body).await;
    assert!(resp.status().is_success());
}

// ---------------------------------------------------------------------------
// Response structure
// ---------------------------------------------------------------------------

#[tokio::test]
async fn retrieve_search_response_has_query_ms() {
    let body = MemorySearchRequest {
        query: "performance measurement".into(),
        user_id: None, scope: None, kind: None, limit: Some(5), min_confidence: None,
    };
    let resp = internal_post(&retrieve_url(), "/internal/search", &body).await;
    assert!(resp.status().is_success());
    let result: MemorySearchResponse = resp.json().await.unwrap();
    // query_ms should be a non-negative number
    assert!(result.query_ms < 30_000, "Search took too long: {}ms", result.query_ms);
}

#[tokio::test]
async fn retrieve_search_results_sorted_by_score() {
    let body = MemorySearchRequest {
        query: "common word test".into(),
        user_id: None, scope: None, kind: None, limit: Some(20), min_confidence: None,
    };
    let resp = internal_post(&retrieve_url(), "/internal/search", &body).await;
    assert!(resp.status().is_success());
    let result: MemorySearchResponse = resp.json().await.unwrap();
    // Verify descending score order
    for i in 1..result.results.len() {
        assert!(
            result.results[i - 1].score >= result.results[i].score,
            "Results not sorted by score: {} < {} at index {}",
            result.results[i - 1].score, result.results[i].score, i
        );
    }
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[tokio::test]
async fn retrieve_search_very_long_query() {
    let long_query = "test ".repeat(500);
    let body = MemorySearchRequest {
        query: long_query,
        user_id: None, scope: None, kind: None, limit: Some(5), min_confidence: None,
    };
    let resp = internal_post(&retrieve_url(), "/internal/search", &body).await;
    assert!(
        resp.status().is_success() || resp.status() == StatusCode::BAD_REQUEST,
        "Long query: {}",
        resp.status()
    );
}

#[tokio::test]
async fn retrieve_search_sql_injection_attempt() {
    let body = MemorySearchRequest {
        query: "'; DROP TABLE memories; --".into(),
        user_id: None, scope: None, kind: None, limit: Some(5), min_confidence: None,
    };
    let resp = internal_post(&retrieve_url(), "/internal/search", &body).await;
    // Should handle safely — either return results or empty, never crash
    assert!(
        resp.status().is_success() || resp.status() == StatusCode::BAD_REQUEST,
        "SQL injection attempt: {}",
        resp.status()
    );
}
