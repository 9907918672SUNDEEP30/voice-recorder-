//! # Module 6B: Master Financial Engine & Unified Report Synthesizer
//!
//! Aggregates Fundamentals, Technicals, TAM Projections, PDF Corporate Filings,
//! and CDP Intelligence into an actionable `MasterStockReport` with deterministic
//! verdicts and sub-50ms JSON dashboard rendering.

use crate::cdp_scraper::{CdpAutonomousScraper, ScrapedIntelligence};
use crate::fundamentals::{FinancialRatioEngine, FinancialRatios, StockDatabase};
use crate::pdf_parser::{ExtractedFilingMetrics, PdfFilingParser};
use crate::tam_engine::{TamScenarioSet, TamSimulationEngine};
use crate::technicals::{OhlcvCandle, TechnicalEngine, TechnicalSummary};
use ai_core::models::onnx_adapters::BgeRerankerLocal;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::data_feeds::DataFeedPipeline;

/// Final Institutional Verdict
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MasterVerdict {
    StrongAccumulation, // > 8.0 Composite Score, Stellar ROCE + Strong Technical Breakout
    NeutralWatch,       // 5.0 - 7.9 Composite Score, Stable but awaiting catalysts
    AvoidRisk,          // < 5.0 Composite Score, High Debt, Weak Cash Flow, or Technical Breakdown
}

/// Unified Institutional Master Stock Report
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MasterStockReport {
    pub ticker: String,
    pub company_name: String,
    pub sector: String,
    pub current_price: f64,
    pub market_cap_cr: f64,
    pub technical_score: f32,          // 1.0 - 10.0
    pub fundamental_health_score: f32, // 1.0 - 10.0
    pub tam_growth_score: f32,         // 1.0 - 10.0
    pub composite_score: f32,          // 1.0 - 10.0 weighted aggregate
    pub master_verdict: MasterVerdict,
    pub computed_ratios: Option<FinancialRatios>,
    pub technical_summary: Option<TechnicalSummary>,
    pub tam_scenarios: Option<TamScenarioSet>,
    pub filing_metrics: Option<ExtractedFilingMetrics>,
    pub cdp_macro_intelligence: Option<ScrapedIntelligence>,
    pub key_catalysts: Vec<String>,
    pub red_flags: Vec<String>,
    pub generated_at_epoch_ms: u64,
}

/// The Core Master Financial Super Engine
pub struct FinancialSuperEngine {
    pub database: StockDatabase,
    pub pdf_parser: PdfFilingParser,
    pub cdp_scraper: CdpAutonomousScraper,
    pub data_feeds: DataFeedPipeline,
    local_reranker: Option<BgeRerankerLocal>,
}

impl Default for FinancialSuperEngine {
    fn default() -> Self {
        Self::new(None)
    }
}

impl FinancialSuperEngine {
    pub fn new(models_dir: Option<&Path>) -> Self {
        let database = StockDatabase::new();
        let pdf_parser = PdfFilingParser::new().unwrap_or_default();
        let cdp_scraper = CdpAutonomousScraper::new(models_dir);
        let data_feeds = DataFeedPipeline::new();
        let local_reranker = PdfFilingParser::try_load_reranker(models_dir);

        // Pre-seed primary Indian equity universe into in-memory database
        data_feeds.seed_indian_equities_universe(&database);

        Self {
            database,
            pdf_parser,
            cdp_scraper,
            data_feeds,
            local_reranker,
        }
    }

