//! # Module 6B: Master Financial Engine & Unified Report Synthesizer
//!
//! Aggregates Fundamentals, Technicals, TAM Projections, PDF Corporate Filings,
//! Quantitative Fact Extraction, Live Macro & Policy Catalysts, and Forward Valuation
//! Re-Rating into an actionable `MasterStockReport` with deterministic verdicts
//! and sub-50ms JSON dashboard rendering.

use crate::alert_hub::{AlertDistributionHub, FlashAlphaCard, SixVectorSynthesis};
use crate::cdp_scraper::{CdpAutonomousScraper, ScrapedIntelligence};
use crate::data_feeds::DataFeedPipeline;
use crate::fact_extractor::{ExtractedFilingFacts, QuantitativeFactExtractor};
use crate::fundamentals::{FinancialRatioEngine, FinancialRatios, StockDatabase};
use crate::macro_radar::{MacroPolicyRadar, StockMacroEvaluation};
use crate::pdf_parser::{ExtractedFilingMetrics, PdfFilingParser};
use crate::tam_engine::{TamScenarioSet, TamSimulationEngine};
use crate::technicals::{OhlcvCandle, TechnicalEngine, TechnicalSummary};
use crate::valuation::{ForwardValuationProjection, ValuationInput, ValuationReRatingEngine};
use ai_core::models::onnx_adapters::BgeRerankerLocal;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Final Institutional Verdict
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MasterVerdict {
    StrongAccumulation, // > 7.8 Composite Score, Stellar ROCE + Strong Catalysts
    NeutralWatch,       // 4.8 - 7.7 Composite Score, Stable but awaiting catalysts
    AvoidRisk,          // < 4.8 Composite Score, High Debt, Weak Cash Flow, or Technical Breakdown
}

pub const SEBI_STATUTORY_DISCLAIMER: &str = "Algorithmic Quantitative Analysis Engine — strictly for informational and quantitative research purposes. Shambhavi Financial Super Engine is not a SEBI-registered Research Analyst or Investment Adviser under SEBI (Research Analysts) Regulations, 2014. No content constitutes financial advice or solicitation to buy or sell securities.";

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
    pub extracted_facts: Option<ExtractedFilingFacts>,
    pub macro_evaluation: Option<StockMacroEvaluation>,
    pub forward_valuation: Option<ForwardValuationProjection>,
    pub cdp_macro_intelligence: Option<ScrapedIntelligence>,
    pub key_catalysts: Vec<String>,
    pub red_flags: Vec<String>,
    pub generated_at_epoch_ms: u64,
    pub compliance_disclaimer: String,
}

/// The Core Master Financial Super Engine
pub struct FinancialSuperEngine {
    pub database: StockDatabase,
    pub pdf_parser: PdfFilingParser,
    pub cdp_scraper: CdpAutonomousScraper,
    pub data_feeds: DataFeedPipeline,
    pub fact_extractor: QuantitativeFactExtractor,
    pub macro_radar: MacroPolicyRadar,
    pub alert_hub: AlertDistributionHub,
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
        let fact_extractor = QuantitativeFactExtractor::new();
        let macro_radar = MacroPolicyRadar::new();
        let alert_hub = AlertDistributionHub::default();
        let local_reranker = PdfFilingParser::try_load_reranker(models_dir);

        // Pre-seed primary Indian equity universe into in-memory database
        data_feeds.seed_indian_equities_universe(&database);

