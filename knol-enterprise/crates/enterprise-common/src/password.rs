//! Password complexity validation shared between admin and tenant services.

/// Enforce password complexity: >= 12 chars, <= 128, uppercase, lowercase, digit, special.
pub fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 12 {
        return Err("Password must be at least 12 characters".into());
    }
    if password.len() > 128 {
        return Err("Password must not exceed 128 characters".into());
    }
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());
    if !has_upper || !has_lower || !has_digit || !has_special {
        return Err(
            "Password must include uppercase, lowercase, digit, and special character".into(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_password() {
        assert!(validate_password("MyP@ssw0rd!23").is_ok());
    }

    #[test]
    fn test_too_short() {
        assert!(validate_password("Short1!").is_err());
    }

    #[test]
    fn test_missing_special() {
        assert!(validate_password("NoSpecialChar1A").is_err());
    }

    #[test]
    fn test_missing_digit() {
        assert!(validate_password("NoDigitHere!AB").is_err());
    }

    #[test]
    fn test_missing_uppercase() {
        assert!(validate_password("abcdefgh1234!").is_err());
    }

    #[test]
    fn test_missing_lowercase() {
        assert!(validate_password("ABCDEFGH1234!").is_err());
    }

    #[test]
    fn test_exactly_12_chars_passes() {
        // Exactly 12 chars with all requirements
        assert!(validate_password("Abcdefgh12!x").is_ok());
    }

    #[test]
    fn test_11_chars_fails() {
        // 11 chars — should fail on length
        assert!(validate_password("Abcdefg12!x").is_err());
    }
}
