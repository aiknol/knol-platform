//! Stripe REST API client — thin wrapper over reqwest.
//! Uses the Stripe v1 REST API directly to avoid SDK version coupling.

use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::Deserialize;
use sha2::Sha256;
use std::collections::HashMap;

const STRIPE_API_BASE: &str = "https://api.stripe.com/v1";

pub struct StripeClient<'a> {
    http: &'a Client,
    secret_key: &'a str,
}

impl<'a> StripeClient<'a> {
    pub fn new(http: &'a Client, secret_key: &'a str) -> Self {
        Self { http, secret_key }
    }

    // -- Customer --

    pub async fn create_customer(
        &self,
        email: &str,
        name: &str,
        tenant_id: &str,
    ) -> Result<StripeCustomer, StripeError> {
        let params = [
            ("email", email),
            ("name", name),
            ("metadata[tenant_id]", tenant_id),
        ];
        let resp = self
            .http
            .post(format!("{}/customers", STRIPE_API_BASE))
            .basic_auth(self.secret_key, None::<&str>)
            .form(&params)
            .send()
            .await
            .map_err(|e| StripeError::Network(e.to_string()))?;
        parse_response(resp).await
    }

    // -- Checkout Session --

    pub async fn create_checkout_session(
        &self,
        params: &CreateCheckoutParams,
    ) -> Result<CheckoutSession, StripeError> {
        let mut form: Vec<(&str, &str)> = vec![
            ("customer", &params.customer_id),
            ("mode", "subscription"),
            ("line_items[0][price]", &params.price_id),
            ("line_items[0][quantity]", "1"),
            ("success_url", &params.success_url),
            ("cancel_url", &params.cancel_url),
        ];
        let trial_str;
        if let Some(days) = params.trial_days {
            trial_str = days.to_string();
            form.push(("subscription_data[trial_period_days]", &trial_str));
        }
        let resp = self
            .http
            .post(format!("{}/checkout/sessions", STRIPE_API_BASE))
            .basic_auth(self.secret_key, None::<&str>)
            .form(&form)
            .send()
            .await
            .map_err(|e| StripeError::Network(e.to_string()))?;
        parse_response(resp).await
    }

    // -- Customer Portal --

    pub async fn create_portal_session(
        &self,
        customer_id: &str,
        return_url: &str,
    ) -> Result<PortalSession, StripeError> {
        let params = [("customer", customer_id), ("return_url", return_url)];
        let resp = self
            .http
            .post(format!("{}/billing_portal/sessions", STRIPE_API_BASE))
            .basic_auth(self.secret_key, None::<&str>)
            .form(&params)
            .send()
            .await
            .map_err(|e| StripeError::Network(e.to_string()))?;
        parse_response(resp).await
    }

    // -- Subscription --

    pub async fn get_subscription(&self, sub_id: &str) -> Result<StripeSubscription, StripeError> {
        let resp = self
            .http
            .get(format!("{}/subscriptions/{}", STRIPE_API_BASE, sub_id))
            .basic_auth(self.secret_key, None::<&str>)
            .send()
            .await
            .map_err(|e| StripeError::Network(e.to_string()))?;
        parse_response(resp).await
    }

    pub async fn cancel_subscription(
        &self,
        sub_id: &str,
        at_period_end: bool,
    ) -> Result<StripeSubscription, StripeError> {
        if at_period_end {
            let params = [("cancel_at_period_end", "true")];
            let resp = self
                .http
                .post(format!("{}/subscriptions/{}", STRIPE_API_BASE, sub_id))
                .basic_auth(self.secret_key, None::<&str>)
                .form(&params)
                .send()
                .await
                .map_err(|e| StripeError::Network(e.to_string()))?;
            parse_response(resp).await
        } else {
            let resp = self
                .http
                .delete(format!("{}/subscriptions/{}", STRIPE_API_BASE, sub_id))
                .basic_auth(self.secret_key, None::<&str>)
                .send()
                .await
                .map_err(|e| StripeError::Network(e.to_string()))?;
            parse_response(resp).await
        }
    }

    pub async fn reactivate_subscription(
        &self,
        sub_id: &str,
    ) -> Result<StripeSubscription, StripeError> {
        let params = [("cancel_at_period_end", "false")];
        let resp = self
            .http
            .post(format!("{}/subscriptions/{}", STRIPE_API_BASE, sub_id))
            .basic_auth(self.secret_key, None::<&str>)
            .form(&params)
            .send()
            .await
            .map_err(|e| StripeError::Network(e.to_string()))?;
        parse_response(resp).await
    }

    // -- Invoices --

    pub async fn list_invoices(
        &self,
        customer_id: &str,
        limit: u32,
    ) -> Result<StripeList<StripeInvoice>, StripeError> {
        let limit_str = limit.to_string();
        let params = [("customer", customer_id), ("limit", &limit_str)];
        let resp = self
            .http
            .get(format!("{}/invoices", STRIPE_API_BASE))
            .basic_auth(self.secret_key, None::<&str>)
            .query(&params)
            .send()
            .await
            .map_err(|e| StripeError::Network(e.to_string()))?;
        parse_response(resp).await
    }