    /// Generates a comprehensive Master Stock Report with deterministic scoring (<50ms execution)
    pub async fn analyze_stock(
        &self,
        ticker: &str,
        candles: &[OhlcvCandle],
        filing_text: Option<&str>,
        macro_news_url: Option<&str>,
    ) -> Result<MasterStockReport> {
        let profile = self
            .database
            .get_stock(ticker)
            .context(format!("Stock ticker {} not found in in-memory database", ticker))?;

        // 1. Compute Fundamental Ratios
        let ratios = FinancialRatioEngine::compute_ratios(&profile);
        let fund_score = ratios
            .as_ref()
            .map(|r| r.fundamental_health_score)
            .unwrap_or(5.0);

        // 2. Evaluate Technical Indicators & Patterns
        let tech_summary = if !candles.is_empty() {
            Some(TechnicalEngine::evaluate_technicals(candles))
        } else {
            None
        };
        let tech_score = tech_summary
            .as_ref()
            .map(|t| t.technical_score)
            .unwrap_or(5.0);

        // 3. Simulate TAM Velocity
        let base_sales = profile
            .income_statements
            .last()
            .map(|i| i.sales)
            .unwrap_or(1000.0);
        let base_ebitda_margin = profile
            .income_statements
            .last()
            .map(|i| i.opm_percent)
            .unwrap_or(15.0);
        let base_pat_margin = profile
            .income_statements
            .last()
            .map(|i| (i.net_profit / i.sales.max(1.0)) * 100.0)
            .unwrap_or(10.0);

        let tam_set = TamSimulationEngine::simulate(
            base_sales,
            base_ebitda_margin,
            base_pat_margin,
            profile.market_cap_cr / profile.current_price.max(1.0), // Approximate shares
            base_sales * 15.0,                                      // Industry TAM estimate
            6.67,                                                   // Current market share estimate
            14.0,                                                   // 14% Industry CAGR
        );
        let tam_score = tam_set.tam_growth_score;

        // 4. Parse Corporate PDF Disclosures if supplied
        let filing_metrics = if let Some(text) = filing_text {
            Some(
                self.pdf_parser
                    .parse_filing_disclosures(text, self.local_reranker.as_ref())
                    .await,
            )
        } else {
            None
        };

        // 5. Gather Macro Web Intelligence via fast scrape or CDP
        let cdp_intelligence = if let Some(url) = macro_news_url {
            self.cdp_scraper.scrape_with_fast_http(url).await.ok()
        } else {
            None
        };

        // 6. Synthesize Catalysts & Red Flags
        let mut key_catalysts = Vec::new();
        let mut red_flags = Vec::new();

        if let Some(r) = &ratios {
            if r.roce_percent >= 20.0 {
                key_catalysts.push(format!("High-quality ROCE of {:.1}%", r.roce_percent));
            }
            if r.debt_to_equity <= 0.2 {
                key_catalysts.push("Virtually debt-free balance sheet".to_string());
            } else if r.debt_to_equity > 1.2 {
                red_flags.push(format!("Elevated Debt-to-Equity of {:.2}x", r.debt_to_equity));
            }
            if r.cfo_to_net_profit < 0.6 {
                red_flags.push(format!("Weak cash conversion: CFO/PAT only {:.2}x", r.cfo_to_net_profit));
            }
        }

        if let Some(t) = &tech_summary {
            if t.is_golden_cross {
                key_catalysts.push("Bullish 50/200 EMA Golden Cross active".to_string());
            }
            for pattern in &t.detected_patterns {
                key_catalysts.push(format!("Technical pattern: {:?}", pattern.pattern));
            }
        }

        if let Some(f) = &filing_metrics {
            if !f.capex_disclosures.is_empty() {
                key_catalysts.push(format!("Capex initiative disclosed: {}", f.capex_disclosures[0]));
            }
            if !f.promoter_pledge_disclosures.is_empty() {
                red_flags.push(format!("Promoter pledge risk detected: {}", f.promoter_pledge_disclosures[0]));
            }
        }

        // 7. Weighted Composite Score
        // 40% Fundamentals, 35% Technicals, 25% TAM Growth
        let composite_score = ((fund_score * 0.40) + (tech_score * 0.35) + (tam_score * 0.25)).clamp(1.0, 10.0);

        let master_verdict = if composite_score >= 7.8 && red_flags.is_empty() {
            MasterVerdict::StrongAccumulation
        } else if composite_score < 4.8 || red_flags.len() >= 2 {
            MasterVerdict::AvoidRisk
        } else {
            MasterVerdict::NeutralWatch
        };

        let generated_at_epoch_ms = chrono::Utc::now().timestamp_millis() as u64;

        Ok(MasterStockReport {
            ticker: ticker.to_string(),
            company_name: profile.company_name,
            sector: profile.sector,
            current_price: profile.current_price,
            market_cap_cr: profile.market_cap_cr,
            technical_score: tech_score,
            fundamental_health_score: fund_score,
            tam_growth_score: tam_score,
            composite_score,
            master_verdict,
            computed_ratios: ratios,
            technical_summary: tech_summary,
            tam_scenarios: Some(tam_set),
            filing_metrics,
            cdp_macro_intelligence: cdp_intelligence,
            key_catalysts,
            red_flags,
            generated_at_epoch_ms,
        })
    }

    /// Serializes report to JSON ready for PWA / Web dashboard (<50ms latency)
    pub fn to_dashboard_json(&self, report: &MasterStockReport) -> Result<String> {
        serde_json::to_string_pretty(report).context("Failed to serialize MasterStockReport to JSON")
    }
}
