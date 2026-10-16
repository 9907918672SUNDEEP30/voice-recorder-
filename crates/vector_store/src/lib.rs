//! # Vector Store Crate
//!
//! High-performance AES-256-GCM encrypted document vault (SQLite) paired with an
//! in-memory SIMD-accelerated cosine similarity vector index.

pub mod crypto;
pub mod index;
pub mod vault;

pub use crypto::{AesGcmCipher, SessionKey};
pub use index::VectorIndex;
pub use vault::EncryptedDocumentVault;
