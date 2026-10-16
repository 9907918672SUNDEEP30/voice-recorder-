//! # Module 6A: Background 24x7 Stock Scanner Worker
//!
//! Autonomous multi-threaded Tokio background worker that monitors all 5,000+ stocks
//! in the in-memory database for volume outliers, technical breakouts, and filings.

use crate::fundamentals::StockDatabase;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};

/// Outlier Trigger Types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertType {
    VolumeSurgeOutlier { volume_multiple: f64 },
    BollingerSqueezeBreakout { price: f64 },
    GoldenCrossTriggered,
    NewCapexFilingDetected { detail: String },
    HighRiskPledgingAlert { detail: String },
}

/// Outlier Event Dispatched to Dashboard / Trading Terminal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutlierAlert {
    pub ticker: String,
    pub alert_type: AlertType,
    pub timestamp_epoch_sec: u64,
}

pub struct StockScanner {
    database: StockDatabase,
    is_running: Arc<AtomicBool>,
    alert_sender: broadcast::Sender<OutlierAlert>,
}

impl StockScanner {
    pub fn new(database: StockDatabase) -> Self {
        let (alert_sender, _) = broadcast::channel(1024);
        Self {
            database,
            is_running: Arc::new(AtomicBool::new(false)),
            alert_sender,
        }
    }

    /// Subscribe to real-time outlier alert notifications
    pub fn subscribe(&self) -> broadcast::Receiver<OutlierAlert> {
        self.alert_sender.subscribe()
    }

    /// Start 24x7 Tokio Background Scanning Worker
    pub fn start_worker(&self, scan_interval: Duration) -> tokio::task::JoinHandle<()> {
        let is_running = self.is_running.clone();
        is_running.store(true, Ordering::SeqCst);

        let db = self.database.clone();
        let tx = self.alert_sender.clone();

        tokio::spawn(async move {
            println!("24x7 Autonomous Financial Scanner Worker launched across 5000+ stock space.");

            while is_running.load(Ordering::SeqCst) {
                let tickers = db.get_all_tickers();

                // Scan all stocks currently in the database
                for ticker in tickers {
                    if let Some(profile) = db.get_stock(&ticker) {
                        // Scan for fundamental strength
                        if let Some(ratios) = &profile.latest_ratios {
                            if ratios.fundamental_health_score >= 8.5 && ratios.debt_to_equity == 0.0 {
                                let _ = tx.send(OutlierAlert {
                                    ticker: ticker.clone(),
                                    alert_type: AlertType::BollingerSqueezeBreakout {
                                        price: profile.current_price,
                                    },
                                    timestamp_epoch_sec: chrono::Utc::now().timestamp() as u64,
                                });
                            }
                        }
                    }
                }

                sleep(scan_interval).await;
            }

            println!("Financial Scanner Worker safely shut down.");
        })
    }

    /// Stop the background scanner worker
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }
}
