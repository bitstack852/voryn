//! Hardware keystore abstraction layer.
//!
//! Provides a unified interface for Secure Enclave (iOS) and StrongBox (Android).
//! Platform-specific implementations will be added in Phase 2.

/// Handle to a key stored in hardware.
#[derive(Debug, Clone)]
pub struct KeyHandle {
    pub alias: String,
}

/// Abstraction over platform-specific hardware security modules.
pub trait HardwareKeyStore: Send + Sync {
    /// Generate a new device-bound key.
    fn generate_device_key(&self, alias: &str) -> Result<KeyHandle, String>;

    /// Encrypt data using a hardware-bound key.
    fn encrypt(&self, handle: &KeyHandle, plaintext: &[u8]) -> Result<Vec<u8>, String>;

    /// Decrypt data using a hardware-bound key.
    fn decrypt(&self, handle: &KeyHandle, ciphertext: &[u8]) -> Result<Vec<u8>, String>;

    /// Sign data using a hardware-bound key.
    fn sign(&self, handle: &KeyHandle, data: &[u8]) -> Result<Vec<u8>, String>;

    /// Check if a key exists in the hardware store.
    fn key_exists(&self, handle: &KeyHandle) -> bool;

    /// Delete a key from the hardware store.
    fn delete_key(&self, handle: &KeyHandle) -> Result<(), String>;
}

/// Software-based fallback keystore for development and testing.
/// NOT secure for production use — keys are stored in memory.
pub struct SoftwareKeyStore;

impl HardwareKeyStore for SoftwareKeyStore {
    fn generate_device_key(&self, alias: &str) -> Result<KeyHandle, String> {
        Ok(KeyHandle {
            alias: alias.to_string(),
        })
    }

    fn encrypt(&self, _handle: &KeyHandle, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        // Placeholder: no-op passthrough for development
        Ok(plaintext.to_vec())
    }

    fn decrypt(&self, _handle: &KeyHandle, ciphertext: &[u8]) -> Result<Vec<u8>, String> {
        // Placeholder: no-op passthrough for development
        Ok(ciphertext.to_vec())
    }

    fn sign(&self, _handle: &KeyHandle, _data: &[u8]) -> Result<Vec<u8>, String> {
        // Placeholder: returns empty signature for development
        Ok(vec![0u8; 64])
    }

    fn key_exists(&self, _handle: &KeyHandle) -> bool {
        true
    }

    fn delete_key(&self, _handle: &KeyHandle) -> Result<(), String> {
        Ok(())
    }
}
