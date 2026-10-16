//! # Native PDF Parser
//!
//! Page-by-page text extraction from PDF files using pure Rust `lopdf` (zero external C-binaries).

use crate::chunker::{DocumentChunk, SlidingWindowChunker};
use anyhow::{Context, Result};
use lopdf::Document;
use std::io::Cursor;

pub struct PdfDocumentParser {
    chunker: SlidingWindowChunker,
}

impl Default for PdfDocumentParser {
    fn default() -> Self {
        Self::new(SlidingWindowChunker::default())
    }
}

impl PdfDocumentParser {
    pub fn new(chunker: SlidingWindowChunker) -> Self {
        Self { chunker }
    }

    /// Extracts and chunks all pages from PDF bytes
    pub fn parse_pdf_bytes(
        &self,
        doc_id: &str,
        pdf_bytes: &[u8],
        category: &str,
    ) -> Result<Vec<DocumentChunk>> {
        let cursor = Cursor::new(pdf_bytes.to_vec());
        let doc = Document::load_from(cursor)
            .context("Failed to load PDF document into native lopdf parser")?;

        let pages = doc.get_pages();
        let mut all_chunks = Vec::new();

        for &page_num in pages.keys() {
            if let Ok(content) = doc.extract_text(&[page_num]) {
                if !content.trim().is_empty() {
                    let page_chunks = self.chunker.chunk_text(
                        doc_id,
                        &content,
                        Some(page_num),
                        category,
                    );
                    all_chunks.extend(page_chunks);
                }
            }
        }

        // Adjust overall chunk indices sequentially
        for (idx, chunk) in all_chunks.iter_mut().enumerate() {
            chunk.chunk_index = idx;
        }

        Ok(all_chunks)
    }
}