    pub async fn upcoming_invoice(&self, customer_id: &str) -> Result<StripeInvoice, StripeError> {
        let params = [("customer", customer_id)];
        let resp = self
            .http
            .get(format!("{}/invoices/upcoming", STRIPE_API_BASE))
            .basic_auth(self.secret_key, None::<&str>)
            .query(&params)
            .send()
            .await
            .map_err(|e| StripeError::Network(e.to_string()))?;
        parse_response(resp).await
    }
}

// -- Webhook signature verification --

/// Verify Stripe webhook signature (HMAC-SHA256).
///
/// SECURITY: Uses `hmac` crate which provides constant-time comparison
/// via `verify_slice`, preventing timing side-channel attacks.
pub fn verify_webhook_signature(
    payload: &[u8],
    sig_header: &str,
    webhook_secret: &str,
    tolerance_secs: i64,
) -> Result<(), StripeError> {
    let mut timestamp: Option<&str> = None;
    let mut signatures: Vec<&str> = Vec::new();

    for part in sig_header.split(',') {
        let part = part.trim();
        if let Some(t) = part.strip_prefix("t=") {
            timestamp = Some(t);
        } else if let Some(v1) = part.strip_prefix("v1=") {
            signatures.push(v1);
        }
    }

    let timestamp = timestamp.ok_or(StripeError::SignatureInvalid)?;
    if signatures.is_empty() {
        return Err(StripeError::SignatureInvalid);
    }

    let ts: i64 = timestamp
        .parse()
        .map_err(|_| StripeError::SignatureInvalid)?;
    let now = chrono::Utc::now().timestamp();
    if (now - ts).abs() > tolerance_secs {
        return Err(StripeError::SignatureExpired);
    }

    let signed_payload = format!(
        "{}.{}",
        timestamp,
        std::str::from_utf8(payload).map_err(|_| StripeError::SignatureInvalid)?
    );
    // Compute HMAC of the signed payload
    let mut mac = Hmac::<Sha256>::new_from_slice(webhook_secret.as_bytes())
        .map_err(|_| StripeError::SignatureInvalid)?;
    mac.update(signed_payload.as_bytes());

    // Compare using constant-time verification to prevent timing attacks.
    // Decode each hex signature and compare against the computed HMAC.
    for sig in &signatures {
        if let Ok(sig_bytes) = hex::decode(sig) {
            // Clone the mac so we can reuse it for multiple signatures
            let mac_clone = mac.clone();
            if mac_clone.verify_slice(&sig_bytes).is_ok() {
                return Ok(());
            }
        }
    }
    Err(StripeError::SignatureInvalid)
}

// -- Plan mapping --

pub struct PlanConfig {
    pub stripe_price_id: String,
    pub plan_name: String,
    pub usage_limit: Option<i32>,
}

pub fn load_plan_configs() -> HashMap<String, PlanConfig> {
    let mut plans = HashMap::new();

    if let Ok(price_id) = std::env::var("STRIPE_PRICE_BUILDER") {
        plans.insert(
            price_id.clone(),
            PlanConfig {
                stripe_price_id: price_id,
                plan_name: "builder".to_string(),
                usage_limit: Some(100_000),
            },
        );
    }
    if let Ok(price_id) = std::env::var("STRIPE_PRICE_GROWTH") {
        plans.insert(
            price_id.clone(),
            PlanConfig {
                stripe_price_id: price_id,
                plan_name: "growth".to_string(),
                usage_limit: Some(500_000),
            },
        );
    }
    plans
}

/// Map a Stripe Price ID to a (plan_name, usage_limit) tuple.
pub fn plan_from_price_id(price_id: &str) -> Option<(String, Option<i32>)> {
    let configs = load_plan_configs();
    configs
        .get(price_id)
        .map(|c| (c.plan_name.clone(), c.usage_limit))
}

