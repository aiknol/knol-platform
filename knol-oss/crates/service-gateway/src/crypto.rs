//! Re-export encryption helpers from memory-common for local convenience.
//! This module also contains gateway-specific helpers like secret masking.

pub use memory_common::webhook_crypto::{decrypt_secret, encrypt_secret, ENCRYPTED_PREFIX};

/// Mask a secret for display: show first 4 chars then asterisks.
pub fn mask_secret(secret: &str) -> String {
    if secret.len() <= 4 {
        return "****".to_string();
    }
    format!("{}****", &secret[..4])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let plaintext = "my-webhook-secret-123";
        let encrypted = encrypt_secret(plaintext, &key).unwrap();
        assert_ne!(encrypted, plaintext);
        let decrypted = decrypt_secret(&encrypted, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_mask_secret() {
        assert_eq!(mask_secret("abcdefgh"), "abcd****");
        assert_eq!(mask_secret("ab"), "****");
        assert_eq!(mask_secret(""), "****");
    }
}
