// =============================================================================
// Billing Service E2E Tests (service-billing, port 8086)
// Covers: /internal/usage, /internal/usage/record, /internal/plan/check,
//         /internal/billing/reset-monthly, /health
//         Plan tiers: free, developer, pro, team, enterprise
// =============================================================================

use crate::harness::*;
use reqwest::StatusCode;

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

#[tokio::test]
async fn billing_health_returns_200() {
    let resp = client()
        .get(format!("{}/health", billing_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// Get usage
// ---------------------------------------------------------------------------

#[tokio::test]
async fn billing_get_usage_returns_valid_response() {
    let resp = internal_get(&billing_url(), "/internal/usage").await;
    assert!(resp.status().is_success());
    let usage: UsageResponse = resp.json().await.unwrap();
    assert!(!usage.plan.is_empty());
    assert!(usage.ops_this_month >= 0);
    assert!(usage.active_memories >= 0);
    assert!(usage.active_entities >= 0);
}

#[tokio::test]
async fn billing_get_usage_rejects_missing_tenant() {
    let resp = client()
        .get(format!("{}/internal/usage", billing_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// Record usage
// ---------------------------------------------------------------------------

#[tokio::test]
async fn billing_record_usage_increments_ops() {
    // Get current usage
    let resp = internal_get(&billing_url(), "/internal/usage").await;
    assert!(resp.status().is_success());
    let before: UsageResponse = resp.json().await.unwrap();

    // Record some ops
    let body = RecordUsageRequest { ops_count: 5 };
    let resp = internal_post(&billing_url(), "/internal/usage/record", &body).await;
    assert!(resp.status().is_success());

    // Verify increment
    let resp = internal_get(&billing_url(), "/internal/usage").await;
    let after: UsageResponse = resp.json().await.unwrap();
    assert!(
        after.ops_this_month >= before.ops_this_month + 5,
        "Ops should have increased by at least 5: before={}, after={}",
        before.ops_this_month,
        after.ops_this_month
    );
}

#[tokio::test]
async fn billing_record_usage_zero_ops() {
    let body = RecordUsageRequest { ops_count: 0 };
    let resp = internal_post(&billing_url(), "/internal/usage/record", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn billing_record_usage_single_op() {
    let body = RecordUsageRequest { ops_count: 1 };
    let resp = internal_post(&billing_url(), "/internal/usage/record", &body).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn billing_record_usage_large_batch() {
    let body = RecordUsageRequest { ops_count: 1000 };
    let resp = internal_post(&billing_url(), "/internal/usage/record", &body).await;
    assert!(resp.status().is_success());
}

// ---------------------------------------------------------------------------
// Check plan limits
// ---------------------------------------------------------------------------

#[tokio::test]
async fn billing_check_plan_limits() {
    let resp = internal_get(&billing_url(), "/internal/plan/check").await;
    assert!(resp.status().is_success());
    let limits: PlanLimitsResponse = resp.json().await.unwrap();
    assert!(!limits.plan.is_empty());
    assert!(!limits.features.is_empty());
    assert!(limits.max_memories_per_user > 0);
    assert!(limits.max_users > 0);
}

#[tokio::test]
async fn billing_check_plan_limits_contains_expected_features() {
    let resp = internal_get(&billing_url(), "/internal/plan/check").await;
    assert!(resp.status().is_success());
    let limits: PlanLimitsResponse = resp.json().await.unwrap();

    // All plans should include at least vector_search
    assert!(
        limits.features.iter().any(|f| f.contains("vector") || f.contains("search")),
        "Plan should include vector search: {:?}",
        limits.features
    );
}

#[tokio::test]
async fn billing_check_plan_limits_rejects_missing_tenant() {
    let resp = client()
        .get(format!("{}/internal/plan/check", billing_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// Reset monthly usage
// ---------------------------------------------------------------------------

#[tokio::test]
async fn billing_reset_monthly_usage() {
    let resp = internal_post(
        &billing_url(),
        "/internal/billing/reset-monthly",
        &serde_json::json!({}),
    )
    .await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn billing_reset_and_verify_zero() {
    // Reset
    let resp = internal_post(
        &billing_url(),
        "/internal/billing/reset-monthly",
        &serde_json::json!({}),
    )
    .await;
    assert!(resp.status().is_success());

    // Check usage is 0
    let resp = internal_get(&billing_url(), "/internal/usage").await;
    assert!(resp.status().is_success());
    let usage: UsageResponse = resp.json().await.unwrap();
    assert_eq!(usage.ops_this_month, 0, "Ops should be 0 after reset");
}

// ---------------------------------------------------------------------------
// Plan tier validation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn billing_plan_within_limits_initially() {
    // Fresh tenant or after reset should be within limits
    let resp = internal_get(&billing_url(), "/internal/plan/check").await;
    assert!(resp.status().is_success());
    let limits: PlanLimitsResponse = resp.json().await.unwrap();
    // After reset, should be within limits
    assert!(
        limits.within_limits || limits.ops_limit.is_none(),
        "Should be within limits or unlimited"
    );
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[tokio::test]
async fn billing_record_negative_ops_handled() {
    let body = serde_json::json!({"ops_count": -1});
    let resp = internal_post(&billing_url(), "/internal/usage/record", &body).await;
    let status = resp.status();
    assert!(
        status.is_success() || status == StatusCode::BAD_REQUEST,
        "Negative ops: {}",
        status
    );
}
