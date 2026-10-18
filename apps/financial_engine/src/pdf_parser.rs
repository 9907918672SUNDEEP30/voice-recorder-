//! # Module 4: PDF Corporate Filing & Keyword/Regex Extractor
//!
//! Native Rust asynchronous PDF text extraction (`lopdf`) for BSE/NSE corporate
//! announcements, investor presentations, and concall transcripts.
//! Features SIMD-accelerated regex matching for Capex, Order Book, Plant Expansion,
//! and Promoter Pledging, paired with local ONNX BGE Semantic Reranker from `C:\AI_Models`.

use ai_core::models::adapters::RerankerAdapter;
use ai_core::models::onnx_adapters::BgeRerankerLocal;
use anyhow::{Context, Result};
use lopdf::Document;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path::Path;

/// Key Metrics Extracted from Corporate Presentations / Concalls
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedFilingMetrics {
    pub capex_disclosures: Vec<String>,
    pub order_book_disclosures: Vec<String>,
    pub plant_expansion_disclosures: Vec<String>,
    pub promoter_pledge_disclosures: Vec<String>,
    pub top_semantic_highlights: Vec<(String, f32)>, // Ranked by BgeReranker
    pub sentiment_flag: FilingSentiment,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FilingSentiment {
    StrongExpansionary,
    StableNeutral,
    PromoterPledgeRisk,
    DilutionOrStress,
}

/// Native Rust PDF Text & Disclosure Miner
pub struct PdfFilingParser {
    capex_regex: Regex,
    order_book_regex: Regex,
    expansion_regex: Regex,
    pledge_regex: Regex,
}

impl Default for PdfFilingParser {
    fn default() -> Self {
        Self::new().expect("Failed to compile financial regexes")
    }
}

impl PdfFilingParser {
    pub fn new() -> Result<Self> {
        let capex_regex = Regex::new(
            r"(?i)(?:capex|capital\s+expenditure|investment\s+outlay)[^\.\n]{1,120}(?:rs\.?|inr|₹)?\s*(\d+(?:,\d+)*(?:\.\d+)?)\s*(?:cr|crore|crores|mn|bn)?",
        )?;
        let order_book_regex = Regex::new(
            r"(?i)(?:order\s+book|unexecuted\s+orders|pipeline|bid\s+pipeline)[^\.\n]{1,120}(?:rs\.?|inr|₹)?\s*(\d+(?:,\d+)*(?:\.\d+)?)\s*(?:cr|crore|crores)?",
        )?;
        let expansion_regex = Regex::new(
            r"(?i)(?:capacity\s+expansion|greenfield|brownfield|commercial\s+production|new\s+manufacturing\s+facility|debottlenecking)[^\.\n]{1,150}",
        )?;
        let pledge_regex = Regex::new(
            r"(?i)(?:pledged|encumbered|encumbrance|revocation\s+of\s+pledge)\s+(?:shares|promoter|equity)[^\r\n]{1,120}",
        )?;

        Ok(Self {
            capex_regex,
            order_book_regex,
            expansion_regex,
            pledge_regex,
        })
    }

    /// Extracts all readable text from a PDF byte slice asynchronously without external C-binaries
    pub async fn extract_text_from_bytes(&self, pdf_bytes: &[u8]) -> Result<String> {
        let cursor = Cursor::new(pdf_bytes.to_vec());
        let doc = Document::load_from(cursor)
            .context("Failed to load PDF document into native lopdf parser")?;

        let mut extracted_text = String::new();
        let pages = doc.get_pages();
        for &page_num in pages.keys() {
            if let Ok(content) = doc.extract_text(&[page_num]) {
                extracted_text.push_str(&content);
                extracted_text.push('\n');
            }
        }

        Ok(extracted_text)
    }

    /// Scans extracted text and uses local BGE Reranker (`C:\AI_Models\bge_reranker`)
    /// to prioritize institutional-grade catalysts vs red-flags.
    pub async fn parse_filing_disclosures(
        &self,
        text: &str,
        reranker: Option<&BgeRerankerLocal>,
    ) -> ExtractedFilingMetrics {
        let mut capex_disclosures = Vec::new();
        let mut order_book_disclosures = Vec::new();
        let mut plant_expansion_disclosures = Vec::new();
        let mut promoter_pledge_disclosures = Vec::new();

        for cap in self.capex_regex.find_iter(text) {
            capex_disclosures.push(cap.as_str().trim().to_string());
        }

        for cap in self.order_book_regex.find_iter(text) {
            order_book_disclosures.push(cap.as_str().trim().to_string());
        }

        for cap in self.expansion_regex.find_iter(text) {
            plant_expansion_disclosures.push(cap.as_str().trim().to_string());
        }

        for cap in self.pledge_regex.find_iter(text) {
            let matched_text = cap.as_str().trim();
            let lower = matched_text.to_lowercase();
            // Disclosures stating 0.0%, 0%, nil, zero, or none mean ZERO encumbrance, which is safe
            let is_zero_pledge = lower.contains("0.0%")
                || lower.contains("0%")
                || lower.contains("nil")
                || lower.contains("zero")
                || lower.contains("none")
                || lower.contains("no pledge");
            if !is_zero_pledge {
                promoter_pledge_disclosures.push(matched_text.to_string());
            }
        }

        // Aggregate unique candidate sentences for semantic reranking
        let mut candidates = Vec::new();
        candidates.extend(capex_disclosures.iter().cloned());
        candidates.extend(order_book_disclosures.iter().cloned());
        candidates.extend(plant_expansion_disclosures.iter().cloned());
        candidates.extend(promoter_pledge_disclosures.iter().cloned());
        candidates.dedup();

        let top_semantic_highlights = if let Some(model) = reranker {
            if !candidates.is_empty() {
                let query = "Capital expenditure, order book growth, capacity expansion, or promoter pledge risk";
                match model.rerank(query, candidates).await {
                    Ok(ranked) => ranked.into_iter().take(5).collect(),
                    Err(_) => Vec::new(),
                }
            } else {
                Vec::new()
            }
        } else {
            // Deterministic top candidates fallback
            candidates.into_iter().take(5).map(|c| (c, 0.75f32)).collect()
        };

        // Sentiment determination
        let sentiment_flag = if !promoter_pledge_disclosures.is_empty() {
            FilingSentiment::PromoterPledgeRisk
        } else if !capex_disclosures.is_empty() || !plant_expansion_disclosures.is_empty() {
            FilingSentiment::StrongExpansionary
        } else {
            FilingSentiment::StableNeutral
        };

        ExtractedFilingMetrics {
            capex_disclosures,
            order_book_disclosures,
            plant_expansion_disclosures,
            promoter_pledge_disclosures,
            top_semantic_highlights,
            sentiment_flag,
        }
    }

    /// Helper to attempt loading BGE Reranker from C:\AI_Models with graceful fallback
    pub fn try_load_reranker(models_dir: Option<&Path>) -> Option<BgeRerankerLocal> {
        let base_path = models_dir
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| Path::new("C:/AI_Models").to_path_buf());

        let reranker_model = base_path.join("bge_reranker").join("model.onnx");
        if reranker_model.exists() {
            if let Some(path_str) = reranker_model.to_str() {
                match BgeRerankerLocal::new(path_str) {
                    Ok(adapter) => {
                        println!("Successfully attached Local BGE Reranker: {:?}", reranker_model);
                        return Some(adapter);
                    }
                    Err(e) => eprintln!("Failed to initialize BGE Reranker ONNX: {}", e),
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_regex_extraction() {
        let parser = PdfFilingParser::new().unwrap();
        let sample_filing = r#"
            The company has announced a greenfield capex of Rs 450 crore in Pune.
            Total order book as of December 31, 2024 stands at Rs 3,400 crore with 18 months execution timeline.
            New commercial production is expected by Q3 FY26.
            Promoter pledged shares remain 0.0% of total equity.
        "#;

        let metrics = parser.parse_filing_disclosures(sample_filing, None).await;
        assert_eq!(metrics.capex_disclosures.len(), 1);
        assert_eq!(metrics.order_book_disclosures.len(), 1);
        assert_eq!(metrics.plant_expansion_disclosures.len(), 2);
        // Zero pledge (0.0%) is not a risk; expansion capex is recognized
        assert_eq!(metrics.sentiment_flag, FilingSentiment::StrongExpansionary);
        assert_eq!(metrics.promoter_pledge_disclosures.len(), 0);

        // Active pledging disclosure test
        let sample_risk = "Promoter pledged shares increased to 45.2% of total equity as collateral.";
        let risk_metrics = parser.parse_filing_disclosures(sample_risk, None).await;
        assert_eq!(risk_metrics.sentiment_flag, FilingSentiment::PromoterPledgeRisk);
        assert_eq!(risk_metrics.promoter_pledge_disclosures.len(), 1);
    }
}
