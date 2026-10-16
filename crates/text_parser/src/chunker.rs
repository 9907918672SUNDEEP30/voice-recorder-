//! # Sliding-Window Chunker
//!
//! Splitting documents into semantically coherent overlapping windows (300-500 words,
//! 50-word overlap) with unicode word boundary preservation.

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A granular text chunk ready for vectorization and encrypted indexing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentChunk {
    pub chunk_id: String,
    pub doc_id: String,
    pub chunk_index: usize,
    pub content: String,
    pub word_count: usize,
    pub page_number: Option<u32>,
    pub category: String, // e.g. "Invoice", "Contract", "GST_Record", "General"
    pub metadata_tags: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct SlidingWindowChunker {
    pub min_words: usize,
    pub max_words: usize,
    pub overlap_words: usize,
}

impl Default for SlidingWindowChunker {
    fn default() -> Self {
        Self::new(300, 500, 50)
    }
}

impl SlidingWindowChunker {
    pub fn new(min_words: usize, max_words: usize, overlap_words: usize) -> Self {
        let max_words = max_words.max(min_words + 10);
        let overlap_words = overlap_words.min(min_words / 2);
        Self {
            min_words,
            max_words,
            overlap_words,
        }
    }

    /// Chunks a text string using sliding window with overlap
    pub fn chunk_text(
        &self,
        doc_id: &str,
        raw_text: &str,
        page_number: Option<u32>,
        category: &str,
    ) -> Vec<DocumentChunk> {
        let words: Vec<&str> = raw_text.split_whitespace().collect();
        if words.is_empty() {
            return Vec::new();
        }

        // If total words is small, return a single chunk
        if words.len() <= self.max_words {
            let content = words.join(" ");
            let chunk_id = Self::compute_chunk_id(doc_id, 0, &content);
            return vec![DocumentChunk {
                chunk_id,
                doc_id: doc_id.to_string(),
                chunk_index: 0,
                word_count: words.len(),
                content,
                page_number,
                category: category.to_string(),
                metadata_tags: Self::detect_metadata_tags(&words),
            }];
        }

        let mut chunks = Vec::new();
        let target_size = (self.min_words + self.max_words) / 2;
        let step = (target_size - self.overlap_words).max(1);

        let mut start = 0;
        let mut chunk_index = 0;

        while start < words.len() {
            let end = (start + target_size).min(words.len());
            let chunk_words = &words[start..end];
            let content = chunk_words.join(" ");
            let chunk_id = Self::compute_chunk_id(doc_id, chunk_index, &content);

            chunks.push(DocumentChunk {
                chunk_id,
                doc_id: doc_id.to_string(),
                chunk_index,
                word_count: chunk_words.len(),
                content,
                page_number,
                category: category.to_string(),
                metadata_tags: Self::detect_metadata_tags(chunk_words),
            });

            chunk_index += 1;
            if end >= words.len() {
                break;
            }
            start += step;
        }

        chunks
    }

    /// Automatically extracts relevant business tags (Invoice, Total Amount, Date, GST)
    fn detect_metadata_tags(words: &[&str]) -> Vec<String> {
        let mut tags = Vec::new();
        let lower: Vec<String> = words.iter().map(|w| w.to_lowercase()).collect();

        if lower.iter().any(|w| w.contains("invoice") || w.contains("bill") || w.contains("challan")) {
            tags.push("Financial_Invoice".to_string());
        }
        if lower.iter().any(|w| w.contains("gstin") || w.contains("gst") || w.contains("tax")) {
            tags.push("Tax_Compliance".to_string());
        }
        if lower.iter().any(|w| w.contains("contract") || w.contains("agreement") || w.contains("clause")) {
            tags.push("Legal_Contract".to_string());
        }
        if lower.iter().any(|w| w.contains("patient") || w.contains("rx") || w.contains("diagnosis")) {
            tags.push("Medical_Health".to_string());
        }

        tags
    }

    fn compute_chunk_id(doc_id: &str, chunk_index: usize, content: &str) -> String {
        let mut hasher = DefaultHasher::new();
        doc_id.hash(&mut hasher);
        chunk_index.hash(&mut hasher);
        content.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sliding_window_chunking() {
        let chunker = SlidingWindowChunker::new(10, 20, 3);
        let sample_text = "The quick brown fox jumps over the lazy dog repeatedly to generate a long enough sentence that exceeds twenty words so that we can verify the sliding window overlap algorithm functioning properly.";

        let chunks = chunker.chunk_text("doc_test_01", sample_text, Some(1), "Invoice");
        assert!(chunks.len() >= 2);
        assert_eq!(chunks[0].category, "Invoice");
        assert_eq!(chunks[0].chunk_index, 0);
        assert_eq!(chunks[1].chunk_index, 1);

        // Verify word overlap exists between chunk 0 and chunk 1
        let words_0: Vec<&str> = chunks[0].content.split_whitespace().collect();
        let words_1: Vec<&str> = chunks[1].content.split_whitespace().collect();
        let overlap_found = words_0.iter().any(|w| words_1.contains(w));
        assert!(overlap_found);
    }
}
