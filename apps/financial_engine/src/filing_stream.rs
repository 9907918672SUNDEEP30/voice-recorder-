//! # High-Throughput In-Memory Corporate Filing Streaming Engine
//!
//! Sub-second streaming ingestion, triage, and in-memory parsing for 5,000+ daily
//! NSE & BSE corporate filings. Zero disk I/O: processes PDF bytes entirely in RAM.

use anyhow::{Context, Result};
use lopdf::Document;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc};

/// Category of Exchange Filing under SEBI Regulation 30 (LODR)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilingCategory {
    /// Significant commercial order win, purchase order, or global tender award
    OrderWin,
    /// Greenfield / brownfield plant setup, capacity expansion, or mega capex
    CapexExpansion,
    /// Financial results, quarterly earnings, limited review, or investor presentation
    FinancialResults,
    /// Appointment, resignation, or removal of Key Managerial Personnel (CEO, MD, CFO, Auditor)
    LeadershipChange,
    /// Mergers, demergers, acquisitions, joint ventures, or strategic restructuring
    MergerAcquisition,
    /// Credit rating upgrade or downgrade
    CreditRating,
    /// Tax notice, SEBI enquiry, regulatory penalty, or litigation notice
    RegulatoryAction,
    /// Routine statutory notice (Loss of share certificate, postal ballot, AGM notice)
    RoutineNotice,
}

/// Triage priority for pipeline processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriagePriority {
    /// Immediate price-sensitive catalyst (process through AI and alert in <2s)
    HighPriority,
    /// Standard statutory disclosure (archive without heavy AI processing)
    LowPriority,
}

/// Raw filing metadata received from Exchange Announcement Feed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExchangeFiling {
    pub filing_id: String,
    pub exchange: String, // "NSE" or "BSE"
    pub symbol: String,   // e.g. "DIXON", "TCS", "500325"
    pub company_name: String,
    pub category: FilingCategory,
    pub priority: TriagePriority,
    pub headline: String,
    pub pdf_url: Option<String>,
    pub filing_time_epoch_ms: u64,
}

/// Processed filing output containing extracted text and numerical facts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessedFilingAlert {
    pub filing_id: String,
    pub symbol: String,
    pub company_name: String,
    pub category: FilingCategory,
    pub headline: String,
    pub extracted_text_snippet: String,
    pub detected_order_value_cr: Option<f64>,
    pub detected_capex_cr: Option<f64>,
    pub execution_time_ms: u64,
    pub filing_time_epoch_ms: u64,
}

/// Fast Regex-based Triage Classifier for SEBI LODR Filings
pub struct FilingTriageClassifier {
    order_regex: Regex,
    capex_regex: Regex,
    results_regex: Regex,
    leadership_regex: Regex,
    ma_regex: Regex,
    credit_regex: Regex,
    regulatory_regex: Regex,
    routine_regex: Regex,
}

impl Default for FilingTriageClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl FilingTriageClassifier {
    pub fn new() -> Self {
        let order_regex = Regex::new(
            r"(?i)(?:bags?|bagged|awarded|receives?|received|secures?|secured|wins?|won)(?:[\s\w,]{0,60}?)(?:order|contract|tender|mandate|project|agreement)|order\s+win|purchase\s+order",
        ).unwrap();

        let capex_regex = Regex::new(
            r"(?i)capacity\s+expansion|greenfield|brownfield|commercial\s+production|new\s+manufacturing\s+facility|capex|capital\s+expenditure",
        ).unwrap();

        let results_regex = Regex::new(
            r"(?i)financial\s+results|outcome\s+of\s+board\s+meeting|investor\s+presentation|earnings\s+call|press\s+release",
        ).unwrap();

        let leadership_regex = Regex::new(
            r"(?i)(?:appointment|resignation|cessation|re-appointment)\s+of\s+(?:managing\s+director|chief\s+executive|chief\s+financial|director|kmp|cfo|ceo|auditor)",
        ).unwrap();

        let ma_regex = Regex::new(
            r"(?i)acquisition|amalgamation|merger|demerger|joint\s+venture|slump\s+sale|strategic\s+investment",
        ).unwrap();

        let credit_regex = Regex::new(
            r"(?i)credit\s+rating|rating\s+reaffirmed|rating\s+upgraded|rating\s+downgraded|crisil|icra|care\s+ratings",
        ).unwrap();

        let regulatory_regex = Regex::new(
            r"(?i)tax\s+demand|gst\s+notice|penalty|sebi\s+order|show\s+cause|litigation|search\s+and\s+seizure",
        ).unwrap();

        let routine_regex = Regex::new(
            r"(?i)loss\s+of\s+share|duplicate\s+share|postal\s+ballot|annual\s+general\s+meeting|voting\s+results|compliance\s+certificate|newspaper\s+publication",
        ).unwrap();

        Self {
            order_regex,
            capex_regex,
            results_regex,
            leadership_regex,
            ma_regex,
            credit_regex,
            regulatory_regex,
            routine_regex,
        }
    }

