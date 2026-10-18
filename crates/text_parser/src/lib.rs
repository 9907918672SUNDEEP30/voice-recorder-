//! # Text Parser Crate
//!
//! Pure Rust document ingestion and sliding-window chunking engine for local RAG.
//! Supports native PDF text extraction, structured CSV normalization, and text/markdown files.

pub mod chunker;
pub mod csv_parser;
pub mod pdf;

pub use chunker::{DocumentChunk, SlidingWindowChunker};
pub use csv_parser::CsvDocumentParser;
pub use pdf::{PdfDocumentParser, PdfParseResult};

use anyhow::Result;

/// Unified Multi-Format Business Document Parser
pub struct DocumentParser {
    chunker: SlidingWindowChunker,
    pdf_parser: PdfDocumentParser,
    csv_parser: CsvDocumentParser,
}

impl Default for DocumentParser {
    fn default() -> Self {
        let chunker = SlidingWindowChunker::default();
        Self {
            pdf_parser: PdfDocumentParser::new(chunker),
            csv_parser: CsvDocumentParser::new(chunker),
            chunker,
        }
    }
}

impl DocumentParser {
    pub fn new(min_words: usize, max_words: usize, overlap_words: usize) -> Self {
        let chunker = SlidingWindowChunker::new(min_words, max_words, overlap_words);
        Self {
            pdf_parser: PdfDocumentParser::new(chunker),
            csv_parser: CsvDocumentParser::new(chunker),
            chunker,
        }
    }

    /// Parse plain text or markdown document
    pub fn parse_plain_text(&self, doc_id: &str, text: &str, category: &str) -> Vec<DocumentChunk> {
        self.chunker.chunk_text(doc_id, text, None, category)
    }

    /// Parse PDF binary bytes
    pub fn parse_pdf_bytes(&self, doc_id: &str, bytes: &[u8], category: &str) -> Result<Vec<DocumentChunk>> {
        self.pdf_parser.parse_pdf_bytes(doc_id, bytes, category)
    }

    /// Parse CSV formatted string
    pub fn parse_csv_str(&self, doc_id: &str, csv_content: &str, category: &str) -> Result<Vec<DocumentChunk>> {
        self.csv_parser.parse_csv_str(doc_id, csv_content, category)
    }
}
