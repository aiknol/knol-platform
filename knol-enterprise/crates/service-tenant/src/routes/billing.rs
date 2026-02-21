//! Stripe billing, subscription management, usage tracking, and webhook endpoints.

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::Json;
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::auth::{audit, AppClaims, AppError};
use crate::stripe::{self, CreateCheckoutParams, StripeClient, StripeError};
use crate::TenantAppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CheckoutRequest {
    /// Plan name: "builder" or "growth".
    pub plan: String,
}

// -- Helpers --

fn require_stripe(state: &TenantAppState) -> Result<&str, AppError> {
    state
        .stripe_secret_key
        .as_deref()
        .ok_or_else(|| AppError::BadRequest("Billing is not configured".into()))
}

fn stripe_client<'a>(http: &'a reqwest::Client, key: &'a str) -> StripeClient<'a> {
    StripeClient::new(http, key)
}

fn map_stripe_err(e: StripeError) -> AppError {
    match e {
        StripeError::NotConfigured => AppError::BadRequest("Billing is not configured".into()),
        StripeError::ApiError(msg) => AppError::Internal(format!("Stripe: {}", msg)),
        StripeError::Network(msg) => AppError::Internal(format!("Stripe network: {}", msg)),
        StripeError::SignatureInvalid | StripeError::SignatureExpired => AppError::Unauthorized,
    }
}

#[derive(Debug, sqlx::FromRow)]
struct BillingTenantRow {
    id: Uuid,
    name: String,
    plan: String,
    usage_ops_month: i32,
    usage_limit: Option<i32>,
    stripe_customer_id: Option<String>,
    stripe_subscription_id: Option<String>,
    subscription_status: String,
    billing_period_start: Option<chrono::DateTime<chrono::Utc>>,
    billing_period_end: Option<chrono::DateTime<chrono::Utc>>,
}

async fn get_billing_tenant(
    pool: &sqlx::PgPool,
    tenant_id: Uuid,
) -> Result<BillingTenantRow, AppError> {
    sqlx::query_as::<_, BillingTenantRow>(
        r#"SELECT id, name, plan, usage_ops_month, usage_limit,
                  stripe_customer_id, stripe_subscription_id, subscription_status,
                  billing_period_start, billing_period_end
           FROM tenants WHERE id = $1"#,
    )
    .bind(tenant_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("Tenant not found".into()))
}

