//! # AES-256-GCM Authenticated Encryption & Session Key Zeroization
//!
//! Provides hardware-accelerated AES-256-GCM authenticated cipher operations with
//! unique 96-bit nonces per chunk and volatile RAM session key wiping (`zeroize`).

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use anyhow::{anyhow, Result};
use rand::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// 256-bit Volatile Encryption Key stored only in RAM
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SessionKey {
    key_bytes: [u8; 32],
}

impl SessionKey {
    /// Creates a session key from raw 32 bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { key_bytes: bytes }
    }

    /// Creates a session key from a byte slice (must be 32 bytes)
    pub fn from_slice(slice: &[u8]) -> Result<Self> {
        if slice.len() != 32 {
            anyhow::bail!("SessionKey slice must be exactly 32 bytes, got {}", slice.len());
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(slice);
        Ok(Self { key_bytes: bytes })
    }

    /// Generates a cryptographically secure random session key
    pub fn generate_random() -> Self {
        let mut key_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key_bytes);
        Self { key_bytes }
    }

    /// Raw byte access (internal use)
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key_bytes
    }
}

pub struct AesGcmCipher;

impl AesGcmCipher {
    /// Encrypts plaintext bytes using AES-256-GCM with a freshly generated 96-bit nonce
    pub fn encrypt(key: &SessionKey, plaintext: &[u8]) -> Result<(Vec<u8>, [u8; 12])> {
        let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
            .map_err(|e| anyhow!("Failed to initialize AES-256-GCM cipher: {}", e))?;

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| anyhow!("AES-256-GCM encryption failed: {}", e))?;

        Ok((ciphertext, nonce_bytes))
    }

    /// Decrypts ciphertext bytes using AES-256-GCM and the supplied 96-bit nonce
    pub fn decrypt(key: &SessionKey, ciphertext: &[u8], nonce_bytes: &[u8; 12]) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
            .map_err(|e| anyhow!("Failed to initialize AES-256-GCM cipher: {}", e))?;

        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("AES-256-GCM decryption failed or invalid authentication tag: {}", e))?;

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_gcm_encryption_roundtrip() {
        let key = SessionKey::generate_random();
        let message = b"Confidential invoice data: Total INR 2,50,000 for Customer Sharma.";

        let (ciphertext, nonce) = AesGcmCipher::encrypt(&key, message).expect("Encryption failed");
        assert_ne!(ciphertext, message);

        let decrypted = AesGcmCipher::decrypt(&key, &ciphertext, &nonce).expect("Decryption failed");
        assert_eq!(decrypted, message);
    }

    #[test]
    fn test_wrong_key_fails_decryption() {
        let key1 = SessionKey::generate_random();
        let key2 = SessionKey::generate_random();
        let message = b"Secret patient prescription notes";

        let (ciphertext, nonce) = AesGcmCipher::encrypt(&key1, message).unwrap();
        let result = AesGcmCipher::decrypt(&key2, &ciphertext, &nonce);
        assert!(result.is_err(), "Decryption must fail with an incorrect session key");
    }
}