    /// Classifies an exchange announcement by headline in under 1 millisecond
    pub fn classify(&self, headline: &str) -> (FilingCategory, TriagePriority) {
        if self.order_regex.is_match(headline) {
            (FilingCategory::OrderWin, TriagePriority::HighPriority)
        } else if self.capex_regex.is_match(headline) {
            (FilingCategory::CapexExpansion, TriagePriority::HighPriority)
        } else if self.leadership_regex.is_match(headline) {
            (FilingCategory::LeadershipChange, TriagePriority::HighPriority)
        } else if self.ma_regex.is_match(headline) {
            (FilingCategory::MergerAcquisition, TriagePriority::HighPriority)
        } else if self.regulatory_regex.is_match(headline) {
            (FilingCategory::RegulatoryAction, TriagePriority::HighPriority)
        } else if self.results_regex.is_match(headline) {
            (FilingCategory::FinancialResults, TriagePriority::HighPriority)
        } else if self.credit_regex.is_match(headline) {
            (FilingCategory::CreditRating, TriagePriority::HighPriority)
        } else if self.routine_regex.is_match(headline) {
            (FilingCategory::RoutineNotice, TriagePriority::LowPriority)
        } else {
            // Default unclassified filings are treated as standard notices
            (FilingCategory::RoutineNotice, TriagePriority::LowPriority)
        }
    }
}

/// In-Memory PDF Text Extractor using pure Rust `lopdf` (Zero Disk I/O)
pub struct InMemPdfExtractor;

impl InMemPdfExtractor {
    /// Extracts semantic plain text from raw PDF bytes resident in memory
    pub fn extract_text(pdf_bytes: &[u8]) -> Result<String> {
        let cursor = Cursor::new(pdf_bytes);
        let doc = Document::load_from(cursor).context("Failed to load PDF bytes with lopdf")?;

        let mut full_text = String::new();
        for page_num in doc.get_pages().keys() {
            if let Ok(text) = doc.extract_text(&[*page_num]) {
                full_text.push_str(&text);
                full_text.push('\n');
            }
        }

        let cleaned = full_text.split_whitespace().collect::<Vec<&str>>().join(" ");
        Ok(cleaned)
    }

    /// Quick numerical regex to detect order size or capex value in ₹ Crores
    pub fn extract_value_in_crores(text: &str) -> Option<f64> {
        let val_regex = Regex::new(
            r"(?i)(?:rs\.?|inr|₹)?\s*(\d+(?:,\d+)*(?:\.\d+)?)\s*(?:cr|crore|crores)",
        ).ok()?;

        if let Some(caps) = val_regex.captures(text) {
            if let Some(m) = caps.get(1) {
                let num_str = m.as_str().replace(',', "");
                if let Ok(val) = num_str.parse::<f64>() {
                    return Some(val);
                }
            }
        }
        None
    }
}

/// High-Throughput 24x7 Filing Streaming Worker
pub struct FilingStreamWorker {
    http_client: reqwest::Client,
    classifier: Arc<FilingTriageClassifier>,
    alert_sender: broadcast::Sender<ProcessedFilingAlert>,
    is_running: Arc<AtomicBool>,
    total_filings_processed: Arc<AtomicU64>,
    total_price_sensitive_alerts: Arc<AtomicU64>,
}

impl Default for FilingStreamWorker {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl FilingStreamWorker {
    pub fn new(channel_capacity: usize) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) ShambhaviFinancialSuperEngine/1.0")
            .build()
            .unwrap_or_default();

        let (alert_sender, _) = broadcast::channel(channel_capacity);