/// Ensure Stripe customer exists for this tenant; create lazily if needed.
async fn ensure_stripe_customer(
    state: &TenantAppState,
    tenant: &BillingTenantRow,
    email: &str,
) -> Result<String, AppError> {
    if let Some(cid) = &tenant.stripe_customer_id {
        return Ok(cid.clone());
    }
    let key = require_stripe(state)?;
    let client = stripe_client(&state.http_client, key);
    let customer = client
        .create_customer(email, &tenant.name, &tenant.id.to_string())
        .await
        .map_err(map_stripe_err)?;

    sqlx::query("UPDATE tenants SET stripe_customer_id = $1, updated_at = NOW() WHERE id = $2")
        .bind(&customer.id)
        .bind(tenant.id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(customer.id)
}

// ── Checkout ────────────────────────────────────────────────────────────

/// Create a Stripe Checkout Session for subscribing to a plan.
#[utoipa::path(
    post,
    path = "/app/billing/checkout",
    tag = "Billing",
    security(("bearer_auth" = [])),
    request_body = CheckoutRequest,
    responses(
        (status = 200, description = "Checkout session created"),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn create_checkout(
    claims: AppClaims,
    State(state): State<Arc<TenantAppState>>,
    Json(body): Json<CheckoutRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !matches!(claims.role.as_str(), "owner" | "admin") {
        return Err(AppError::Forbidden);
    }

    let plan = body.plan.trim().to_lowercase();
    if !matches!(plan.as_str(), "builder" | "growth") {
        return Err(AppError::BadRequest(
            "Invalid plan. Choose 'builder' or 'growth'.".into(),
        ));
    }

    let key = require_stripe(&state)?;
    let configs = stripe::load_plan_configs();
    let plan_config = configs
        .values()
        .find(|c| c.plan_name == plan)
        .ok_or_else(|| {
            AppError::BadRequest(format!(
                "Stripe price not configured for plan '{}'. Set STRIPE_PRICE_{} env var.",
                plan,
                plan.to_uppercase()
            ))
        })?;

    let tenant = get_billing_tenant(&state.db_pool, claims.tenant_id).await?;
    let customer_id = ensure_stripe_customer(&state, &tenant, &claims.email).await?;

    let success_url = std::env::var("STRIPE_CHECKOUT_SUCCESS_URL").unwrap_or_else(|_| {
        "https://cloud.aiknol.com/billing?session_id={CHECKOUT_SESSION_ID}".to_string()
    });
    let cancel_url = std::env::var("STRIPE_CHECKOUT_CANCEL_URL")
        .unwrap_or_else(|_| "https://cloud.aiknol.com/billing?canceled=true".to_string());

    let client = stripe_client(&state.http_client, key);
    let session = client
        .create_checkout_session(&CreateCheckoutParams {
            customer_id,
            price_id: plan_config.stripe_price_id.clone(),
            success_url,
            cancel_url,
            trial_days: None,
        })
        .await
        .map_err(map_stripe_err)?;

    audit(
        &state,
        claims.tenant_id,
        Some(&claims),
        "checkout_initiated",
        "billing",
        Some(&session.id),
        None,
        Some(serde_json::json!({"plan": plan})),
        None,
    )
    .await;

    Ok(Json(serde_json::json!({
        "checkout_url": session.url,
        "session_id": session.id,
    })))
}

// ── Portal ──────────────────────────────────────────────────────────────

/// Create a Stripe Customer Portal session for managing billing.
#[utoipa::path(
    post,
    path = "/app/billing/portal",
    tag = "Billing",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Portal session created"),
        (status = 400, description = "No billing account"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn create_portal(
    claims: AppClaims,
    State(state): State<Arc<TenantAppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !matches!(claims.role.as_str(), "owner" | "admin") {
        return Err(AppError::Forbidden);
    }
    let key = require_stripe(&state)?;
    let tenant = get_billing_tenant(&state.db_pool, claims.tenant_id).await?;
    let customer_id = tenant.stripe_customer_id.as_deref().ok_or_else(|| {
        AppError::BadRequest("No billing account. Subscribe to a plan first.".into())
    })?;

    let return_url = std::env::var("STRIPE_PORTAL_RETURN_URL")
        .unwrap_or_else(|_| "https://cloud.aiknol.com/billing".to_string());

    let client = stripe_client(&state.http_client, key);
    let portal = client
        .create_portal_session(customer_id, &return_url)
        .await
        .map_err(map_stripe_err)?;

    Ok(Json(serde_json::json!({
        "portal_url": portal.url,
    })))
}

// ── Subscription ────────────────────────────────────────────────────────

/// Get current subscription details.
#[utoipa::path(
    get,
    path = "/app/billing/subscription",
    tag = "Billing",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Subscription details"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn get_subscription(
    claims: AppClaims,
    State(state): State<Arc<TenantAppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let tenant = get_billing_tenant(&state.db_pool, claims.tenant_id).await?;

    let mut result = serde_json::json!({
        "plan": tenant.plan,
        "subscription_status": tenant.subscription_status,
        "billing_period_start": tenant.billing_period_start.map(|t| t.to_rfc3339()),
        "billing_period_end": tenant.billing_period_end.map(|t| t.to_rfc3339()),
        "usage_ops_month": tenant.usage_ops_month,
        "usage_limit": tenant.usage_limit,
        "has_stripe_customer": tenant.stripe_customer_id.is_some(),
    });

    if let (Some(sub_id), Some(key)) = (
        &tenant.stripe_subscription_id,
        state.stripe_secret_key.as_deref(),
    ) {
        let client = stripe_client(&state.http_client, key);
        if let Ok(sub) = client.get_subscription(sub_id).await {
            result["stripe_status"] = serde_json::json!(sub.status);
            result["cancel_at_period_end"] = serde_json::json!(sub.cancel_at_period_end);
        }
    }

    Ok(Json(result))
}

/// Cancel subscription at period end (owner only).
#[utoipa::path(
    post,
    path = "/app/billing/cancel",
    tag = "Billing",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Subscription canceled at period end"),
        (status = 400, description = "No active subscription"),
        (status = 403, description = "Forbidden — owner only"),
    )
)]
pub async fn cancel_subscription(
    claims: AppClaims,
    State(state): State<Arc<TenantAppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    if claims.role != "owner" {
        return Err(AppError::Forbidden);
    }
    let key = require_stripe(&state)?;
    let tenant = get_billing_tenant(&state.db_pool, claims.tenant_id).await?;
    let sub_id = tenant
        .stripe_subscription_id
        .as_deref()
        .ok_or_else(|| AppError::BadRequest("No active subscription".into()))?;

    let client = stripe_client(&state.http_client, key);
    let sub = client
        .cancel_subscription(sub_id, true)
        .await
        .map_err(map_stripe_err)?;

    sqlx::query(
        "UPDATE tenants SET subscription_status = 'canceled', updated_at = NOW() WHERE id = $1",
    )
    .bind(claims.tenant_id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    audit(
        &state,
        claims.tenant_id,
        Some(&claims),
        "cancel_subscription",
        "billing",
        Some(sub_id),
        None,
        Some(serde_json::json!({"cancel_at_period_end": sub.cancel_at_period_end})),
        None,
    )
    .await;

    Ok(Json(serde_json::json!({
        "canceled": true,
        "cancel_at_period_end": sub.cancel_at_period_end,
        "current_period_end": tenant.billing_period_end.map(|t| t.to_rfc3339()),
    })))
}

/// Reactivate a canceled subscription (owner only).
#[utoipa::path(
    post,
    path = "/app/billing/reactivate",
    tag = "Billing",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Subscription reactivated"),
        (status = 400, description = "No subscription to reactivate"),
        (status = 403, description = "Forbidden — owner only"),
    )
)]
pub async fn reactivate_subscription(
    claims: AppClaims,
    State(state): State<Arc<TenantAppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    if claims.role != "owner" {
        return Err(AppError::Forbidden);
    }
    let key = require_stripe(&state)?;
    let tenant = get_billing_tenant(&state.db_pool, claims.tenant_id).await?;
    let sub_id = tenant
        .stripe_subscription_id
        .as_deref()
        .ok_or_else(|| AppError::BadRequest("No subscription to reactivate".into()))?;

    let client = stripe_client(&state.http_client, key);
    let sub = client
        .reactivate_subscription(sub_id)
        .await
        .map_err(map_stripe_err)?;

    sqlx::query("UPDATE tenants SET subscription_status = $1, updated_at = NOW() WHERE id = $2")
        .bind(&sub.status)
        .bind(claims.tenant_id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    audit(
        &state,
        claims.tenant_id,
        Some(&claims),
        "reactivate_subscription",
        "billing",
        Some(sub_id),
        None,
        Some(serde_json::json!({"status": sub.status})),
        None,
    )
    .await;

    Ok(Json(serde_json::json!({
        "reactivated": true,
        "status": sub.status,
    })))
}

// ── Invoices ────────────────────────────────────────────────────────────

/// List invoices from Stripe.
#[utoipa::path(
    get,
    path = "/app/billing/invoices",
    tag = "Billing",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Invoices list"),
        (status = 400, description = "No billing account"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn list_invoices(
    claims: AppClaims,
    State(state): State<Arc<TenantAppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !matches!(claims.role.as_str(), "owner" | "admin") {
        return Err(AppError::Forbidden);
    }
    let key = require_stripe(&state)?;
    let tenant = get_billing_tenant(&state.db_pool, claims.tenant_id).await?;
    let customer_id = tenant
        .stripe_customer_id
        .as_deref()
        .ok_or_else(|| AppError::BadRequest("No billing account".into()))?;

    let client = stripe_client(&state.http_client, key);
    let invoices = client
        .list_invoices(customer_id, 20)
        .await
        .map_err(map_stripe_err)?;

    let items: Vec<serde_json::Value> = invoices
        .data
        .iter()
        .map(|inv| {
            serde_json::json!({
                "id": inv.id,
                "status": inv.status,
                "amount_due": inv.amount_due,
                "amount_paid": inv.amount_paid,
                "currency": inv.currency,
                "hosted_invoice_url": inv.hosted_invoice_url,
                "created": inv.created,
                "period_start": inv.period_start,
                "period_end": inv.period_end,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "invoices": items,
        "has_more": invoices.has_more,
    })))
}

/// Preview the next upcoming invoice.
#[utoipa::path(
    get,
    path = "/app/billing/invoices/upcoming",
    tag = "Billing",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Upcoming invoice preview"),
        (status = 400, description = "No billing account"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn upcoming_invoice(
    claims: AppClaims,
    State(state): State<Arc<TenantAppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !matches!(claims.role.as_str(), "owner" | "admin") {
        return Err(AppError::Forbidden);
    }
    let key = require_stripe(&state)?;
    let tenant = get_billing_tenant(&state.db_pool, claims.tenant_id).await?;
    let customer_id = tenant
        .stripe_customer_id
        .as_deref()
        .ok_or_else(|| AppError::BadRequest("No billing account".into()))?;

    let client = stripe_client(&state.http_client, key);
    let invoice = client
        .upcoming_invoice(customer_id)
        .await
        .map_err(map_stripe_err)?;

    Ok(Json(serde_json::json!({
        "amount_due": invoice.amount_due,
        "currency": invoice.currency,
        "period_start": invoice.period_start,
        "period_end": invoice.period_end,
    })))
}

// ── Usage ───────────────────────────────────────────────────────────────

/// Get current month usage and alert status.
#[utoipa::path(
    get,
    path = "/app/billing/usage",
    tag = "Usage",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Usage data"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn get_usage(
    claims: AppClaims,
    State(state): State<Arc<TenantAppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let tenant = get_billing_tenant(&state.db_pool, claims.tenant_id).await?;
    let month = chrono::Utc::now().format("%Y-%m").to_string();

    let mut alerts_triggered: Vec<i32> = Vec::new();

    if let Some(limit) = tenant.usage_limit {
        if limit > 0 {
            let pct = (tenant.usage_ops_month as f64 / limit as f64 * 100.0) as i32;

            for threshold in [50, 80, 100] {
                if pct >= threshold {
                    let inserted = sqlx::query_scalar::<_, Uuid>(
                        r#"INSERT INTO usage_alerts (tenant_id, threshold_pct, month)
                           VALUES ($1, $2, $3)
                           ON CONFLICT (tenant_id, threshold_pct, month) DO NOTHING
                           RETURNING id"#,
                    )
                    .bind(claims.tenant_id)
                    .bind(threshold)
                    .bind(&month)
                    .fetch_optional(&state.db_pool)
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;

                    if inserted.is_some() {
                        alerts_triggered.push(threshold);
                    }
                }
            }

            if !alerts_triggered.is_empty() {
                audit(
                    &state,
                    claims.tenant_id,
                    None,
                    "usage_alert",
                    "billing",
                    None,
                    None,
                    Some(serde_json::json!({"thresholds": alerts_triggered, "usage": tenant.usage_ops_month, "limit": limit})),
                    None,
                )
                .await;
            }
        }
    }

    let existing_alerts = sqlx::query_scalar::<_, i32>(
        "SELECT threshold_pct FROM usage_alerts WHERE tenant_id = $1 AND month = $2 ORDER BY threshold_pct",
    )
    .bind(claims.tenant_id)
    .bind(&month)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let usage_pct = tenant.usage_limit.map(|limit| {
        if limit > 0 {
            (tenant.usage_ops_month as f64 / limit as f64 * 100.0).round()
        } else {
            0.0
        }
    });

    Ok(Json(serde_json::json!({
        "plan": tenant.plan,
        "ops_this_month": tenant.usage_ops_month,
        "ops_limit": tenant.usage_limit,
        "usage_percentage": usage_pct,
        "alerts_triggered": existing_alerts,
        "month": month,
    })))
}

/// Get monthly usage history (last 12 months).
#[utoipa::path(
    get,
    path = "/app/billing/usage/history",
    tag = "Usage",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Monthly usage history"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn get_usage_history(
    claims: AppClaims,
    State(state): State<Arc<TenantAppState>>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    #[derive(sqlx::FromRow)]
    struct HistoryRow {
        month: String,
        ops_count: i32,
        plan: String,
        usage_limit: Option<i32>,
    }

    let rows = sqlx::query_as::<_, HistoryRow>(
        r#"SELECT month, ops_count, plan, usage_limit
           FROM usage_history
           WHERE tenant_id = $1
           ORDER BY month DESC
           LIMIT 12"#,
    )
    .bind(claims.tenant_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(
        rows.iter()
            .map(|r| {
                serde_json::json!({
                    "month": r.month,
                    "ops_count": r.ops_count,
                    "plan": r.plan,
                    "usage_limit": r.usage_limit,
                })
            })
            .collect(),
    ))
}

// ── Stripe Webhook ──────────────────────────────────────────────────────

/// Stripe webhook handler (unauthenticated, signature-verified).
#[utoipa::path(
    post,
    path = "/app/webhooks/stripe",
    tag = "Billing",
    request_body(content = String, description = "Raw Stripe event JSON payload", content_type = "application/json"),
    responses(
        (status = 200, description = "Webhook processed"),
        (status = 400, description = "Invalid payload"),
        (status = 401, description = "Invalid signature"),
    )
)]
pub async fn stripe_webhook(
    State(state): State<Arc<TenantAppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Result<StatusCode, AppError> {
    let webhook_secret = state
        .stripe_webhook_secret
        .as_deref()
        .ok_or_else(|| AppError::BadRequest("Webhook not configured".into()))?;

    let sig_header = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    stripe::verify_webhook_signature(&body, sig_header, webhook_secret, 300)
        .map_err(map_stripe_err)?;

    let event: stripe::StripeEvent = serde_json::from_slice(&body)
        .map_err(|e| AppError::BadRequest(format!("Invalid event payload: {}", e)))?;

    // Idempotency check
    let already_processed = sqlx::query_scalar::<_, bool>(
        "SELECT processed FROM stripe_event_log WHERE stripe_event_id = $1",
    )
    .bind(&event.id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if already_processed.is_some() {
        return Ok(StatusCode::OK);
    }

    let _ = sqlx::query(
        r#"INSERT INTO stripe_event_log (stripe_event_id, event_type, payload)
           VALUES ($1, $2, $3)
           ON CONFLICT (stripe_event_id) DO NOTHING"#,
    )
    .bind(&event.id)
    .bind(&event.event_type)
    .bind(&event.data.object)
    .execute(&state.db_pool)
    .await;

    let result = match event.event_type.as_str() {
        "checkout.session.completed" => handle_checkout_completed(&state, &event.data.object).await,
        "customer.subscription.updated" => {
            handle_subscription_updated(&state, &event.data.object).await
        }
        "customer.subscription.deleted" => {
            handle_subscription_deleted(&state, &event.data.object).await
        }
        "invoice.payment_succeeded" => handle_invoice_paid(&state, &event.data.object).await,
        "invoice.payment_failed" => handle_invoice_failed(&state, &event.data.object).await,
        _ => Ok(()),
    };

    let error_msg = result.as_ref().err().map(|e| format!("{:?}", e));
    let _ = sqlx::query(
        "UPDATE stripe_event_log SET processed = true, error = $2 WHERE stripe_event_id = $1",
    )
    .bind(&event.id)
    .bind(error_msg)
    .execute(&state.db_pool)
    .await;

    match result {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => {
            tracing::error!("Stripe webhook error for {}: {:?}", event.event_type, e);
            Ok(StatusCode::OK)
        }
    }
}

async fn handle_checkout_completed(
    state: &TenantAppState,
    object: &serde_json::Value,
) -> Result<(), AppError> {
    let customer_id = object["customer"]
        .as_str()
        .ok_or_else(|| AppError::Internal("Missing customer in checkout".into()))?;
    let subscription_id = object["subscription"]
        .as_str()
        .ok_or_else(|| AppError::Internal("Missing subscription in checkout".into()))?;

    let key = require_stripe(state)?;
    let client = stripe_client(&state.http_client, key);
    let sub = client
        .get_subscription(subscription_id)
        .await
        .map_err(map_stripe_err)?;

    let price_id = sub
        .items
        .data
        .first()
        .map(|item| item.price.id.as_str())
        .ok_or_else(|| AppError::Internal("No subscription items".into()))?;

    let (plan, usage_limit) = stripe::plan_from_price_id(price_id)
        .ok_or_else(|| AppError::Internal(format!("Unknown price ID: {}", price_id)))?;

    let updated = sqlx::query(
        r#"UPDATE tenants
           SET plan = $1, usage_limit = $2, stripe_customer_id = $3,
               stripe_subscription_id = $4, subscription_status = $5,
               billing_period_start = to_timestamp($6::double precision),
               billing_period_end = to_timestamp($7::double precision),
               updated_at = NOW()
           WHERE stripe_customer_id = $3 OR id = (
               SELECT id FROM tenants WHERE stripe_customer_id = $3 LIMIT 1
           )"#,
    )
    .bind(&plan)
    .bind(usage_limit)
    .bind(customer_id)
    .bind(subscription_id)
    .bind(&sub.status)
    .bind(sub.current_period_start as f64)
    .bind(sub.current_period_end as f64)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if updated.rows_affected() > 0 {
        if let Ok(Some(tenant_id)) =
            sqlx::query_scalar::<_, Uuid>("SELECT id FROM tenants WHERE stripe_customer_id = $1")
                .bind(customer_id)
                .fetch_optional(&state.db_pool)
                .await
        {
            audit(
                state,
                tenant_id,
                None,
                "subscription_created",
                "billing",
                Some(subscription_id),
                None,
                Some(serde_json::json!({"plan": plan, "status": sub.status})),
                None,
            )
            .await;
        }
    }

    Ok(())
}

async fn handle_subscription_updated(
    state: &TenantAppState,
    object: &serde_json::Value,
) -> Result<(), AppError> {
    let sub_id = object["id"]
        .as_str()
        .ok_or_else(|| AppError::Internal("Missing subscription id".into()))?;
    let status = object["status"].as_str().unwrap_or("active");
    let cancel_at_period_end = object["cancel_at_period_end"].as_bool().unwrap_or(false);
    let period_start = object["current_period_start"].as_f64();
    let period_end = object["current_period_end"].as_f64();

    let price_id = object["items"]["data"]
        .as_array()
        .and_then(|items| items.first())
        .and_then(|item| item["price"]["id"].as_str());

    let (plan, usage_limit) = if let Some(pid) = price_id {
        stripe::plan_from_price_id(pid).unwrap_or_else(|| ("free".to_string(), None))
    } else {
        ("free".to_string(), None)
    };

    let effective_status = if cancel_at_period_end {
        "canceled"
    } else {
        status
    };

    let mut query = String::from("UPDATE tenants SET subscription_status = $1, updated_at = NOW()");
    let mut param_idx = 2;

    if price_id.is_some() {
        query.push_str(&format!(
            ", plan = ${}, usage_limit = ${}",
            param_idx,
            param_idx + 1
        ));
        param_idx += 2;
    }
    if period_start.is_some() {
        query.push_str(&format!(
            ", billing_period_start = to_timestamp(${}::double precision)",
            param_idx
        ));
        param_idx += 1;
    }
    if period_end.is_some() {
        query.push_str(&format!(
            ", billing_period_end = to_timestamp(${}::double precision)",
            param_idx
        ));
        param_idx += 1;
    }
    query.push_str(&format!(" WHERE stripe_subscription_id = ${}", param_idx));

    let mut q = sqlx::query(&query).bind(effective_status);
    if price_id.is_some() {
        q = q.bind(&plan).bind(usage_limit);
    }
    if let Some(ps) = period_start {
        q = q.bind(ps);
    }
    if let Some(pe) = period_end {
        q = q.bind(pe);
    }
    q = q.bind(sub_id);

    q.execute(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(())
}

async fn handle_subscription_deleted(
    state: &TenantAppState,
    object: &serde_json::Value,
) -> Result<(), AppError> {
    let sub_id = object["id"]
        .as_str()
        .ok_or_else(|| AppError::Internal("Missing subscription id".into()))?;

    sqlx::query(
        r#"UPDATE tenants
           SET plan = 'free', usage_limit = NULL, subscription_status = 'canceled',
               stripe_subscription_id = NULL, updated_at = NOW()
           WHERE stripe_subscription_id = $1"#,
    )
    .bind(sub_id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if let Ok(Some(tenant_id)) =
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM tenants WHERE stripe_subscription_id = $1")
            .bind(sub_id)
            .fetch_optional(&state.db_pool)
            .await
    {
        audit(
            state,
            tenant_id,
            None,
            "subscription_deleted",
            "billing",
            Some(sub_id),
            None,
            Some(serde_json::json!({"plan": "free"})),
            None,
        )
        .await;
    }

    Ok(())
}

async fn handle_invoice_paid(
    state: &TenantAppState,
    object: &serde_json::Value,
) -> Result<(), AppError> {
    let customer_id = object["customer"]
        .as_str()
        .ok_or_else(|| AppError::Internal("Missing customer in invoice".into()))?;

    sqlx::query(
        "UPDATE tenants SET subscription_status = 'active', updated_at = NOW() WHERE stripe_customer_id = $1",
    )
    .bind(customer_id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(())
}

async fn handle_invoice_failed(
    state: &TenantAppState,
    object: &serde_json::Value,
) -> Result<(), AppError> {
    let customer_id = object["customer"]
        .as_str()
        .ok_or_else(|| AppError::Internal("Missing customer in invoice".into()))?;

    sqlx::query(
        "UPDATE tenants SET subscription_status = 'past_due', updated_at = NOW() WHERE stripe_customer_id = $1",
    )
    .bind(customer_id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if let Ok(Some(tenant_id)) =
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM tenants WHERE stripe_customer_id = $1")
            .bind(customer_id)
            .fetch_optional(&state.db_pool)
            .await
    {
        audit(
            state,
            tenant_id,
            None,
            "payment_failed",
            "billing",
            None,
            None,
            Some(serde_json::json!({"status": "past_due"})),
            None,
        )
        .await;
    }

    Ok(())
}
