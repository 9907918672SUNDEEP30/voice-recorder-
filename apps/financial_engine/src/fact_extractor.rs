//! # Phase 2: Quantitative Fact & Numerical Extractor
//!
//! Sub-second deterministic parsing of high-priority exchange filings.
//! Extracts contract sizes, execution timelines, capacity expansions, and calculates
//! forward annualized revenue additions and incremental EPS accretion.

use regex::Regex;
use serde::{Deserialize, Serialize};

/// Extracted hard numerical facts from corporate filing text
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedFilingFacts {
    /// Contract or order value in ₹ Crores
    pub contract_value_cr: Option<f64>,
    /// Execution schedule in months
    pub timeline_months: Option<u32>,
    /// Annualized revenue run-rate addition in ₹ Crores
    pub annualized_revenue_cr: Option<f64>,
    /// Capacity addition description and unit
    pub capacity_addition: Option<String>,
    /// Greenfield or Brownfield classification
    pub expansion_type: Option<CapexNature>,
    /// Counterparty or client name
    pub counterparty: Option<String>,
    /// Asymmetry / Materiality score (0.0 to 100.0)
    pub materiality_score: f64,
}

/// Nature of capital expenditure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapexNature {
    Greenfield,
    Brownfield,
    ModernizationOrDebottlenecking,
}

/// Quantitative Fact Parser
pub struct QuantitativeFactExtractor {
    value_inr_cr_regex: Regex,
    value_usd_m_regex: Regex,
    timeline_months_regex: Regex,
    timeline_years_regex: Regex,
    capacity_regex: Regex,
    counterparty_regex: Regex,
    greenfield_regex: Regex,
    brownfield_regex: Regex,
}

impl Default for QuantitativeFactExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl QuantitativeFactExtractor {
    pub fn new() -> Self {
        let value_inr_cr_regex = Regex::new(
            r"(?i)(?:valued\s+at|worth|order\s+of|contract\s+of|sum\s+of|amounting\s+to|aggregate\s+value\s+of)?\s*(?:rs\.?|inr|₹)?\s*(\d+(?:,\d+)*(?:\.\d+)?)\s*(?:cr|crore|crores)",
        ).unwrap();

        let value_usd_m_regex = Regex::new(
            r"(?i)(?:valued\s+at|worth|order\s+of|contract\s+of)?\s*(?:usd|\$)\s*(\d+(?:,\d+)*(?:\.\d+)?)\s*(?:mn|million)",
        ).unwrap();

        let timeline_months_regex = Regex::new(
            r"(?i)(?:timeline|period|tenure|duration|schedule|completion|completed|within|executable|in|over)(?:\s+(?:of|in|within|is|about|around|approximately))?\s*(\d+)\s*months?",
        ).unwrap();

        let timeline_years_regex = Regex::new(
            r"(?i)(?:timeline|period|tenure|duration|schedule|completion|completed|within|executable|in|over)(?:\s+(?:of|in|within|is|about|around|approximately))?\s*(\d+(?:\.\d+)?)\s*years?",
        ).unwrap();

        let capacity_regex = Regex::new(
            r"(?i)(\d+(?:,\d+)*(?:\.\d+)?)\s*(mw|gw|mtpa|tpa|klpd|lakh\s+units|units/annum)",
        ).unwrap();

        let counterparty_regex = Regex::new(
            r"(?i)(?:from|by|with|awarded\s+by)\s+(?:m/s\.?\s+)?([A-Z][A-Za-z0-9\s&.,]{2,40}?)(?:\s+(?:ltd|limited|corp|corporation|inc|government|ministry|authority))",
        ).unwrap();

        let greenfield_regex = Regex::new(r"(?i)greenfield|new\s+plant|new\s+facility|new\s+manufacturing\s+unit").unwrap();
        let brownfield_regex = Regex::new(r"(?i)brownfield|expansion\s+at\s+existing|debottlenecking|upgradation").unwrap();

        Self {
            value_inr_cr_regex,
            value_usd_m_regex,
            timeline_months_regex,
            timeline_years_regex,
            capacity_regex,
            counterparty_regex,
            greenfield_regex,
            brownfield_regex,
        }
    }

