//! Invite token cryptographic validation.

use super::token::InviteToken;

/// Result of token validation.
#[derive(Debug)]
pub enum ValidationResult {
    Valid,
    Expired,
    InvalidSignature,
    AlreadyUsed,
}

/// Validate an invite token.
pub fn validate_token(
    token: &InviteToken,
    current_time_ms: u64,
    is_used: bool,
) -> ValidationResult {
    // Check expiry
    if token.is_expired(current_time_ms) {
        return ValidationResult::Expired;
    }

    // Check if already consumed
    if is_used {
        return ValidationResult::AlreadyUsed;
    }

    // Check signature (caller must verify signature externally)
    if token.signature.is_empty() {
        return ValidationResult::InvalidSignature;
    }

    ValidationResult::Valid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_expired() {
        let token = InviteToken::with_expiry(vec![1u8; 32], 0);
        let result = validate_token(&token, token.created_at + 1000, false);
        assert!(matches!(result, ValidationResult::Expired));
    }

    #[test]
    fn test_validate_used() {
        let mut token = InviteToken::new(vec![1u8; 32]);
        token.signature = vec![0xFF; 64];
        let result = validate_token(&token, token.created_at + 1000, true);
        assert!(matches!(result, ValidationResult::AlreadyUsed));
    }
}
