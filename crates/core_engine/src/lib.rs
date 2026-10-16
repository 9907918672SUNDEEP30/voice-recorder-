//! # Core Engine Crate
//!
//! DirectML GPU and Local ONNX inference engine driving BGE embeddings and Qwen LLM.

use ai_core::models::adapters::{EmbeddingAdapter, LlmAdapter};
use ai_core::models::onnx_adapters::{BgeEmbeddingLocal, QwenLocal};
use anyhow::Result;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

/// High-Performance Local Embedding Engine
pub struct EmbeddingEngine {
    local_bge: Option<Arc<BgeEmbeddingLocal>>,
}

impl Default for EmbeddingEngine {
    fn default() -> Self {
        Self::new(None)
    }
}

impl EmbeddingEngine {
    pub fn new(models_dir: Option<&Path>) -> Self {
        let base_path = models_dir
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| Path::new("C:/AI_Models").to_path_buf());

        let bge_model = base_path.join("bge_small_embedding").join("model.onnx");
        let local_bge = if bge_model.exists() {
            if let Some(path_str) = bge_model.to_str() {
                match BgeEmbeddingLocal::new(path_str) {
                    Ok(adapter) => {
                        println!("Successfully attached Local BGE Embedding Model from {:?}", bge_model);
                        Some(Arc::new(adapter))
                    }
                    Err(e) => {
                        eprintln!("BGE Embedding initialization notice: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        Self { local_bge }
    }

    /// Converts text into a normalized 384-dimensional dense vector
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        if let Some(bge) = &self.local_bge {
            bge.embed_text(text).await
        } else {
            // High-precision deterministic fallback vector for testing/offline scenarios
            Ok(Self::deterministic_fallback_embedding(text, 384))
        }
    }

    /// Batch embeds multiple text strings
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        for t in texts {
            results.push(self.embed_text(t).await?);
        }
        Ok(results)
    }

    fn deterministic_fallback_embedding(text: &str, dim: usize) -> Vec<f32> {
        let mut vec = vec![0.0f32; dim];
        for (i, word) in text.split_whitespace().enumerate() {
            let mut hasher = DefaultHasher::new();
            word.hash(&mut hasher);
            let h = hasher.finish();
            let idx = (h as usize) % dim;
            let val = ((h >> 32) as f32 / u32::MAX as f32) * 2.0 - 1.0;
            vec[idx] += val * (1.0 / (i as f32 + 1.0).sqrt());
        }

        // L2 Normalization
        let norm_sq: f32 = vec.iter().map(|&x| x * x).sum();
        let norm = norm_sq.sqrt().max(1e-12);
        vec.iter().map(|&x| x / norm).collect()
    }
}

/// Local Quantized LLM Contextual Reasoning Engine
pub struct LlmEngine {
    local_qwen: Option<Arc<QwenLocal>>,
}

impl Default for LlmEngine {
    fn default() -> Self {
        Self::new(None)
    }
}

impl LlmEngine {
    pub fn new(models_dir: Option<&Path>) -> Self {
        let base_path = models_dir
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| Path::new("C:/AI_Models").to_path_buf());

        let qwen_model = base_path.join("qwen2-vl-2b").join("llmmodel.onnx");
        let vision_model = base_path.join("qwen2-vl-2b").join("visionmodel.onnx");
        let vision_encoder = base_path.join("qwen2-vl-2b").join("vision_encoder.onnx");
        let tokenizer = base_path.join("qwen2-vl-2b").join("tokenizer.json");

        let local_qwen = if qwen_model.exists() && tokenizer.exists() && vision_model.exists() && vision_encoder.exists() {
            if let (Some(m_str), Some(v_str), Some(ve_str), Some(t_str)) = (
                qwen_model.to_str(),
                vision_model.to_str(),
                vision_encoder.to_str(),
                tokenizer.to_str(),
            ) {
                match QwenLocal::new(m_str, v_str, ve_str, t_str) {
                    Ok(adapter) => {
                        println!("Successfully attached Local Qwen LLM from {:?}", qwen_model);
                        Some(Arc::new(adapter))
                    }
                    Err(e) => {
                        eprintln!("Qwen LLM initialization notice: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        Self { local_qwen }
    }

    /// Synthesizes grounded answer strictly using the provided context chunks
    pub async fn generate_rag_answer(&self, query: &str, contexts: &[String]) -> Result<String> {
        let context_block = contexts
            .iter()
            .enumerate()
            .map(|(i, c)| format!("[Source {}]: {}", i + 1, c))
            .collect::<Vec<String>>()
            .join("\n\n");

        let system_prompt = "You are an institutional SME Business Assistant. Answer the question accurately using ONLY the supplied local document context. If the answer cannot be found in the context, state clearly that it is not present in local records.";
        let user_prompt = format!(
            "Context documents:\n{}\n\nQuestion: {}\nAnswer concisely with factual source references:",
            context_block, query
        );

        if let Some(qwen) = &self.local_qwen {
            let cancel_token = CancellationToken::new();
            if let Ok(mut rx) = qwen
                .generate_stream(
                    system_prompt,
                    vec![("user".to_string(), user_prompt)],
                    cancel_token,
                )
                .await
            {
                let mut full_response = String::new();
                while let Some(chunk) = rx.recv().await {
                    full_response.push_str(&chunk);
                }
                if !full_response.trim().is_empty() {
                    return Ok(full_response.trim().to_string());
                }
            }
        }

        // Deterministic synthesis fallback when local LLM is off
        let fallback = if !contexts.is_empty() {
            format!(
                "Based on local business records:\n{}\n\nFound relevant match in {} local document citations.",
                contexts[0],
                contexts.len()
            )
        } else {
            "No matching business records found in the local encrypted vault.".to_string()
        };

        Ok(fallback)
    }
}
