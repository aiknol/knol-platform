// =============================================================================
// Test Harness — shared configuration, HTTP client, types, and helpers
// =============================================================================

use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Service URLs
// ---------------------------------------------------------------------------

pub fn gateway_url() -> String {
    env::var("GATEWAY_URL").unwrap_or_else(|_| "http://localhost:8080".into())
}

pub fn write_url() -> String {
    env::var("WRITE_URL").unwrap_or_else(|_| "http://localhost:8081".into())
}

pub fn retrieve_url() -> String {
    env::var("RETRIEVE_URL").unwrap_or_else(|_| "http://localhost:8082".into())
}

pub fn admin_url() -> String {
    env::var("ADMIN_URL").unwrap_or_else(|_| "http://localhost:8084".into())
}

#[allow(dead_code)]
pub fn jobs_url() -> String {
    env::var("JOBS_URL").unwrap_or_else(|_| "http://localhost:8085".into())
}

pub fn billing_url() -> String {
    env::var("BILLING_URL").unwrap_or_else(|_| "http://localhost:8086".into())
}

pub fn ingest_url() -> String {
    env::var("INGEST_URL").unwrap_or_else(|_| "http://localhost:8087".into())
}

pub fn test_api_key() -> String {
    env::var("TEST_API_KEY").unwrap_or_else(|_| "test-api-key-for-e2e".into())
}

pub fn test_tenant_id() -> String {
    env::var("TEST_TENANT_ID")
        .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000001".into())
}

// ---------------------------------------------------------------------------
// HTTP Client
// ---------------------------------------------------------------------------

pub fn client() -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("Failed to create HTTP client")
}

/// Authenticated request to the gateway
pub async fn gateway_get(path: &str) -> Response {
    client()
        .get(format!("{}{}", gateway_url(), path))
        .header("Authorization", format!("Bearer {}", test_api_key()))
        .send()
        .await
        .expect("Gateway GET request failed")
}

pub async fn gateway_post(path: &str, body: &(impl Serialize + Sync)) -> Response {
    client()
        .post(format!("{}{}", gateway_url(), path))
        .header("Authorization", format!("Bearer {}", test_api_key()))
        .json(body)
        .send()
        .await
        .expect("Gateway POST request failed")
}

pub async fn gateway_put(path: &str, body: &(impl Serialize + Sync)) -> Response {
    client()
        .put(format!("{}{}", gateway_url(), path))
        .header("Authorization", format!("Bearer {}", test_api_key()))
        .json(body)
        .send()
        .await
        .expect("Gateway PUT request failed")
}

pub async fn gateway_delete(path: &str) -> Response {
    client()
        .delete(format!("{}{}", gateway_url(), path))
        .header("Authorization", format!("Bearer {}", test_api_key()))
        .send()
        .await
        .expect("Gateway DELETE request failed")
}

/// Internal service request with x-tenant-id header
pub async fn internal_post(base_url: &str, path: &str, body: &(impl Serialize + Sync)) -> Response {
    client()
        .post(format!("{}{}", base_url, path))
        .header("x-tenant-id", test_tenant_id())
        .header("x-user-id", Uuid::new_v4().to_string())
        .json(body)
        .send()
        .await
        .expect("Internal POST request failed")
}

pub async fn internal_get(base_url: &str, path: &str) -> Response {
    client()
        .get(format!("{}{}", base_url, path))
        .header("x-tenant-id", test_tenant_id())
        .send()
        .await
        .expect("Internal GET request failed")
}

pub async fn internal_put(base_url: &str, path: &str, body: &(impl Serialize + Sync)) -> Response {
    client()
        .put(format!("{}{}", base_url, path))
        .header("x-tenant-id", test_tenant_id())
        .json(body)
        .send()
        .await
        .expect("Internal PUT request failed")
}

pub async fn internal_delete(base_url: &str, path: &str) -> Response {
    client()
        .delete(format!("{}{}", base_url, path))
        .header("x-tenant-id", test_tenant_id())
        .send()
        .await
        .expect("Internal DELETE request failed")
}

// ---------------------------------------------------------------------------
// Shared request/response types (mirroring the service types)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryWriteRequest {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct MemoryWriteResponse {
    pub episode_id: Uuid,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemorySearchRequest {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_confidence: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct MemorySearchResponse {
    pub results: Vec<SearchResult>,
    pub total: usize,
    pub query_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct SearchResult {
    pub memory: serde_json::Value,
    pub score: f64,
}

#[derive(Debug, Serialize)]
pub struct UpdateMemoryRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub importance: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct MergeRequest {
    pub source_ids: Vec<Uuid>,
    pub merged_content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub importance: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct CreatePolicyRequest {
    pub name: String,
    pub rule_type: String,
    pub config: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct SimulateRequest {
    pub point_in_time: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct RecordUsageRequest {
    pub ops_count: i32,
}

#[derive(Debug, Deserialize)]
pub struct UsageResponse {
    pub plan: String,
    pub ops_this_month: i32,
    pub ops_limit: Option<i32>,
    pub active_memories: i64,
    pub active_entities: i64,
}

#[derive(Debug, Deserialize)]
pub struct PlanLimitsResponse {
    pub plan: String,
    pub within_limits: bool,
    pub ops_used: i32,
    pub ops_limit: Option<i32>,
    pub max_memories_per_user: i64,
    pub max_users: i64,
    pub features: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConnectorInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct WebhookPayload {
    pub source: String,
    pub items: Vec<WebhookItem>,
}

#[derive(Debug, Serialize)]
pub struct WebhookItem {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct BulkIngestRequest {
    pub texts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct IngestResponse {
    pub ingested: usize,
    pub total: usize,
    pub source: String,
}

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Deserialize)]
pub struct AuditEntry {
    pub id: Uuid,
    pub memory_id: Option<Uuid>,
    pub action: String,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
pub struct PolicyResponse {
    pub id: Uuid,
}

// ---------------------------------------------------------------------------
// Helper: assert JSON response
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub async fn assert_status(resp: Response, expected: StatusCode) -> serde_json::Value {
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    assert_eq!(
        status, expected,
        "Expected {} but got {}: {}",
        expected, status, body
    );
    serde_json::from_str(&body).unwrap_or(serde_json::Value::Null)
}

/// Generate a unique content string for test isolation
pub fn unique_content(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4().to_string()[..8].to_string())
}
