//! # Volatile Session Key & AES-256-GCM Cipher Module
//!
//! Provides ephemeral in-memory encryption key management with automatic
//! memory zeroization on drop (`zeroize::ZeroizeOnDrop`).

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use anyhow::{anyhow, Result};
use rand::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// 256-bit ephemeral cryptographic key stored strictly in RAM.
/// Implements `ZeroizeOnDrop` so key bytes are wiped with `0x00` when dropped.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct VolatileSessionKey {
    key_bytes: [u8; 32],
}

impl VolatileSessionKey {
    /// Creates a session key from raw 32 bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { key_bytes: bytes }
    }

    /// Creates a session key from a byte slice (must be exactly 32 bytes)
    pub fn from_slice(slice: &[u8]) -> Result<Self> {
        if slice.len() != 32 {
            anyhow::bail!("Session key must be 32 bytes, got {}", slice.len());
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

    /// Internal raw byte accessor
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key_bytes
    }
}

pub struct AesGcmCipher;

impl AesGcmCipher {
    /// Encrypts plaintext bytes using AES-256-GCM with a freshly generated 96-bit nonce
    pub fn encrypt(key: &VolatileSessionKey, plaintext: &[u8]) -> Result<(Vec<u8>, [u8; 12])> {
        let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
            .map_err(|e| anyhow!("Failed to initialize AES-256-GCM cipher: {}", e))?;

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| anyhow!("AES-256-GCM encryption error: {}", e))?;

        Ok((ciphertext, nonce_bytes))
    }

    /// Decrypts ciphertext bytes using AES-256-GCM and the supplied 96-bit nonce
    pub fn decrypt(key: &VolatileSessionKey, ciphertext: &[u8], nonce_bytes: &[u8; 12]) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
            .map_err(|e| anyhow!("Failed to initialize AES-256-GCM cipher: {}", e))?;

        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("AES-256-GCM authentication/decryption failed: {}", e))?;

        Ok(plaintext)
    }
}
