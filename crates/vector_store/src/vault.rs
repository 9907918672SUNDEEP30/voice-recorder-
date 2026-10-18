//! # Encrypted SQLite Document Vault
//!
//! Stores document chunks and metadata on disk with AES-256-GCM authenticated encryption.
//! Zero plaintext stored at rest. Requires a valid volatile `SessionKey` to decrypt.

use crate::crypto::{AesGcmCipher, SessionKey};
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::{Arc, Mutex};
use text_parser::DocumentChunk;

pub struct EncryptedDocumentVault {
    conn: Arc<Mutex<Connection>>,
}

impl EncryptedDocumentVault {
    /// Computes a one-way blind SHA-256 index hash of a document ID to allow grouping and deletion
    /// without exposing plaintext document names in the SQLite database.
    pub fn compute_doc_hash(doc_id: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(doc_id.as_bytes());
        let hash = hasher.finalize();
        hash.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Opens or creates an encrypted SQLite document database file
    pub fn open<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)
            .context("Failed to open or create SQLite document database")?;

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS encrypted_chunks (
                chunk_id TEXT PRIMARY KEY,
                doc_id_hash TEXT NOT NULL,
                nonce BLOB NOT NULL,
                ciphertext BLOB NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_doc_id_hash ON encrypted_chunks(doc_id_hash);
            "#,
        )
        .context("Failed to initialize encrypted_chunks schema")?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Creates an in-memory encrypted SQLite document vault (ideal for testing / temp caches)
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .context("Failed to open in-memory SQLite database")?;

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS encrypted_chunks (
                chunk_id TEXT PRIMARY KEY,
                doc_id_hash TEXT NOT NULL,
                nonce BLOB NOT NULL,
                ciphertext BLOB NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_doc_id_hash ON encrypted_chunks(doc_id_hash);
            "#,
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Stores a document chunk after encrypting it with the volatile session key.
    /// All metadata (`doc_id`, `category`, `chunk_index`, `page_number`) is encrypted
    /// inside the AES-256-GCM ciphertext payload.
    pub fn store_chunk(&self, key: &SessionKey, chunk: &DocumentChunk) -> Result<()> {
        let json_bytes = serde_json::to_vec(chunk)
            .context("Failed to serialize DocumentChunk to JSON")?;

        let (ciphertext, nonce) = AesGcmCipher::encrypt(key, &json_bytes)
            .context("Failed to encrypt DocumentChunk with AES-256-GCM")?;

        let doc_hash = Self::compute_doc_hash(&chunk.doc_id);

        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT OR REPLACE INTO encrypted_chunks
            (chunk_id, doc_id_hash, nonce, ciphertext)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![
                chunk.chunk_id,
                doc_hash,
                nonce.to_vec(),
                ciphertext
            ],
        )
        .context("Failed to insert encrypted chunk into SQLite vault")?;

        Ok(())
    }

    /// Batch stores a slice of document chunks
    pub fn store_chunks(&self, key: &SessionKey, chunks: &[DocumentChunk]) -> Result<usize> {
        for chunk in chunks {
            self.store_chunk(key, chunk)?;
        }
        Ok(chunks.len())
    }

    /// Retrieves and decrypts a document chunk using the volatile session key
    pub fn get_chunk(&self, key: &SessionKey, chunk_id: &str) -> Result<DocumentChunk> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT nonce, ciphertext FROM encrypted_chunks WHERE chunk_id = ?1")
            .context("Failed to prepare chunk query statement")?;

        let (nonce_blob, ciphertext): (Vec<u8>, Vec<u8>) = stmt
            .query_row(params![chunk_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .context(format!("Encrypted chunk with ID '{}' not found", chunk_id))?;

        if nonce_blob.len() != 12 {
            anyhow::bail!("Corrupted nonce in SQLite database for chunk '{}'", chunk_id);
        }

        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&nonce_blob);

        let plaintext = AesGcmCipher::decrypt(key, &ciphertext, &nonce)
            .context("Failed to decrypt chunk with current session key (invalid key or tampered record)")?;

        let chunk: DocumentChunk = serde_json::from_slice(&plaintext)
            .context("Failed to deserialize decrypted DocumentChunk")?;

        Ok(chunk)
    }

    /// Returns count of total stored encrypted chunks
    pub fn count_chunks(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM encrypted_chunks", [], |row| row.get(0))
            .context("Failed to count chunks in SQLite vault")?;
        Ok(count as usize)
    }

    /// Lists all unique document IDs stored in the vault by decrypting metadata with session key
    pub fn list_document_ids(&self, key: &SessionKey) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT nonce, ciphertext FROM encrypted_chunks")
            .context("Failed to prepare document listing statement")?;

        let rows = stmt
            .query_map([], |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?)))
            .context("Failed to query document chunks")?;

        let mut seen = std::collections::HashSet::new();
        let mut doc_ids = Vec::new();

        for r in rows {
            let (nonce_blob, ciphertext) = r?;
            if nonce_blob.len() == 12 {
                let mut nonce = [0u8; 12];
                nonce.copy_from_slice(&nonce_blob);
                if let Ok(plaintext) = AesGcmCipher::decrypt(key, &ciphertext, &nonce) {
                    if let Ok(chunk) = serde_json::from_slice::<DocumentChunk>(&plaintext) {
                        if seen.insert(chunk.doc_id.clone()) {
                            doc_ids.push(chunk.doc_id);
                        }
                    }
                }
            }
        }
        Ok(doc_ids)
    }

    /// Deletes all chunks associated with a specific document ID using blind hash
    pub fn delete_document(&self, doc_id: &str) -> Result<usize> {
        let doc_hash = Self::compute_doc_hash(doc_id);
        let conn = self.conn.lock().unwrap();
        let affected = conn
            .execute("DELETE FROM encrypted_chunks WHERE doc_id_hash = ?1", params![doc_hash])
            .context(format!("Failed to delete document '{}' from SQLite vault", doc_id))?;
        Ok(affected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypted_vault_store_and_retrieve() {
        let vault = EncryptedDocumentVault::in_memory().unwrap();
        let key = SessionKey::generate_random();

        let chunk = DocumentChunk {
            chunk_id: "chk_001".to_string(),
            doc_id: "doc_tax_2024".to_string(),
            chunk_index: 0,
            content: "GST Notice Ref No 4921/2024: Input tax credit reversal confirmed.".to_string(),
            word_count: 10,
            page_number: Some(1),
            category: "Tax_Notice".to_string(),
            metadata_tags: vec!["GST".to_string()],
        };

        vault.store_chunk(&key, &chunk).unwrap();
        assert_eq!(vault.count_chunks().unwrap(), 1);

        // Retrieve with correct key
        let retrieved = vault.get_chunk(&key, "chk_001").unwrap();
        assert_eq!(retrieved.content, chunk.content);

        // Retrieve with wrong key must fail
        let wrong_key = SessionKey::generate_random();
        let fail_result = vault.get_chunk(&wrong_key, "chk_001");
        assert!(fail_result.is_err());
    }
}
