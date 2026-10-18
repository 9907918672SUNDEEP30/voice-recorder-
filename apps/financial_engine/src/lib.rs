//! # Pure Rust Financial Super Engine
//!
//! Autonomous, institutional-grade equity analysis crate for 5,000+ NSE/BSE listed stocks.
//! Features zero Python, 100% free public data architecture, deterministic mathematical
//! rule engines, headless Chrome DevTools Protocol (CDP) scraping, and local ONNX AI models
//! hosted in `C:\AI_Models`.

pub mod alert_hub;
pub mod cdp_scraper;
pub mod data_feeds;
pub mod engine;
pub mod fact_extractor;
pub mod filing_stream;
pub mod fundamentals;
pub mod macro_radar;
pub mod pdf_parser;
pub mod scanner;
pub mod tam_engine;
pub mod technicals;
pub mod valuation;

// Convenience re-exports
pub use alert_hub::{AlertDistributionHub, FlashAlphaCard, SixVectorSynthesis};
pub use cdp_scraper::{CdpAutonomousScraper, ScrapedIntelligence};
pub use data_feeds::{BhavcopyRecord, BseAnnouncement, DataFeedPipeline};
pub use engine::{FinancialSuperEngine, MasterStockReport, MasterVerdict};
pub use fact_extractor::{CapexNature, ExtractedFilingFacts, QuantitativeFactExtractor};
pub use filing_stream::{
    ExchangeFiling, FilingCategory, FilingStreamWorker, FilingTriageClassifier,
    ProcessedFilingAlert, TriagePriority,
};
pub use fundamentals::{
    BalanceSheet, CashFlowStatement, FinancialRatioEngine, FinancialRatios, IncomeStatement,
    StockDatabase, StockProfile,
};
pub use macro_radar::{
    GovernmentMinistry, GovernmentPolicyCircular, MacroIndicatorType, MacroObservation,
    MacroPolicyRadar, MacroTrend, PolicyCategory, PolicyImpact, StockMacroEvaluation,
};
pub use pdf_parser::{ExtractedFilingMetrics, FilingSentiment, PdfFilingParser};
pub use scanner::{AlertType, OutlierAlert, StockScanner};
pub use tam_engine::{ScenarioTrajectory, TamScenarioSet, TamSimulationEngine, YearProjection};
pub use technicals::{
    CandlePatternMatch, MacdOutput, OhlcvCandle, PatternType, TechnicalEngine, TechnicalSummary,
};
pub use valuation::{
    ForwardValuationProjection, ValuationInput, ValuationReRatingEngine, ValuationScenarioOutput,
};
