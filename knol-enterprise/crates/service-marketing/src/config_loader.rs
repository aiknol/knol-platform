//! Config loader with DB → env → default fallback chain.
//!
//! Loads credentials from `system_credentials` (AES-256-GCM encrypted)
//! and settings from `system_config`, falling back to env vars and
//! compiled defaults when DB values are not available.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use tracing::{info, warn};

use crate::config::{ChannelCredentials, TwitterCredentials};

/// Load the AES-256 encryption key (same logic as service-admin).
fn load_encryption_key() -> [u8; 32] {
    let b64 = std::env::var("ADMIN_ENCRYPTION_KEY")
        .expect("ADMIN_ENCRYPTION_KEY is required and must be set");
    let bytes = B64
        .decode(&b64)
        .expect("ADMIN_ENCRYPTION_KEY must be valid base64");
    if bytes.len() != 32 {
        panic!(
            "ADMIN_ENCRYPTION_KEY must be 32 bytes (got {})",
            bytes.len()
        );
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    key
}

/// Decrypt a value from system_credentials.
fn decrypt(data: &[u8], key: &[u8; 32]) -> Result<String, String> {
    if data.len() < 12 {
        return Err("Ciphertext too short".into());
    }
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| format!("AES key error: {}", e))?;
    let nonce = Nonce::from_slice(&data[..12]);
    let plaintext = cipher
        .decrypt(nonce, &data[12..])
        .map_err(|e| format!("Decrypt error: {}", e))?;
    String::from_utf8(plaintext).map_err(|e| format!("UTF-8 error: {}", e))
}

/// Row from system_credentials table.
#[derive(sqlx::FromRow)]
struct CredRow {
    name: String,
    encrypted_value: Vec<u8>,
}

/// Row from system_config table.
#[derive(sqlx::FromRow)]
struct ConfigRow {
    key: String,
    value: serde_json::Value,
}

/// Load a single credential: DB (encrypted) → env var → None.
async fn load_credential(
    pool: &sqlx::PgPool,
    key: &[u8; 32],
    db_name: &str,
    env_name: &str,
) -> Option<String> {
    // Try DB first
    if let Ok(row) = sqlx::query_as::<_, CredRow>(
        "SELECT name, encrypted_value FROM system_credentials WHERE name = $1",
    )
    .bind(db_name)
    .fetch_one(pool)
    .await
    {
        match decrypt(&row.encrypted_value, key) {
            Ok(val) if !val.is_empty() => return Some(val),
            Ok(_) => {}
            Err(e) => warn!("Failed to decrypt credential '{}': {}", db_name, e),
        }
    }

    // Fallback to env var
    std::env::var(env_name).ok().filter(|s| !s.is_empty())
}

/// Load a single config value: DB → env var → default.
async fn load_config_value(
    pool: &sqlx::PgPool,
    db_key: &str,
    env_name: &str,
    default: &str,
) -> String {
    // Try DB first
    if let Ok(row) = sqlx::query_as::<_, ConfigRow>(
        "SELECT key, value FROM system_config WHERE key = $1",
    )
    .bind(db_key)
    .fetch_one(pool)
    .await
    {
        // Check if env override is set
        if let Ok(env_val) = std::env::var(env_name) {
            if !env_val.is_empty() {
                return env_val;
            }
        }
        // Use DB value (strip JSON string quotes)
        if let Some(s) = row.value.as_str() {
            return s.to_string();
        }
        return row.value.to_string();
    }

    // Fallback to env var
    if let Ok(val) = std::env::var(env_name) {
        if !val.is_empty() {
            return val;
        }
    }

    default.to_string()
}

