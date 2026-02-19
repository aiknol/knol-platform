//! Encryption helpers for webhook secrets at rest.
//!
//! Secrets are stored in the database with an `enc:` prefix followed by
//! base64-encoded AES-256-GCM ciphertext (nonce || ciphertext). Secrets
//! without the `enc:` prefix are treated as plaintext (backward compatible).

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use rand::RngCore;

/// Prefix used to identify encrypted secrets in the database.
pub const ENCRYPTED_PREFIX: &str = "enc:";

/// Encrypt a plaintext secret for database storage.
/// Returns a string with the `enc:` prefix followed by base64-encoded ciphertext.
pub fn encrypt_secret(plaintext: &str, key: &[u8; 32]) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("AES key error: {}", e))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encrypt error: {}", e))?;

    let mut blob = Vec::with_capacity(12 + ciphertext.len());
    blob.extend_from_slice(&nonce_bytes);
    blob.extend_from_slice(&ciphertext);

    Ok(format!("{}{}", ENCRYPTED_PREFIX, B64.encode(&blob)))
}

/// Decrypt a stored secret. Handles both encrypted (`enc:...`) and plaintext values.
/// Returns the plaintext secret in either case.
pub fn decrypt_secret(stored: &str, key: &[u8; 32]) -> Result<String, String> {
    if let Some(encoded) = stored.strip_prefix(ENCRYPTED_PREFIX) {
        let data = B64
            .decode(encoded)
            .map_err(|e| format!("Base64 decode error: {}", e))?;

        if data.len() < 12 {
            return Err("Ciphertext too short (need at least 12-byte nonce)".into());
        }

        let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("AES key error: {}", e))?;
        let nonce = Nonce::from_slice(&data[..12]);
        let plaintext = cipher
            .decrypt(nonce, &data[12..])
            .map_err(|e| format!("Decrypt error: {}", e))?;

        String::from_utf8(plaintext).map_err(|e| format!("UTF-8 error: {}", e))
    } else {
        // Plaintext (legacy / no encryption key configured at creation time)
        Ok(stored.to_string())
    }
}

/// Decrypt an optional secret, preserving None values.
pub fn decrypt_secret_opt(stored: Option<&str>, key: &[u8; 32]) -> Option<String> {
    stored.map(|s| {
        decrypt_secret(s, key).unwrap_or_else(|e| {
            tracing::warn!("Failed to decrypt webhook secret: {}", e);
            // Return the raw value as fallback so HMAC can still be attempted
            s.to_string()
        })
    })
}

/// Load the webhook encryption key from environment.
/// Tries WEBHOOK_ENCRYPTION_KEY, then ADMIN_ENCRYPTION_KEY.
pub fn load_encryption_key_from_env() -> Option<[u8; 32]> {
    let b64 = std::env::var("WEBHOOK_ENCRYPTION_KEY")
        .or_else(|_| std::env::var("ADMIN_ENCRYPTION_KEY"))
        .ok()?;

    if b64.is_empty() {
        return None;
    }

    let bytes = B64.decode(&b64).ok()?;
    if bytes.len() != 32 {
        tracing::warn!(
            "Webhook encryption key must be 32 bytes (got {})",
            bytes.len()
        );
        return None;
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Some(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let secret = "whsec_my-webhook-secret-123";
        let encrypted = encrypt_secret(secret, &key).unwrap();
        assert!(encrypted.starts_with(ENCRYPTED_PREFIX));
        let decrypted = decrypt_secret(&encrypted, &key).unwrap();
        assert_eq!(decrypted, secret);
    }

    #[test]
    fn test_decrypt_plaintext_passthrough() {
        let key = [42u8; 32];
        let plaintext = "plain-secret-no-prefix";
        let result = decrypt_secret(plaintext, &key).unwrap();
        assert_eq!(result, plaintext);
    }

    #[test]
    fn test_encrypt_produces_unique_ciphertexts() {
        let key = [42u8; 32];
        let a = encrypt_secret("same", &key).unwrap();
        let b = encrypt_secret("same", &key).unwrap();
        assert_ne!(a, b); // Different nonces
        assert_eq!(decrypt_secret(&a, &key).unwrap(), "same");
        assert_eq!(decrypt_secret(&b, &key).unwrap(), "same");
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        let key = [42u8; 32];
        let wrong = [99u8; 32];
        let encrypted = encrypt_secret("secret", &key).unwrap();
        assert!(decrypt_secret(&encrypted, &wrong).is_err());
    }

    #[test]
    fn test_decrypt_secret_opt_none() {
        let key = [42u8; 32];
        assert!(decrypt_secret_opt(None, &key).is_none());
    }

    #[test]
    fn test_decrypt_secret_opt_some() {
        let key = [42u8; 32];
        let enc = encrypt_secret("test", &key).unwrap();
        let result = decrypt_secret_opt(Some(&enc), &key);
        assert_eq!(result, Some("test".to_string()));
    }
}