// -- Response types --

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct StripeCustomer {
    pub id: String,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CheckoutSession {
    pub id: String,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct PortalSession {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct StripeSubscription {
    pub id: String,
    pub status: String,
    pub cancel_at_period_end: bool,
    pub current_period_start: i64,
    pub current_period_end: i64,
    pub items: StripeList<SubscriptionItem>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SubscriptionItem {
    pub id: String,
    pub price: StripePrice,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct StripePrice {
    pub id: String,
    pub product: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct StripeInvoice {
    pub id: String,
    pub status: Option<String>,
    pub amount_due: i64,
    pub amount_paid: i64,
    pub currency: String,
    pub hosted_invoice_url: Option<String>,
    pub created: i64,
    pub period_start: i64,
    pub period_end: i64,
}

#[derive(Debug, Deserialize)]
pub struct StripeList<T> {
    pub data: Vec<T>,
    pub has_more: bool,
}

#[derive(Debug, Deserialize)]
pub struct StripeEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: StripeEventData,
}

#[derive(Debug, Deserialize)]
pub struct StripeEventData {
    pub object: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct StripeApiError {
    error: StripeApiErrorBody,
}

#[derive(Debug, Deserialize)]
struct StripeApiErrorBody {
    message: String,
}

#[derive(Debug)]
pub enum StripeError {
    #[allow(dead_code)]
    NotConfigured,
    ApiError(String),
    SignatureInvalid,
    SignatureExpired,
    Network(String),
}

impl std::fmt::Display for StripeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StripeError::NotConfigured => write!(f, "Stripe is not configured"),
            StripeError::ApiError(msg) => write!(f, "Stripe API error: {}", msg),
            StripeError::SignatureInvalid => write!(f, "Invalid webhook signature"),
            StripeError::SignatureExpired => write!(f, "Webhook signature expired"),
            StripeError::Network(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

pub struct CreateCheckoutParams {
    pub customer_id: String,
    pub price_id: String,
    pub success_url: String,
    pub cancel_url: String,
    pub trial_days: Option<u32>,
}

async fn parse_response<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T, StripeError> {
    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| StripeError::Network(e.to_string()))?;

    if !status.is_success() {
        let msg = serde_json::from_str::<StripeApiError>(&body)
            .map(|e| e.error.message)
            .unwrap_or_else(|_| format!("HTTP {}: {}", status, body));
        return Err(StripeError::ApiError(msg));
    }

    serde_json::from_str(&body).map_err(|e| StripeError::ApiError(format!("Parse error: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_webhook_signature_valid() {
        let secret = "whsec_test_secret";
        let payload = b"{\"id\":\"evt_test\"}";
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let signed = format!("{}.{}", timestamp, std::str::from_utf8(payload).unwrap());

        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(signed.as_bytes());
        let sig = hex::encode(mac.finalize().into_bytes());

        let header = format!("t={},v1={}", timestamp, sig);
        assert!(verify_webhook_signature(payload, &header, secret, 300).is_ok());
    }

    #[test]
    fn test_verify_webhook_signature_invalid() {
        let payload = b"{\"id\":\"evt_test\"}";
        let header = format!("t={},v1=invalidsignature", chrono::Utc::now().timestamp());
        assert!(matches!(
            verify_webhook_signature(payload, &header, "whsec_test", 300),
            Err(StripeError::SignatureInvalid)
        ));
    }

    #[test]
    fn test_verify_webhook_signature_expired() {
        let secret = "whsec_test_secret";
        let payload = b"{\"id\":\"evt_test\"}";
        let old_ts = (chrono::Utc::now().timestamp() - 600).to_string();
        let signed = format!("{}.{}", old_ts, std::str::from_utf8(payload).unwrap());

        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(signed.as_bytes());
        let sig = hex::encode(mac.finalize().into_bytes());

        let header = format!("t={},v1={}", old_ts, sig);
        assert!(matches!(
            verify_webhook_signature(payload, &header, secret, 300),
            Err(StripeError::SignatureExpired)
        ));
    }

    #[test]
    fn test_plan_mapping() {
        // Without env vars set, should return None
        assert!(plan_from_price_id("price_nonexistent").is_none());
    }

    #[test]
    fn test_verify_missing_timestamp() {
        let payload = b"{\"id\":\"evt_test\"}";
        let header = "v1=abc123";
        assert!(matches!(
            verify_webhook_signature(payload, header, "whsec_test", 300),
            Err(StripeError::SignatureInvalid)
        ));
    }

    #[test]
    fn test_verify_missing_signatures() {
        let payload = b"{\"id\":\"evt_test\"}";
        let header = format!("t={}", chrono::Utc::now().timestamp());
        assert!(matches!(
            verify_webhook_signature(payload, &header, "whsec_test", 300),
            Err(StripeError::SignatureInvalid)
        ));
    }

    #[test]
    fn test_verify_multiple_v1_sigs_one_valid() {
        let secret = "whsec_test_secret";
        let payload = b"{\"id\":\"evt_test\"}";
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let signed = format!("{}.{}", timestamp, std::str::from_utf8(payload).unwrap());

        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(signed.as_bytes());
        let valid_sig = hex::encode(mac.finalize().into_bytes());

        // Header with one invalid and one valid signature
        let header = format!("t={},v1=badbadbadbad,v1={}", timestamp, valid_sig);
        assert!(verify_webhook_signature(payload, &header, secret, 300).is_ok());
    }

    #[test]
    fn test_load_plan_configs_empty() {
        // Without STRIPE_PRICE_BUILDER or STRIPE_PRICE_GROWTH env vars
        std::env::remove_var("STRIPE_PRICE_BUILDER");
        std::env::remove_var("STRIPE_PRICE_GROWTH");
        let configs = load_plan_configs();
        assert!(configs.is_empty());
    }

    #[test]
    fn test_plan_from_price_id_none() {
        assert!(plan_from_price_id("price_does_not_exist").is_none());
    }
}
