//! Secure memory types — auto-zeroing containers for sensitive data.

use zeroize::{Zeroize, ZeroizeOnDrop};

/// A byte buffer that is zeroed on drop. Use for any sensitive data
/// (plaintext messages, key material, passwords).
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecureBytes {
    inner: Vec<u8>,
}

impl SecureBytes {
    pub fn new(data: Vec<u8>) -> Self {
        Self { inner: data }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

/// A string that is zeroed on drop. Use for passcodes, display names, etc.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecureString {
    inner: String,
}

impl SecureString {
    pub fn new(data: String) -> Self {
        Self { inner: data }
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_bytes_access() {
        let data = vec![1, 2, 3, 4];
        let secure = SecureBytes::new(data.clone());
        assert_eq!(secure.as_bytes(), &data);
        assert_eq!(secure.len(), 4);
        assert!(!secure.is_empty());
    }

    #[test]
    fn test_secure_string_access() {
        let secure = SecureString::new("secret".to_string());
        assert_eq!(secure.as_str(), "secret");
    }
}
