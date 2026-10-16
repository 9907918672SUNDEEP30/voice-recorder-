//! # Local RAG Engine Coordinator
//!
//! Integrates Document Parsing, Local BGE Vector Embedding, SQLite AES-256 Vault,
//! In-Memory SIMD Cosine Search, Local Qwen LLM Reasoning, DRM Lease Guard, and CRM Intelligence.

use super::crm::CrmManager;
use super::drm::{DrmGuard, DrmInfo};
use anyhow::{anyhow, Context, Result};
use core_engine::{EmbeddingEngine, LlmEngine};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use text_parser::DocumentParser;
use vector_store::{EncryptedDocumentVault, VectorIndex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestResult {
    pub doc_id: String,
    pub filename: String,
    pub category: String,
    pub chunks_created: usize,
    pub total_words: usize,
    pub duration_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub chunk_id: String,
    pub doc_id: String,
    pub category: String,
    pub page_number: Option<u32>,
    pub score: f32,
    pub content_preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub query: String,
    pub answer: String,
    pub citations: Vec<Citation>,
    pub latency_ms: u128,
    pub drm_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStats {
    pub total_encrypted_chunks: usize,
    pub indexed_vectors_in_ram: usize,
    pub drm_info: DrmInfo,
}

#[derive(Clone)]
pub struct LocalRagEngine {
    parser: Arc<DocumentParser>,
    vault: Arc<EncryptedDocumentVault>,
    index: Arc<VectorIndex>,
    embedding: Arc<EmbeddingEngine>,
    llm: Arc<LlmEngine>,
    drm: Arc<DrmGuard>,
    crm: CrmManager,
}

impl LocalRagEngine {
    pub fn new<P: AsRef<Path>>(
        db_path: P,
        models_dir: Option<&Path>,
        drm: Arc<DrmGuard>,
        crm: CrmManager,
    ) -> Result<Self> {
        let vault = EncryptedDocumentVault::open(db_path)
            .context("Failed to open encrypted document vault")?;

        let index = VectorIndex::new(384);
        let parser = DocumentParser::default();
        let embedding = EmbeddingEngine::new(models_dir);
        let llm = LlmEngine::new(models_dir);

        Ok(Self {
            parser: Arc::new(parser),
            vault: Arc::new(vault),
            index: Arc::new(index),
            embedding: Arc::new(embedding),
            llm: Arc::new(llm),
            drm,
            crm,
        })
    }

    pub fn crm(&self) -> &CrmManager {
        &self.crm
    }

    pub fn drm(&self) -> &Arc<DrmGuard> {
        &self.drm
    }

    /// Ingests a raw file (PDF, CSV, or Text) with automatic type detection and encryption.
    pub async fn ingest_file(
        &self,
        doc_id: &str,
        filename: &str,
        bytes: &[u8],
        category: &str,
        customer_id: Option<&str>,
    ) -> Result<IngestResult> {
        let start = Instant::now();

        // 1. Verify DRM license & retrieve ephemeral session key
        let session_key = self
            .drm
            .get_session_key()
            .await
            .ok_or_else(|| anyhow!("DRM lease is expired or inactive. Document vault is locked."))?;

        // 2. Parse file based on extension
        let lower = filename.to_lowercase();
        let chunks = if lower.ends_with(".pdf") {
            self.parser.parse_pdf_bytes(doc_id, bytes, category)?
        } else if lower.ends_with(".csv") {
            let csv_str = String::from_utf8_lossy(bytes);
            self.parser.parse_csv_str(doc_id, &csv_str, category)?
        } else {
            let text_str = String::from_utf8_lossy(bytes);
            self.parser.parse_plain_text(doc_id, &text_str, category)
        };

        if chunks.is_empty() {
            return Err(anyhow!("No parseable text extracted from document '{}'", filename));
        }

        let total_words: usize = chunks.iter().map(|c| c.word_count).sum();

        // 3. Generate embeddings
        let chunk_texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let embeddings = self.embedding.embed_batch(&chunk_texts).await?;

        // 4. Store encrypted in SQLite vault
        self.vault.store_chunks(&session_key, &chunks)?;

        // 5. Index into in-memory vector index
        let mut index_items = Vec::with_capacity(chunks.len());
        for (chunk, vector) in chunks.iter().zip(embeddings.into_iter()) {
            index_items.push((chunk.chunk_id.clone(), vector));
        }
        self.index.add_vectors(index_items);

        // 6. Associate with CRM customer if specified
        if let Some(c_id) = customer_id {
            self.crm.associate_document(c_id, doc_id).await;
            self.crm
                .add_note(
                    c_id,
                    "Automated Ingestion",
                    &format!("Associated document '{}' ({}) with {} chunks.", filename, doc_id, chunks.len()),
                )
                .await;
        }

        let duration_ms = start.elapsed().as_millis();
        Ok(IngestResult {
            doc_id: doc_id.to_string(),
            filename: filename.to_string(),
            category: category.to_string(),
            chunks_created: chunks.len(),
            total_words,
            duration_ms,
        })
    }

    /// Ingests plain text directly
    pub async fn ingest_text(
        &self,
        doc_id: &str,
        text: &str,
        category: &str,
        customer_id: Option<&str>,
    ) -> Result<IngestResult> {
        self.ingest_file(doc_id, "direct_input.txt", text.as_bytes(), category, customer_id).await
    }

    /// Executes RAG search: Cosine Top-K Retrieval + Decryption + Grounded LLM Synthesis.
    pub async fn query(&self, prompt: &str, top_k: usize) -> Result<QueryResult> {
        let start = Instant::now();

        // 1. Verify DRM license
        let session_key = self
            .drm
            .get_session_key()
            .await
            .ok_or_else(|| anyhow!("DRM lease is inactive or expired. Search index locked."))?;

        // 2. Embed user query
        let query_vec = self.embedding.embed_text(prompt).await?;

        // 3. In-memory SIMD Top-K search
        let k = if top_k == 0 { 5 } else { top_k };
        let matches = self.index.search_top_k(&query_vec, k);

        // 4. Decrypt matched chunks from SQLite vault
        let mut citations = Vec::new();
        let mut contexts = Vec::new();

        for (chunk_id, score) in matches {
            match self.vault.get_chunk(&session_key, &chunk_id) {
                Ok(chunk) => {
                    contexts.push(chunk.content.clone());
                    let preview = if chunk.content.chars().count() > 180 {
                        format!("{}...", chunk.content.chars().take(180).collect::<String>())
                    } else {
                        chunk.content.clone()
                    };

                    citations.push(Citation {
                        chunk_id: chunk.chunk_id,
                        doc_id: chunk.doc_id,
                        category: chunk.category,
                        page_number: chunk.page_number,
                        score,
                        content_preview: preview,
                    });
                }
                Err(e) => {
                    eprintln!("Vault decryption warning for chunk {}: {}", chunk_id, e);
                }
            }
        }

        // 5. Generate LLM synthesis grounded in retrieved context
        let answer = self.llm.generate_rag_answer(prompt, &contexts).await?;
        let latency_ms = start.elapsed().as_millis();

        Ok(QueryResult {
            query: prompt.to_string(),
            answer,
            citations,
            latency_ms,
            drm_valid: true,
        })
    }

    /// Returns engine operational statistics
    pub async fn get_stats(&self) -> EngineStats {
        let total_chunks = self.vault.count_chunks().unwrap_or(0);
        let total_indexed = self.index.count();
        let drm_info = self.drm.get_status_info().await;

        EngineStats {
            total_encrypted_chunks: total_chunks,
            indexed_vectors_in_ram: total_indexed,
            drm_info,
        }
    }

    /// Lists all unique document IDs
    pub fn list_documents(&self) -> Result<Vec<String>> {
        self.vault.list_document_ids()
    }
}