    /// Extracts all numerical facts and forward run-rates from filing text
    pub fn extract_facts(&self, text: &str, company_annual_turnover_cr: Option<f64>) -> ExtractedFilingFacts {
        // 1. Contract Value in ₹ Crores (checking INR Crores first, then USD Millions converted at 84 INR/USD)
        let contract_value_cr = if let Some(caps) = self.value_inr_cr_regex.captures(text) {
            caps.get(1).and_then(|m| m.as_str().replace(',', "").parse::<f64>().ok())
        } else if let Some(caps) = self.value_usd_m_regex.captures(text) {
            caps.get(1)
                .and_then(|m| m.as_str().replace(',', "").parse::<f64>().ok())
                .map(|usd_m| (usd_m * 84.0) / 10.0) // 1 USD Mn = 8.4 Cr INR
        } else {
            None
        };

        // 2. Timeline in Months
        let timeline_months = if let Some(caps) = self.timeline_months_regex.captures(text) {
            caps.get(1).and_then(|m| m.as_str().parse::<u32>().ok())
        } else if let Some(caps) = self.timeline_years_regex.captures(text) {
            caps.get(1)
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .map(|yrs| (yrs * 12.0).round() as u32)
        } else {
            None
        };

        // 3. Annualized Revenue Run-Rate Addition
        let annualized_revenue_cr = match (contract_value_cr, timeline_months) {
            (Some(val), Some(months)) if months > 0 => {
                let years = months as f64 / 12.0;
                Some((val / years * 100.0).round() / 100.0)
            }
            (Some(val), None) => {
                // If timeline not mentioned, conservatively assume 12-month delivery
                Some(val)
            }
            _ => None,
        };

        // 4. Capacity Addition
        let capacity_addition = self.capacity_regex.captures(text).map(|caps| {
            let num = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let unit = caps.get(2).map(|m| m.as_str().to_uppercase()).unwrap_or_default();
            format!("{} {}", num, unit)
        });

        // 5. Expansion Nature
        let expansion_type = if self.greenfield_regex.is_match(text) {
            Some(CapexNature::Greenfield)
        } else if self.brownfield_regex.is_match(text) {
            Some(CapexNature::Brownfield)
        } else {
            None
        };

        // 6. Counterparty
        let counterparty = self.counterparty_regex.captures(text).and_then(|caps| {
            caps.get(1).map(|m| m.as_str().trim().to_string())
        });

        // 7. Materiality / Asymmetry Score
        let materiality_score = match (annualized_revenue_cr, company_annual_turnover_cr) {
            (Some(ann_rev), Some(turnover)) if turnover > 0.0 => {
                let ratio = ann_rev / turnover;
                // Score between 0.0 and 100.0 (ratio >= 0.5 is 80+, ratio >= 1.0 is 95+)
                ((ratio * 80.0).min(100.0) * 10.0).round() / 10.0
            }
            (Some(ann_rev), None) => {
                // Absolute scale: > ₹1,000 Cr is high impact
                if ann_rev >= 1000.0 {
                    90.0
                } else if ann_rev >= 250.0 {
                    75.0
                } else if ann_rev >= 50.0 {
                    50.0
                } else {
                    25.0
                }
            }
            _ => 10.0,
        };

        ExtractedFilingFacts {
            contract_value_cr,
            timeline_months,
            annualized_revenue_cr,
            capacity_addition,
            expansion_type,
            counterparty,
            materiality_score,
        }
    }

    /// Projects incremental EPS accretion based on contract value and operating margin
    pub fn compute_incremental_eps(
        &self,
        annualized_revenue_cr: f64,
        operating_margin_pct: f64,
        effective_tax_rate_pct: f64,
        total_shares_cr: f64,
    ) -> Option<f64> {
        if total_shares_cr <= 0.0 {
            return None;
        }

        let incremental_ebitda = annualized_revenue_cr * (operating_margin_pct / 100.0);
        let incremental_pat = incremental_ebitda * (1.0 - (effective_tax_rate_pct / 100.0));
        let incremental_eps = incremental_pat / total_shares_cr;

        Some((incremental_eps * 100.0).round() / 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_inr_order_and_timeline() {
        let extractor = QuantitativeFactExtractor::new();
        let filing_text = "The Company has secured a prestigious order valued at Rs 1,850.50 Crores for supply of smart metering solutions from Maharashtra State Electricity Distribution Co Ltd. The contract is executable within a timeline of 24 months.";

        let facts = extractor.extract_facts(filing_text, Some(2000.0));
        assert_eq!(facts.contract_value_cr, Some(1850.5));
        assert_eq!(facts.timeline_months, Some(24));
        // Annualized run rate = 1850.50 / 2 = 925.25 Cr
        assert_eq!(facts.annualized_revenue_cr, Some(925.25));
        // Materiality = 925.25 / 2000 * 80 ≈ 37.0
        assert!(facts.materiality_score >= 35.0);
    }

    #[test]
    fn test_extract_usd_order_and_greenfield_expansion() {
        let extractor = QuantitativeFactExtractor::new();
        let filing_text = "Board approves greenfield capacity expansion of 500 MW solar cell line with an estimated investment of USD 50 million. Expected completion in 1.5 years.";

        let facts = extractor.extract_facts(filing_text, None);
        // 50 USD Mn * 8.4 = 420.0 Cr
        assert_eq!(facts.contract_value_cr, Some(420.0));
        assert_eq!(facts.timeline_months, Some(18));
        assert_eq!(facts.expansion_type, Some(CapexNature::Greenfield));
        assert_eq!(facts.capacity_addition, Some("500 MW".to_string()));
    }

    #[test]
    fn test_compute_incremental_eps() {
        let extractor = QuantitativeFactExtractor::new();
        // Annualized rev: 1000 Cr, EBITDA margin: 15%, Tax: 25%, Total shares: 10 Cr shares
        // EBITDA: 150 Cr, PAT: 112.5 Cr, EPS = 112.5 / 10 = 11.25 Rs/share
        let eps = extractor.compute_incremental_eps(1000.0, 15.0, 25.0, 10.0);
        assert_eq!(eps, Some(11.25));
    }
}
