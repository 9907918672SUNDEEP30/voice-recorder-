//! # Local RAG Engine Coordinator
//!
//! Integrates Document Parsing, Local BGE Vector Embedding, SQLite AES-256 Vault,
//! In-Memory SIMD Cosine Search, Local Qwen LLM Reasoning, DRM Lease Guard, and CRM Intelligence.

use super::crm::CrmManager;
use super::drm::{DrmGuard, DrmInfo};
use anyhow::{anyhow, Context, Result};
use core_engine::{EmbeddingEngine, LlmEngine};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use text_parser::DocumentParser;
use vector_store::{
    reciprocal_rank_fusion, Bm25Index, EncryptedDocumentVault, HnswVectorIndex, VectorIndex,
};

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
    #[serde(default = "default_retrieval_mode")]
    pub retrieval_mode: String,
}

fn default_retrieval_mode() -> String {
    "Contextual Hybrid (Dense HNSW + Sparse BM25 + RRF)".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStats {
    pub total_encrypted_chunks: usize,
    pub indexed_vectors_in_ram: usize,
    #[serde(default)]
    pub indexed_bm25_docs: usize,
    pub drm_info: DrmInfo,
}

#[derive(Clone)]
pub struct LocalRagEngine {
    parser: Arc<DocumentParser>,
    vault: Arc<EncryptedDocumentVault>,
    index: Arc<VectorIndex>,
    hnsw_index: Arc<HnswVectorIndex>,
    bm25_index: Arc<Bm25Index>,
    embedding: Arc<EmbeddingEngine>,
    llm: Arc<LlmEngine>,
    drm: Arc<DrmGuard>,
    crm: CrmManager,
    ingested_files: Arc<RwLock<HashMap<String, u64>>>,
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
        let hnsw_index = HnswVectorIndex::new(384);
        let bm25_index = Bm25Index::default();
        let parser = DocumentParser::default();
        let embedding = EmbeddingEngine::new(models_dir);
        let llm = LlmEngine::new(models_dir);

        Ok(Self {
            parser: Arc::new(parser),
            vault: Arc::new(vault),
            index: Arc::new(index),
            hnsw_index: Arc::new(hnsw_index),
            bm25_index: Arc::new(bm25_index),
            embedding: Arc::new(embedding),
            llm: Arc::new(llm),
            drm,
            crm,
            ingested_files: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub fn crm(&self) -> &CrmManager {
        &self.crm
    }

    pub fn drm(&self) -> &Arc<DrmGuard> {
        &self.drm
    }

    /// Ingests a raw file (PDF, CSV, or Text) with Contextual Prepending (Anthropic 2026 standard)
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

        // 3. Contextual Prepending (Anthropic Methodology):
        // Each chunk embeds parent document metadata + section context to prevent context stripping
        let mut contextual_texts = Vec::with_capacity(chunks.len());
        let mut bm25_items = Vec::with_capacity(chunks.len());

        for chunk in &chunks {
            let page_info = chunk
                .page_number
                .map(|p| format!(" | Page: {}", p))
                .unwrap_or_default();
            let contextual_text = format!(
                "[Document: {} | Category: {}{}]\n{}",
                filename, category, page_info, chunk.content
            );
            contextual_texts.push(contextual_text.clone());
            bm25_items.push((chunk.chunk_id.clone(), contextual_text));
        }

        // 4. Generate embeddings on contextualized texts
        let embeddings = self.embedding.embed_batch(&contextual_texts).await?;

        // 5. Store encrypted chunks in SQLite vault
        self.vault.store_chunks(&session_key, &chunks)?;

        // 6. Index into in-memory Dense vector index, HNSW graph, and Sparse BM25 index
        let mut index_items = Vec::with_capacity(chunks.len());
        for (chunk, vector) in chunks.iter().zip(embeddings.into_iter()) {
            index_items.push((chunk.chunk_id.clone(), vector));
        }
        self.index.add_vectors(index_items.clone());
        self.hnsw_index.add_vectors(index_items);
        self.bm25_index.add_documents(bm25_items);

        // 7. Associate with CRM customer if specified
        if let Some(c_id) = customer_id {
            self.crm.associate_document(c_id, doc_id).await;
            self.crm
                .add_note(
                    c_id,
                    "Automated Ingestion",
                    &format!(
                        "Associated document '{}' ({}) with {} contextual chunks.",
                        filename,
                        doc_id,
                        chunks.len()
                    ),
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

    /// Executes 2026 Contextual Hybrid RAG:
    /// Dense HNSW + Sparse BM25 + Reciprocal Rank Fusion (RRF) + Corrective Agentic Grader.
    pub async fn query(&self, prompt: &str, top_k: usize) -> Result<QueryResult> {
        let start = Instant::now();

        // 1. Verify DRM license
        let session_key = self
            .drm
            .get_session_key()
            .await
            .ok_or_else(|| anyhow!("DRM lease is inactive or expired. Search index locked."))?;

        // 2. Embed user query for Dense retrieval
        let query_vec = self.embedding.embed_text(prompt).await?;

        // 3. Multi-Channel Retrieval:
        // A) Dense Search: Sub-3ms HNSW / Cosine Top-(2*K)
        let k = if top_k == 0 { 5 } else { top_k };
        let candidate_pool_size = (k * 3).max(10);
        let dense_matches = if self.hnsw_index.count() > 0 {
            self.hnsw_index.search_top_k(&query_vec, candidate_pool_size)
        } else {
            self.index.search_top_k(&query_vec, candidate_pool_size)
        };

        // B) Sparse Search: In-memory Okapi BM25 Top-(2*K)
        let bm25_matches = self.bm25_index.search_top_k(prompt, candidate_pool_size);

        // C) Reciprocal Rank Fusion (RRF): Combines dense semantics with sparse exact keywords
        let mut fused_matches =
            reciprocal_rank_fusion(&dense_matches, &bm25_matches, k, 1.0, 1.0);

        // 4. Corrective RAG (CRAG) Self-Grader Loop:
        // If query contains noisy punctuation or candidate set is low-scoring, execute a cleaned pass
        if fused_matches.is_empty() || fused_matches.first().map(|(_, s)| *s < 0.012).unwrap_or(true) {
            let clean_prompt: String = prompt
                .chars()
                .map(|c| if c.is_alphanumeric() || c.is_whitespace() { c } else { ' ' })
                .collect();
            let tokens: Vec<&str> = clean_prompt.split_whitespace().collect();
            if tokens.len() >= 2 {
                let refined_query = tokens.join(" ");
                let secondary_bm25 = self.bm25_index.search_top_k(&refined_query, candidate_pool_size);
                if !secondary_bm25.is_empty() {
                    fused_matches =
                        reciprocal_rank_fusion(&dense_matches, &secondary_bm25, k, 1.0, 1.2);
                }
            }
        }

        // 5. Decrypt matched chunks from SQLite vault
        let mut citations = Vec::new();
        let mut contexts = Vec::new();

        for (chunk_id, score) in fused_matches {
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

        // 6. Generate LLM synthesis grounded in retrieved context
        let answer = self.llm.generate_rag_answer(prompt, &contexts).await?;
        let latency_ms = start.elapsed().as_millis();

        Ok(QueryResult {
            query: prompt.to_string(),
            answer,
            citations,
            latency_ms,
            drm_valid: true,
            retrieval_mode: "Contextual Hybrid (Dense HNSW + Sparse BM25 + RRF)".to_string(),
        })
    }

    /// Returns engine operational statistics
    pub async fn get_stats(&self) -> EngineStats {
        let total_chunks = self.vault.count_chunks().unwrap_or(0);
        let total_indexed = if self.hnsw_index.count() > 0 {
            self.hnsw_index.count()
        } else {
            self.index.count()
        };
        let indexed_bm25 = self.bm25_index.count();
        let drm_info = self.drm.get_status_info().await;

        EngineStats {
            total_encrypted_chunks: total_chunks,
            indexed_vectors_in_ram: total_indexed,
            indexed_bm25_docs: indexed_bm25,
            drm_info,
        }
    }

    /// Lists all unique document IDs
    pub async fn list_documents(&self) -> Result<Vec<String>> {
        let session_key = self
            .drm
            .get_session_key()
            .await
            .ok_or_else(|| anyhow!("DRM lease is expired or inactive. Document vault is locked."))?;
        self.vault.list_document_ids(&session_key)
    }

    /// Scans a local storage directory, automatically ingesting any new or modified PDF, TXT, CSV files
    pub async fn scan_and_ingest_directory<P: AsRef<Path>>(&self, dir_path: P) -> Result<usize> {
        let path = dir_path.as_ref();
        if !path.exists() {
            std::fs::create_dir_all(path)?;
        }
        let mut entries = tokio::fs::read_dir(path).await?;
        let mut new_ingested = 0;

        while let Some(entry) = entries.next_entry().await? {
            let file_path = entry.path();
            if file_path.is_file() {
                let ext = file_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if ext == "pdf" || ext == "txt" || ext == "csv" || ext == "md" {
                    let filename = file_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let metadata = entry.metadata().await?;
                    let file_len = metadata.len();

                    // Check if already ingested with identical byte length
                    let should_ingest = {
                        let cache = self.ingested_files.read().unwrap();
                        match cache.get(&filename) {
                            Some(&prev_len) if prev_len == file_len => false,
                            _ => true,
                        }
                    };

                    if should_ingest {
                        println!("🔍 [STORAGE DETECTED] New file found: {} ({} bytes). Reading & Contextualizing...", filename, file_len);
                        match tokio::fs::read(&file_path).await {
                            Ok(bytes) => {
                                let doc_id = format!("DOC-{}", filename);
                                match self.ingest_file(&doc_id, &filename, &bytes, "General", None).await {
                                    Ok(res) => {
                                        println!(
                                            "✅ [AUTO-INGEST SUCCESS] Indexed '{}' | Chunks: {} | Words: {} | Latency: {}ms",
                                            filename, res.chunks_created, res.total_words, res.duration_ms
                                        );
                                        self.ingested_files.write().unwrap().insert(filename, file_len);
                                        new_ingested += 1;
                                    }
                                    Err(e) => {
                                        eprintln!("❌ [AUTO-INGEST ERROR] Failed indexing '{}': {}", filename, e);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("❌ [READ ERROR] Could not read file '{}': {}", filename, e);
                            }
                        }
                    }
                }
            }
        }
        Ok(new_ingested)
    }

    /// Spawns a dedicated background watcher loop on the storage folder (e.g. from RAG_STORAGE_DIR)
    pub fn spawn_directory_watcher(self: Arc<Self>, dir_path: PathBuf, poll_interval_secs: u64) {
        tokio::spawn(async move {
            println!("📂 [STORAGE WATCHER ACTIVE] Monitoring configured folder: {:?}", dir_path);
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(poll_interval_secs));
            loop {
                interval.tick().await;
                if self.drm.is_active().await {
                    let _ = self.scan_and_ingest_directory(&dir_path).await;
                }
            }
        });
    }
}

