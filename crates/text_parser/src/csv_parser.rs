//! # Streaming CSV Parser
//!
//! Schema-aware CSV parser that converts structured tabular rows (orders, invoices, customer logs)
//! into semantic narrative chunks for vector embeddings and search.

use crate::chunker::{DocumentChunk, SlidingWindowChunker};
use anyhow::{Context, Result};
use std::io::Cursor;

pub struct CsvDocumentParser {
    chunker: SlidingWindowChunker,
}

impl Default for CsvDocumentParser {
    fn default() -> Self {
        Self::new(SlidingWindowChunker::default())
    }
}

impl CsvDocumentParser {
    pub fn new(chunker: SlidingWindowChunker) -> Self {
        Self { chunker }
    }

    /// Parses CSV content into semantic document chunks
    pub fn parse_csv_str(
        &self,
        doc_id: &str,
        csv_content: &str,
        category: &str,
    ) -> Result<Vec<DocumentChunk>> {
        let cursor = Cursor::new(csv_content.as_bytes());
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(cursor);

        let headers: Vec<String> = rdr
            .headers()
            .context("Failed to read CSV headers")?
            .iter()
            .map(|h| h.trim().to_string())
            .collect();

        let mut narrative_lines = Vec::new();
        for (row_idx, result) in rdr.records().enumerate() {
            let record = result.context("Failed to parse CSV record line")?;
            let mut line_parts = Vec::new();

            for (col_idx, val) in record.iter().enumerate() {
                let header_name = headers.get(col_idx).map(|s| s.as_str()).unwrap_or("Column");
                let val = val.trim();
                if !val.is_empty() {
                    line_parts.push(format!("{}: {}", header_name, val));
                }
            }

            if !line_parts.is_empty() {
                narrative_lines.push(format!("Row {}: {}", row_idx + 1, line_parts.join(" | ")));
            }
        }

        let full_text = narrative_lines.join("\n");
        Ok(self.chunker.chunk_text(doc_id, &full_text, None, category))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_parser_semantic_conversion() {
        let parser = CsvDocumentParser::default();
        let sample_csv = r#"CustomerName,InvoiceID,AmountINR,Status,City
Sharma Traders,INV-1001,45000,Paid,Jaipur
Verma Electronics,INV-1002,125000,Pending,Delhi
Gupta Textiles,INV-1003,32000,Delivered,Surat
"#;

        let chunks = parser.parse_csv_str("csv_doc_01", sample_csv, "Invoice").unwrap();
        assert!(!chunks.is_empty());
        assert!(chunks[0].content.contains("CustomerName: Sharma Traders"));
        assert!(chunks[0].content.contains("AmountINR: 45000"));
        assert_eq!(chunks[0].category, "Invoice");
    }
}