/// Load all channel credentials with DB → env fallback.
pub async fn load_credentials(pool: &sqlx::PgPool) -> ChannelCredentials {
    let key = load_encryption_key();

    // Load all credentials in parallel-ish fashion
    let twitter_key = load_credential(pool, &key, "twitter.api_key", "TWITTER_API_KEY").await;
    let twitter_secret =
        load_credential(pool, &key, "twitter.api_secret", "TWITTER_API_SECRET").await;
    let twitter_token =
        load_credential(pool, &key, "twitter.access_token", "TWITTER_ACCESS_TOKEN").await;
    let twitter_token_secret = load_credential(
        pool,
        &key,
        "twitter.access_token_secret",
        "TWITTER_ACCESS_TOKEN_SECRET",
    )
    .await;

    let twitter = match (twitter_key, twitter_secret, twitter_token, twitter_token_secret) {
        (Some(k), Some(s), Some(t), Some(ts)) if !k.is_empty() => {
            info!("Loaded Twitter credentials from config store");
            Some(TwitterCredentials {
                api_key: k,
                api_secret: s,
                access_token: t,
                access_token_secret: ts,
            })
        }
        _ => None,
    };

    let linkedin_token =
        load_credential(pool, &key, "linkedin.access_token", "LINKEDIN_ACCESS_TOKEN").await;
    let linkedin_person_urn =
        load_credential(pool, &key, "linkedin.person_urn", "LINKEDIN_PERSON_URN").await;

    let reddit_client_id =
        load_credential(pool, &key, "reddit.client_id", "REDDIT_CLIENT_ID").await;
    let reddit_client_secret =
        load_credential(pool, &key, "reddit.client_secret", "REDDIT_CLIENT_SECRET").await;
    let reddit_username =
        load_credential(pool, &key, "reddit.username", "REDDIT_USERNAME").await;
    let reddit_password =
        load_credential(pool, &key, "reddit.password", "REDDIT_PASSWORD").await;

    let devto_api_key =
        load_credential(pool, &key, "devto.api_key", "DEVTO_API_KEY").await;
    let github_token =
        load_credential(pool, &key, "github.token", "GITHUB_TOKEN").await;

    let smtp_host = load_credential(pool, &key, "smtp.host", "SMTP_HOST").await;
    let smtp_user = load_credential(pool, &key, "smtp.user", "SMTP_USER").await;
    let smtp_pass = load_credential(pool, &key, "smtp.pass", "SMTP_PASS").await;

    let smtp_port = load_config_value(pool, "marketing.smtp_port", "SMTP_PORT", "587")
        .await
        .parse()
        .unwrap_or(587);

    let anthropic_api_key =
        load_credential(pool, &key, "anthropic.api_key", "ANTHROPIC_API_KEY").await;

    let loaded_from_db = [
        twitter.is_some(),
        linkedin_token.is_some(),
        reddit_client_id.is_some(),
        devto_api_key.is_some(),
        github_token.is_some(),
        smtp_host.is_some(),
        anthropic_api_key.is_some(),
    ]
    .iter()
    .filter(|x| **x)
    .count();

    info!(
        "Credentials loaded: {}/7 channels configured",
        loaded_from_db
    );

    ChannelCredentials {
        twitter,
        linkedin_token,
        linkedin_person_urn,
        reddit_client_id,
        reddit_client_secret,
        reddit_username,
        reddit_password,
        devto_api_key,
        github_token,
        smtp_host,
        smtp_port,
        smtp_user,
        smtp_pass,
        anthropic_api_key,
    }
}

/// Load channel-specific rate limit overrides from system_config.
pub async fn load_rate_limit_override(
    pool: &sqlx::PgPool,
    channel: &str,
) -> Option<u64> {
    let key = format!("marketing.rate_limit.{}.daily", channel);
    if let Ok(row) = sqlx::query_as::<_, ConfigRow>(
        "SELECT key, value FROM system_config WHERE key = $1",
    )
    .bind(&key)
    .fetch_one(pool)
    .await
    {
        row.value.as_u64()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_encryption_key_requires_env() {
        std::env::remove_var("ADMIN_ENCRYPTION_KEY");
        let result = std::panic::catch_unwind(load_encryption_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_roundtrip() {
        use aes_gcm::aead::Aead;
        use rand::RngCore;

        let key = [42u8; 32];
        let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, b"test-secret".as_slice()).unwrap();

        let mut data = Vec::new();
        data.extend_from_slice(&nonce_bytes);
        data.extend_from_slice(&ciphertext);

        let result = decrypt(&data, &key).unwrap();
        assert_eq!(result, "test-secret");
    }
}