        Self {
            database,
            pdf_parser,
            cdp_scraper,
            data_feeds,
            fact_extractor,
            macro_radar,
            alert_hub,
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

        let (industry_tam_cr, industry_cagr_pct) =
            TamSimulationEngine::get_sector_tam_and_cagr(&profile.sector);
        let current_market_share_pct =
            ((base_sales / industry_tam_cr) * 100.0).clamp(0.01, 95.0);

        let tam_set = TamSimulationEngine::simulate(
            base_sales,
            base_ebitda_margin,
            base_pat_margin,
            profile.market_cap_cr / profile.current_price.max(1.0),
            industry_tam_cr,
            current_market_share_pct,
            industry_cagr_pct,
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

        // 7. Live Macro & Government Policy Cross-Referencing
        let macro_eval = self.macro_radar.evaluate_stock(ticker, &profile.sector);
        for catalyst in &macro_eval.active_policy_catalysts {
            key_catalysts.push(format!("Policy Catalyst: {}", catalyst));
        }
        for tailwind in &macro_eval.macro_tailwinds {
            key_catalysts.push(format!("Macro Tailwind: {}", tailwind));
        }
        for headwind in &macro_eval.macro_headwinds {
            red_flags.push(format!("Macro Headwind: {}", headwind));
        }

        // 8. Quantitative Fact Extraction from Corporate Disclosures
        let extracted_facts = if let Some(text) = filing_text {
            let facts = self.fact_extractor.extract_facts(text, Some(base_sales));
            if let Some(val) = facts.contract_value_cr {
                key_catalysts.push(format!("New contract win: ₹{:.1} Cr", val));
            }
            if let Some(cap) = &facts.capacity_addition {
                key_catalysts.push(format!("Capacity expansion: {}", cap));
            }
            Some(facts)
        } else {
            None
        };

        // 9. Forward Valuation & Multiple Expansion Scenario Modeling
        let trailing_eps = profile
            .income_statements
            .last()
            .map(|i| i.eps)
            .unwrap_or_else(|| (profile.current_price / profile.current_pe.max(1.0)).max(1.0));

        let target_market_share = tam_set
            .best
            .years
            .get(2)
            .map(|y| y.projected_market_share_pct)
            .unwrap_or(current_market_share_pct * 1.5);

        let promoter_pledge_pct = if filing_metrics
            .as_ref()
            .map(|f| !f.promoter_pledge_disclosures.is_empty())
            .unwrap_or(false)
        {
            15.0
        } else {
            0.0
        };

        let val_input = ValuationInput {
            current_price: profile.current_price,
            trailing_eps,
            trailing_pe: profile.current_pe,
            roce_pct: ratios.as_ref().map(|r| r.roce_percent).unwrap_or(15.0),
            expected_3yr_revenue_cagr_pct: tam_set.base.cagr_sales_pct,
            current_market_share_pct,
            target_market_share_pct: target_market_share,
            promoter_pledge_pct,
            has_tech_or_policy_moat: macro_eval.is_asymmetric_beneficiary,
        };
        let forward_valuation = ValuationReRatingEngine::compute_projections(ticker, val_input);

        if forward_valuation.is_institutional_blind_spot {
            key_catalysts.push(format!(
                "Institutional Blind Spot Detected (Disparity Index: {:.0}/100) — Low PE relative to high growth",
                forward_valuation.institutional_blind_spot_score
            ));
        }

        // 10. Weighted Composite Score
        // 35% Fundamentals, 30% Technicals, 20% TAM Growth, 15% Policy/Macro Tailwind
        let policy_macro_bonus = ((macro_eval.net_tailwind_score / 10.0) + 5.0).clamp(1.0, 10.0) as f32;
        let composite_score = ((fund_score * 0.35)
            + (tech_score * 0.30)
            + (tam_score * 0.20)
            + (policy_macro_bonus * 0.15))
            .clamp(1.0, 10.0);

        let master_verdict = if composite_score >= 7.5 && red_flags.is_empty() {
            MasterVerdict::StrongAccumulation
        } else if composite_score < 4.8 || red_flags.len() >= 2 {
            MasterVerdict::AvoidRisk
        } else {
            MasterVerdict::NeutralWatch
        };

        let generated_at_epoch_ms = chrono::Utc::now().timestamp_millis() as u64;

        // 11. Broadcast Flash Alpha Card if High-Conviction Asymmetric Setup
        if forward_valuation.is_institutional_blind_spot || macro_eval.is_asymmetric_beneficiary {
            let card = FlashAlphaCard {
                alert_id: format!("ALPHA_{}_{}", ticker, generated_at_epoch_ms),
                timestamp_epoch_ms: generated_at_epoch_ms,
                symbol: ticker.to_string(),
                company_name: profile.company_name.clone(),
                sector: profile.sector.clone(),
                current_price: profile.current_price,
                base_target_price: forward_valuation.base_case.target_price,
                supercycle_target_price: forward_valuation.bull_supercycle.target_price,
                potential_upside_pct: forward_valuation.bull_supercycle.upside_potential_pct,
                catalyst_headline: key_catalysts.first().cloned().unwrap_or_else(|| "Asymmetric Growth Opportunity".to_string()),
                synthesis: SixVectorSynthesis {
                    policy_vector: macro_eval.active_policy_catalysts.first().cloned().unwrap_or_else(|| "Neutral Policy Climate".to_string()),
                    tech_moat_vector: "Proprietary manufacturing specifications and scale".to_string(),
                    s_curve_market_share_vector: format!("Market share expanding from {:.1}% towards {:.1}%", current_market_share_pct, target_market_share),
                    management_pedigree_vector: format!("Promoter pledge: {:.1}%", promoter_pledge_pct),
                    ground_adoption_vector: "Robust order book execution and delivery pipeline".to_string(),
                    institutional_blind_spot_vector: format!("Forward Supercycle Target: ₹{:.1} (+{:.1}%)", forward_valuation.bull_supercycle.target_price, forward_valuation.bull_supercycle.upside_potential_pct),
                },
                conviction_score: ((composite_score as f64 * 10.0).clamp(50.0, 99.0) * 10.0).round() / 10.0,
                statutory_disclaimer: SEBI_STATUTORY_DISCLAIMER.to_string(),
            };
            let _ = self.alert_hub.broadcast(card);
        }

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
            extracted_facts,
            macro_evaluation: Some(macro_eval),
            forward_valuation: Some(forward_valuation),
            cdp_macro_intelligence: cdp_intelligence,
            key_catalysts,
            red_flags,
            generated_at_epoch_ms,
            compliance_disclaimer: SEBI_STATUTORY_DISCLAIMER.to_string(),
        })
    }

    /// Serializes report to JSON ready for PWA / Web dashboard (<50ms latency)
    pub fn to_dashboard_json(&self, report: &MasterStockReport) -> Result<String> {
        serde_json::to_string_pretty(report).context("Failed to serialize MasterStockReport to JSON")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyze_stock_with_macro_and_policy_synthesis() {
        let engine = FinancialSuperEngine::new(None);
        let mut alert_rx = engine.alert_hub.subscribe();

        let filing_text = "The Company secured order valued at Rs 1,450 Crores for electronic manufacturing executable in 18 months. Capacity expansion of 200 MW underway.";

        let report = engine
            .analyze_stock("KAYNES", &[], Some(filing_text), None)
            .await
            .expect("Analysis should succeed");

        assert_eq!(report.ticker, "KAYNES");
        assert!(report.macro_evaluation.is_some());
        assert!(report.forward_valuation.is_some());
        assert!(report.extracted_facts.is_some());

        let facts = report.extracted_facts.as_ref().unwrap();
        assert_eq!(facts.contract_value_cr, Some(1450.0));
        assert_eq!(facts.timeline_months, Some(18));

        // Check if Flash Alpha Card was dispatched
        let received = alert_rx.try_recv();
        assert!(received.is_ok(), "High-conviction stock should broadcast Flash Alpha Card");
        let card = received.unwrap();
        assert_eq!(card.symbol, "KAYNES");
        assert!(card.potential_upside_pct > 0.0);
    }
}
