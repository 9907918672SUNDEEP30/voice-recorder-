//! # Pure Rust Deterministic Automata Parser (Zero-AI Stream Engine)
//!
//! Extracts PAN cards, GSTIN, financial amounts, invoice numbers, and dates
//! in <5ms directly from digital text streams without invoking any heavy neural models.

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedFinancialData {
    pub pan_numbers: Vec<String>,
    pub gstin_numbers: Vec<String>,
    pub invoice_numbers: Vec<String>,
    pub dates: Vec<String>,
    pub amounts: Vec<f64>,
    pub total_amount: Option<f64>,
    pub gst_amount: Option<f64>,
    pub doc_type: String,
    pub is_digital_stream: bool,
}

pub struct DeterministicAutomataParser {
    pan_regex: Regex,
    gstin_regex: Regex,
    amount_regex: Regex,
    date_regex: Regex,
    invoice_regex: Regex,
}

impl Default for DeterministicAutomataParser {
    fn default() -> Self {
        Self::new()
    }
}

impl DeterministicAutomataParser {
    pub fn new() -> Self {
        Self {
            pan_regex: Regex::new(r"\b[A-Z]{5}[0-9]{4}[A-Z]\b").unwrap(),
            gstin_regex: Regex::new(r"\b[0-9]{2}[A-Z]{5}[0-9]{4}[A-Z][1-9A-Z]Z[0-9A-Z]\b").unwrap(),
            amount_regex: Regex::new(r"(?:₹|Rs\.?|INR)\s*([\d,]+(?:\.\d{2})?)").unwrap(),
            date_regex: Regex::new(r"\b\d{2}[/-]\d{2}[/-]\d{4}\b").unwrap(),
            invoice_regex: Regex::new(r"(?i)\b(?:inv|invoice|bill|challan)[\s:#\-_]*([a-z0-9\-_/]+)\b").unwrap(),
        }
    }

    /// Fast stream analysis: determines if PDF contains raw digital text stream
    pub fn probe_is_digital(&self, text_sample: &str) -> bool {
        let alpha_count = text_sample.chars().filter(|c| c.is_alphabetic()).count();
        // If more than 50 alphabetic characters in stream, it is digital
        alpha_count > 50
    }

    /// Parses text stream and extracts all structured entities in <5ms
    pub fn parse_text_stream(&self, raw_text: &str) -> ExtractedFinancialData {
        let is_digital = self.probe_is_digital(raw_text);

        // 1. PAN Cards
        let mut pan_numbers: Vec<String> = self
            .pan_regex
            .find_iter(raw_text)
            .map(|m| m.as_str().to_string())
            .collect();
        pan_numbers.sort();
        pan_numbers.dedup();

        // 2. GSTIN Numbers
        let mut gstin_numbers: Vec<String> = self
            .gstin_regex
            .find_iter(raw_text)
            .map(|m| m.as_str().to_string())
            .collect();
        gstin_numbers.sort();
        gstin_numbers.dedup();

        // 3. Invoice Numbers
        let mut invoice_numbers: Vec<String> = self
            .invoice_regex
            .captures_iter(raw_text)
            .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
            .collect();
        invoice_numbers.sort();
        invoice_numbers.dedup();

        // 4. Dates
        let mut dates: Vec<String> = self
            .date_regex
            .find_iter(raw_text)
            .map(|m| m.as_str().to_string())
            .collect();
        dates.sort();
        dates.dedup();

        // 5. Currency Amounts
        let mut amounts: Vec<f64> = self
            .amount_regex
            .captures_iter(raw_text)
            .filter_map(|c| {
                let clean_str = c.get(1)?.as_str().replace(',', "");
                clean_str.parse::<f64>().ok()
            })
            .collect();
        amounts.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        let total_amount = amounts.first().cloned();
        let gst_amount = amounts.iter().find(|&&a| {
            if let Some(tot) = total_amount {
                // Typical GST rates (18%, 12%, 5%)
                let ratio = a / tot;
                (0.04..0.29).contains(&ratio)
            } else {
                false
            }
        }).cloned();

        // 6. Detect Doc Type
        let lower = raw_text.to_lowercase();
        let doc_type = if lower.contains("itr") || lower.contains("income tax return") {
            "Income Tax Return (ITR-V)".to_string()
        } else if lower.contains("tax invoice") || lower.contains("bill of supply") {
            "Tax Invoice".to_string()
        } else if lower.contains("challan") {
            "Tax Challan".to_string()
        } else if lower.contains("bank") || lower.contains("account statement") {
            "Bank Statement".to_string()
        } else if lower.contains("sale deed") || lower.contains("conveyance") {
            "Sale Deed / Property Document".to_string()
        } else {
            "General Financial Document".to_string()
        };

        ExtractedFinancialData {
            pan_numbers,
            gstin_numbers,
            invoice_numbers,
            dates,
            amounts,
            total_amount,
            gst_amount,
            doc_type,
            is_digital_stream: is_digital,
        }
    }
}
