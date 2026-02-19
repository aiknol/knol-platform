//! Twitter/X channel adapter — OAuth 1.0a signing + POST /2/tweets.

use hmac::{Hmac, Mac};
use sha1::Sha1;
use std::collections::BTreeMap;
use tracing::{info, warn};

use super::{PublishContent, PublishResult};
use crate::config::ChannelCredentials;
use crate::error::MarketingError;

/// Generate OAuth 1.0a signature for Twitter API v2.
fn oauth_sign(
    method: &str,
    url: &str,
    consumer_key: &str,
    consumer_secret: &str,
    token: &str,
    token_secret: &str,
) -> BTreeMap<String, String> {
    let nonce: String = (0..32)
        .map(|_| format!("{:x}", rand::random::<u8>()))
        .collect();
    let timestamp = chrono::Utc::now().timestamp().to_string();

    let mut params = BTreeMap::new();
    params.insert("oauth_consumer_key".to_string(), consumer_key.to_string());
    params.insert("oauth_nonce".to_string(), nonce);
    params.insert(
        "oauth_signature_method".to_string(),
        "HMAC-SHA1".to_string(),
    );
    params.insert("oauth_timestamp".to_string(), timestamp);
    params.insert("oauth_token".to_string(), token.to_string());
    params.insert("oauth_version".to_string(), "1.0".to_string());

    // Build parameter string
    let param_string: String = params
        .iter()
        .map(|(k, v)| {
            format!(
                "{}={}",
                percent_encoding::utf8_percent_encode(k, percent_encoding::NON_ALPHANUMERIC),
                percent_encoding::utf8_percent_encode(v, percent_encoding::NON_ALPHANUMERIC)
            )
        })
        .collect::<Vec<_>>()
        .join("&");

    // Build base string
    let base_string = format!(
        "{}&{}&{}",
        method.to_uppercase(),
        percent_encoding::utf8_percent_encode(url, percent_encoding::NON_ALPHANUMERIC),
        percent_encoding::utf8_percent_encode(&param_string, percent_encoding::NON_ALPHANUMERIC)
    );

    // Build signing key
    let signing_key = format!(
        "{}&{}",
        percent_encoding::utf8_percent_encode(consumer_secret, percent_encoding::NON_ALPHANUMERIC),
        percent_encoding::utf8_percent_encode(token_secret, percent_encoding::NON_ALPHANUMERIC)
    );

    // HMAC-SHA1 signature
    let mut mac =
        Hmac::<Sha1>::new_from_slice(signing_key.as_bytes()).expect("HMAC accepts any key length");
    mac.update(base_string.as_bytes());
    let signature = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        mac.finalize().into_bytes(),
    );

    params.insert("oauth_signature".to_string(), signature);
    params
}

/// Build OAuth Authorization header from signed params.
fn build_auth_header(params: &BTreeMap<String, String>) -> String {
    let parts: Vec<String> = params
        .iter()
        .map(|(k, v)| {
            format!(
                "{}=\"{}\"",
                percent_encoding::utf8_percent_encode(k, percent_encoding::NON_ALPHANUMERIC),
                percent_encoding::utf8_percent_encode(v, percent_encoding::NON_ALPHANUMERIC)
            )
        })
        .collect();
    format!("OAuth {}", parts.join(", "))
}

pub async fn publish(
    content: &PublishContent,
    http_client: &reqwest::Client,
    credentials: &ChannelCredentials,
) -> Result<PublishResult, MarketingError> {
    let creds = credentials
        .twitter
        .as_ref()
        .ok_or_else(|| MarketingError::Channel {
            channel: "twitter".into(),
            message: "Twitter credentials not configured".into(),
        })?;

    let url = "https://api.twitter.com/2/tweets";
    let oauth_params = oauth_sign(
        "POST",
        url,
        &creds.api_key,
        &creds.api_secret,
        &creds.access_token,
        &creds.access_token_secret,
    );
    let auth_header = build_auth_header(&oauth_params);

    let body = serde_json::json!({ "text": content.text });

    let resp = http_client
        .post(url)
        .header("Authorization", &auth_header)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&body).unwrap())
        .send()
        .await
        .map_err(|e| MarketingError::Channel {
            channel: "twitter".into(),
            message: e.to_string(),
        })?;

    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();

    if status.as_u16() == 201 {
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
        let tweet_id = parsed["data"]["id"].as_str().map(|s| s.to_string());
        let tweet_url = tweet_id
            .as_ref()
            .map(|id| format!("https://twitter.com/i/status/{}", id));

        info!(
            "Twitter: posted tweet {}",
            tweet_id.as_deref().unwrap_or("?")
        );
        Ok(PublishResult::success("twitter", tweet_id, tweet_url))
    } else if status.as_u16() == 429 {
        warn!("Twitter: rate limited (429)");
        Err(MarketingError::RateLimited {
            channel: "twitter".into(),
            current: 0,
            limit: 0,
            window: "api".into(),
        })
    } else {
        warn!(
            "Twitter: HTTP {} — {}",
            status,
            &text[..text.len().min(200)]
        );
        Ok(PublishResult::failure(
            "twitter",
            format!("HTTP {}: {}", status, &text[..text.len().min(200)]),
        ))
    }
}
