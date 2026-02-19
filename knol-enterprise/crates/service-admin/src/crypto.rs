//! AES-256-GCM encryption for API credentials stored at rest.
//!
//! Key is loaded from ADMIN_ENCRYPTION_KEY (base64-encoded 32-byte key).
//! Each encrypted value has a 12-byte nonce prepended.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use rand::RngCore;

/// Encrypt plaintext using AES-256-GCM.
/// Returns nonce (12 bytes) || ciphertext.
pub fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("AES key error: {}", e))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption error: {}", e))?;

    // Prepend nonce to ciphertext
    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Decrypt ciphertext (nonce || encrypted data) using AES-256-GCM.
pub fn decrypt(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    if data.len() < 12 {
        return Err("Ciphertext too short (missing nonce)".into());
    }

    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("AES key error: {}", e))?;

    let nonce = Nonce::from_slice(&data[..12]);
    let ciphertext = &data[12..];

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption error: {}", e))
}

/// Load encryption key from ADMIN_ENCRYPTION_KEY env var (base64-encoded).
/// Fails closed if the key is missing or invalid.
pub fn load_encryption_key() -> [u8; 32] {
    let b64 = std::env::var("ADMIN_ENCRYPTION_KEY")
        .expect("ADMIN_ENCRYPTION_KEY must be set (base64-encoded 32-byte key)");
    let bytes = B64
        .decode(&b64)
        .expect("ADMIN_ENCRYPTION_KEY must be valid base64");
    if bytes.len() != 32 {
        panic!(
            "ADMIN_ENCRYPTION_KEY must be 32 bytes (got {}). Generate with: openssl rand -base64 32",
            bytes.len()
        );
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let plaintext = b"sk-ant-api03-secret-key-here";

        let encrypted = encrypt(plaintext, &key).unwrap();
        assert_ne!(&encrypted[12..], plaintext); // ciphertext differs

        let decrypted = decrypt(&encrypted, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn decrypt_with_wrong_key_fails() {
        let key1 = [1u8; 32];
        let key2 = [2u8; 32];

        let encrypted = encrypt(b"secret", &key1).unwrap();
        assert!(decrypt(&encrypted, &key2).is_err());
    }

    #[test]
    fn different_encryptions_produce_different_output() {
        let key = [42u8; 32];
        let plaintext = b"same-plaintext";

        let e1 = encrypt(plaintext, &key).unwrap();
        let e2 = encrypt(plaintext, &key).unwrap();
        assert_ne!(e1, e2); // Different nonces → different ciphertext
    }
}