        Self {
            http_client,
            classifier: Arc::new(FilingTriageClassifier::new()),
            alert_sender,
            is_running: Arc::new(AtomicBool::new(false)),
            total_filings_processed: Arc::new(AtomicU64::new(0)),
            total_price_sensitive_alerts: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Subscribes to the real-time stream of high-priority filing alerts
    pub fn subscribe(&self) -> broadcast::Receiver<ProcessedFilingAlert> {
        self.alert_sender.subscribe()
    }

    /// Downloads raw filing PDF bytes directly to memory buffer (Zero disk I/O)
    pub async fn download_pdf_in_memory(&self, pdf_url: &str) -> Result<Vec<u8>> {
        let resp = self.http_client.get(pdf_url).send().await.context("Failed to download PDF bytes")?;
        let bytes = resp.bytes().await.context("Failed to read PDF stream into memory")?;
        Ok(bytes.to_vec())
    }

    /// Total count of corporate filings processed since worker start
    pub fn total_processed(&self) -> u64 {
        self.total_filings_processed.load(Ordering::Relaxed)
    }

    /// Total count of high-priority / price-sensitive catalysts detected
    pub fn total_price_sensitive_detected(&self) -> u64 {
        self.total_price_sensitive_alerts.load(Ordering::Relaxed)
    }

    /// Processes a single incoming filing synchronously in memory with sub-second execution
    pub async fn process_incoming_filing(&self, filing: ExchangeFiling, raw_pdf_bytes: Option<&[u8]>) -> Result<Option<ProcessedFilingAlert>> {
        let start_time = Instant::now();
        self.total_filings_processed.fetch_add(1, Ordering::Relaxed);

        // Tier 1: Triage classification
        let (category, priority) = self.classifier.classify(&filing.headline);

        // If routine legal notice without market-moving impact, bypass heavy processing
        if priority == TriagePriority::LowPriority && raw_pdf_bytes.is_none() {
            return Ok(None);
        }

        // Tier 2: In-memory text extraction from PDF bytes or headline snippet
        let (text_snippet, detected_order_val, detected_capex_val) = if let Some(bytes) = raw_pdf_bytes {
            let extracted = InMemPdfExtractor::extract_text(bytes).unwrap_or_else(|_| filing.headline.clone());
            let order_val = if category == FilingCategory::OrderWin {
                InMemPdfExtractor::extract_value_in_crores(&extracted)
            } else {
                None
            };
            let capex_val = if category == FilingCategory::CapexExpansion {
                InMemPdfExtractor::extract_value_in_crores(&extracted)
            } else {
                None
            };
            let snippet = extracted.chars().take(500).collect::<String>();
            (snippet, order_val, capex_val)
        } else {
            let order_val = if category == FilingCategory::OrderWin {
                InMemPdfExtractor::extract_value_in_crores(&filing.headline)
            } else {
                None
            };
            let capex_val = if category == FilingCategory::CapexExpansion {
                InMemPdfExtractor::extract_value_in_crores(&filing.headline)
            } else {
                None
            };
            (filing.headline.clone(), order_val, capex_val)
        };

        let elapsed_ms = start_time.elapsed().as_millis() as u64;

        let alert = ProcessedFilingAlert {
            filing_id: filing.filing_id,
            symbol: filing.symbol,
            company_name: filing.company_name,
            category,
            headline: filing.headline,
            extracted_text_snippet: text_snippet,
            detected_order_value_cr: detected_order_val,
            detected_capex_cr: detected_capex_val,
            execution_time_ms: elapsed_ms,
            filing_time_epoch_ms: filing.filing_time_epoch_ms,
        };

        if priority == TriagePriority::HighPriority {
            self.total_price_sensitive_alerts.fetch_add(1, Ordering::Relaxed);
            let _ = self.alert_sender.send(alert.clone());
        }

        Ok(Some(alert))
    }

    /// Starts an autonomous background ingestion worker polling an inbound queue
    pub fn start_worker(&self, mut inbound_rx: mpsc::Receiver<(ExchangeFiling, Option<Vec<u8>>)>) -> tokio::task::JoinHandle<()> {
        let is_running = self.is_running.clone();
        is_running.store(true, Ordering::SeqCst);

        let classifier = self.classifier.clone();
        let alert_sender = self.alert_sender.clone();
        let processed_counter = self.total_filings_processed.clone();
        let price_sensitive_counter = self.total_price_sensitive_alerts.clone();

        tokio::spawn(async move {
            println!("🚀 [FilingStreamWorker] Autonomous in-memory corporate filing pipeline active.");
            while is_running.load(Ordering::Relaxed) {
                match inbound_rx.recv().await {
                    Some((filing, pdf_bytes_opt)) => {
                        let start = Instant::now();
                        processed_counter.fetch_add(1, Ordering::Relaxed);

                        let (category, priority) = classifier.classify(&filing.headline);
                        if priority == TriagePriority::HighPriority || pdf_bytes_opt.is_some() {
                            let (snippet, order_val, capex_val) = if let Some(ref bytes) = pdf_bytes_opt {
                                let text = InMemPdfExtractor::extract_text(bytes).unwrap_or_else(|_| filing.headline.clone());
                                let o_val = InMemPdfExtractor::extract_value_in_crores(&text);
                                let c_val = InMemPdfExtractor::extract_value_in_crores(&text);
                                let snip = text.chars().take(400).collect::<String>();
                                (snip, o_val, c_val)
                            } else {
                                let o_val = InMemPdfExtractor::extract_value_in_crores(&filing.headline);
                                let c_val = InMemPdfExtractor::extract_value_in_crores(&filing.headline);
                                (filing.headline.clone(), o_val, c_val)
                            };

                            let alert = ProcessedFilingAlert {
                                filing_id: filing.filing_id,
                                symbol: filing.symbol,
                                company_name: filing.company_name,
                                category,
                                headline: filing.headline,
                                extracted_text_snippet: snippet,
                                detected_order_value_cr: order_val,
                                detected_capex_cr: capex_val,
                                execution_time_ms: start.elapsed().as_millis() as u64,
                                filing_time_epoch_ms: filing.filing_time_epoch_ms,
                            };

                            price_sensitive_counter.fetch_add(1, Ordering::Relaxed);
                            let _ = alert_sender.send(alert);
                        }
                    }
                    None => {
                        // Channel closed
                        break;
                    }
                }
            }
            println!("🛑 [FilingStreamWorker] Worker terminated.");
        })
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triage_classification_speed_and_accuracy() {
        let classifier = FilingTriageClassifier::new();

        // High Priority Orders
        let (cat, prio) = classifier.classify("Dixon Technologies bags fresh mobile manufacturing order of Rs 1,450 Cr from global OEM");
        assert_eq!(cat, FilingCategory::OrderWin);
        assert_eq!(prio, TriagePriority::HighPriority);

        // High Priority Capex
        let (cat, prio) = classifier.classify("Kaynes Technology announces greenfield capacity expansion for semiconductor OSAT facility in Gujarat");
        assert_eq!(cat, FilingCategory::CapexExpansion);
        assert_eq!(prio, TriagePriority::HighPriority);

        // High Priority Leadership Change
        let (cat, prio) = classifier.classify("Cessation and Resignation of Chief Financial Officer (CFO) under Regulation 30");
        assert_eq!(cat, FilingCategory::LeadershipChange);
        assert_eq!(prio, TriagePriority::HighPriority);

        // Low Priority Routine Notice
        let (cat, prio) = classifier.classify("Intimation of Loss of Share Certificate under Regulation 39(3)");
        assert_eq!(cat, FilingCategory::RoutineNotice);
        assert_eq!(prio, TriagePriority::LowPriority);
    }

    #[tokio::test]
    async fn test_in_memory_filing_processing_latency() {
        let worker = FilingStreamWorker::new(100);
        let mut alert_rx = worker.subscribe();

        let filing = ExchangeFiling {
            filing_id: "BSE_ANN_982312".to_string(),
            exchange: "BSE".to_string(),
            symbol: "DIXON".to_string(),
            company_name: "Dixon Technologies (India) Ltd".to_string(),
            category: FilingCategory::OrderWin,
            priority: TriagePriority::HighPriority,
            headline: "Dixon bags contract valued at Rs 2,400 crore for electronic hardware manufacturing".to_string(),
            pdf_url: Some("https://www.bseindia.com/xml-data/corpfiling/AttachLive/sample.pdf".to_string()),
            filing_time_epoch_ms: 1729240000000,
        };

        let start = Instant::now();
        let res = worker.process_incoming_filing(filing, None).await.expect("Failed to process filing");
        let elapsed = start.elapsed();

        assert!(res.is_some());
        let alert = res.unwrap();
        assert_eq!(alert.symbol, "DIXON");
        assert_eq!(alert.category, FilingCategory::OrderWin);
        assert_eq!(alert.detected_order_value_cr, Some(2400.0));
        assert!(elapsed < Duration::from_millis(50), "In-memory processing must execute in < 50ms, took {:?}", elapsed);

        // Verify broadcast delivery
        let broadcasted = alert_rx.try_recv().expect("Expected broadcasted alert");
        assert_eq!(broadcasted.symbol, "DIXON");
        assert_eq!(worker.total_processed(), 1);
        assert_eq!(worker.total_price_sensitive_detected(), 1);
    }
}
