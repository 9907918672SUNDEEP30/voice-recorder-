//! # Vector Store Crate
//!
//! High-performance AES-256-GCM encrypted document vault (SQLite) paired with an
//! in-memory SIMD-accelerated cosine similarity vector index.

pub mod bm25;
pub mod crypto;
pub mod hnsw;
pub mod index;
pub mod vault;

pub use bm25::{reciprocal_rank_fusion, Bm25Index};
pub use crypto::{AesGcmCipher, SessionKey};
pub use hnsw::HnswVectorIndex;
pub use index::VectorIndex;
pub use vault::EncryptedDocumentVault;

