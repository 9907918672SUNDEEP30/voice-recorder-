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

/// Result of PDF parsing including successful chunks and any pages that failed extraction.
#[derive(Debug, Clone)]
pub struct PdfParseResult {
    pub chunks: Vec<DocumentChunk>,
    /// Page numbers (1-indexed) that encountered extraction errors
    pub failed_pages: Vec<u32>,
    /// Total pages in document
    pub total_pages: usize,
}

impl PdfDocumentParser {
    pub fn new(chunker: SlidingWindowChunker) -> Self {
        Self { chunker }
    }

    /// Extracts and chunks all pages from PDF bytes, reporting any partial extraction failures
    pub fn parse_pdf_bytes_detailed(
        &self,
        doc_id: &str,
        pdf_bytes: &[u8],
        category: &str,
    ) -> Result<PdfParseResult> {
        let cursor = Cursor::new(pdf_bytes.to_vec());
        let doc = Document::load_from(cursor)
            .context("Failed to load PDF document into native lopdf parser")?;

        let pages = doc.get_pages();
        let total_pages = pages.len();
        let mut all_chunks = Vec::new();
        let mut failed_pages = Vec::new();

        for &page_num in pages.keys() {
            match doc.extract_text(&[page_num]) {
                Ok(content) => {
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
                Err(e) => {
                    eprintln!(
                        "[PDF PARSER WARNING] Document '{}', Page {} text extraction failed: {}",
                        doc_id, page_num, e
                    );
                    failed_pages.push(page_num);
                }
            }
        }

        // Adjust overall chunk indices sequentially
        for (idx, chunk) in all_chunks.iter_mut().enumerate() {
            chunk.chunk_index = idx;
        }

        Ok(PdfParseResult {
            chunks: all_chunks,
            failed_pages,
            total_pages,
        })
    }

    /// Extracts and chunks all pages from PDF bytes (standard backwards-compatible interface)
    pub fn parse_pdf_bytes(
        &self,
        doc_id: &str,
        pdf_bytes: &[u8],
        category: &str,
    ) -> Result<Vec<DocumentChunk>> {
        let result = self.parse_pdf_bytes_detailed(doc_id, pdf_bytes, category)?;
        if !result.failed_pages.is_empty() {
            eprintln!(
                "[PDF PARSER] Document '{}' completed with {} skipped/failed page(s): {:?}",
                doc_id,
                result.failed_pages.len(),
                result.failed_pages
            );
        }
        Ok(result.chunks)
    }
}
