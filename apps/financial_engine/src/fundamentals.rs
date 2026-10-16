//! # Module 1: Fundamental Data Structs & Financial Ratio Engine
//!
//! Strongly-typed financial data structures, lock-free 5,000+ stock in-memory database
//! using `DashMap`, and high-precision deterministic ratio computation.

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Periodic Income Statement (Annual or Quarterly)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IncomeStatement {
    pub period_ending: String,
    pub sales: f64,             // In ₹ Crores
    pub ebitda: f64,            // In ₹ Crores
    pub opm_percent: f64,       // EBITDA / Sales * 100
    pub depreciation: f64,      // In ₹ Crores
    pub interest_expense: f64,  // In ₹ Crores
    pub profit_before_tax: f64, // In ₹ Crores
    pub tax_rate_percent: f64,  // Effective Tax Rate
    pub net_profit: f64,        // PAT In ₹ Crores
    pub eps: f64,               // Earnings Per Share in ₹
}

/// Periodic Balance Sheet
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BalanceSheet {
    pub period_ending: String,
    pub equity_share_capital: f64, // In ₹ Crores
    pub reserves_and_surplus: f64, // In ₹ Crores
    pub total_debt: f64,           // Long-term + Short-term borrowings
    pub fixed_assets: f64,         // Net block + CWIP
    pub working_capital: f64,      // Current Assets - Current Liabilities
    pub current_assets: f64,       // Total Current Assets
    pub current_liabilities: f64,  // Total Current Liabilities
    pub cash_and_equivalents: f64, // Liquid cash & bank balance
}

/// Periodic Cash Flow Statement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CashFlowStatement {
    pub period_ending: String,
    pub cfo: f64,            // Cash from Operating Activities (₹ Cr)
    pub cfi: f64,            // Cash from Investing Activities (₹ Cr)
    pub cff: f64,            // Cash from Financing Activities (₹ Cr)
    pub free_cash_flow: f64, // CFO - Capex
}

/// Deterministically Computed Financial Ratios
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinancialRatios {
    pub roce_percent: f64,             // Return on Capital Employed (EBIT / Capital Employed * 100)
    pub debt_to_equity: f64,           // Total Debt / Shareholders Equity
    pub opm_expansion_delta: f64,      // Current OPM% - Historical Median OPM%
    pub pe_mean_reversion_zscore: f64, // (Current PE - 5Y Median PE) / PE StdDev
    pub cfo_to_net_profit: f64,        // CFO / Net Profit (Earnings Quality check, > 1.0 is healthy)
    pub current_ratio: f64,            // Current Assets / Current Liabilities
    pub interest_coverage: f64,        // EBIT / Interest Expense
    pub fundamental_health_score: f32, // Deterministic score 1.0 - 10.0
}

/// Full Profile for a Single Listed Stock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockProfile {
    pub ticker: String,        // e.g. "TCS", "RELIANCE", "500325"
    pub exchange: String,      // "NSE" or "BSE"
    pub company_name: String,
    pub sector: String,
    pub current_price: f64,
    pub market_cap_cr: f64,
    pub current_pe: f64,
    pub pe_5yr_median: f64,
    pub pe_5yr_std: f64,
    pub income_statements: Vec<IncomeStatement>,
    pub balance_sheets: Vec<BalanceSheet>,
    pub cash_flows: Vec<CashFlowStatement>,
    pub latest_ratios: Option<FinancialRatios>,
}

/// Lock-free, High-Concurrency 5,000+ Stock In-Memory Database
#[derive(Clone)]
pub struct StockDatabase {
    stocks: Arc<DashMap<String, StockProfile>>,
}

impl Default for StockDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl StockDatabase {
    pub fn new() -> Self {
        // Pre-allocate capacity for 6,000 items (covers entire NSE/BSE universe)
        Self {
            stocks: Arc::new(DashMap::with_capacity(6000)),
        }
    }

    /// Upsert stock profile into the in-memory database
    pub fn upsert_stock(&self, profile: StockProfile) {
        self.stocks.insert(profile.ticker.clone(), profile);
    }

    /// Batch upsert a collection of stock profiles
    pub fn batch_upsert(&self, profiles: Vec<StockProfile>) {
        for profile in profiles {
            self.upsert_stock(profile);
        }
    }

    /// Retrieve a stock profile by ticker symbol
    pub fn get_stock(&self, ticker: &str) -> Option<StockProfile> {
        self.stocks.get(ticker).map(|entry| entry.value().clone())
    }

    /// Total count of stocks currently held in memory
    pub fn count(&self) -> usize {
        self.stocks.len()
    }

    /// Retrieve all tickers currently stored
    pub fn get_all_tickers(&self) -> Vec<String> {
        self.stocks.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Filter stocks by sector
    pub fn filter_by_sector(&self, sector: &str) -> Vec<StockProfile> {
        self.stocks
            .iter()
            .filter(|entry| entry.value().sector.eq_ignore_ascii_case(sector))
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Filter stocks by minimum market cap
    pub fn filter_by_min_mcap(&self, min_mcap_cr: f64) -> Vec<StockProfile> {
        self.stocks
            .iter()
            .filter(|entry| entry.value().market_cap_cr >= min_mcap_cr)
            .map(|entry| entry.value().clone())
            .collect()
    }
}

/// High-Precision Deterministic Financial Ratio Engine
pub struct FinancialRatioEngine;

impl FinancialRatioEngine {
    /// Computes full financial ratios and assigns an objective fundamental health score
    pub fn compute_ratios(profile: &StockProfile) -> Option<FinancialRatios> {
        let latest_inc = profile.income_statements.last()?;
        let latest_bs = profile.balance_sheets.last()?;
        let latest_cf = profile.cash_flows.last();

        // 1. EBIT = EBITDA - Depreciation
        let ebit = latest_inc.ebitda - latest_inc.depreciation;

        // 2. Capital Employed = Total Equity + Total Debt
        let shareholders_equity = latest_bs.equity_share_capital + latest_bs.reserves_and_surplus;
        let capital_employed = shareholders_equity + latest_bs.total_debt;
        let roce_percent = if capital_employed > 0.0 {
            (ebit / capital_employed) * 100.0
        } else {
            0.0
        };

        // 3. Debt-to-Equity
        let debt_to_equity = if shareholders_equity > 0.0 {
            latest_bs.total_debt / shareholders_equity
        } else {
            999.0
        };

        // 4. OPM Expansion Delta = Latest OPM% - Historical Median OPM%
        let mut historical_opms: Vec<f64> = profile
            .income_statements
            .iter()
            .map(|i| i.opm_percent)
            .collect();
        historical_opms.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median_opm = if !historical_opms.is_empty() {
            let mid = historical_opms.len() / 2;
            if historical_opms.len() % 2 == 0 && mid > 0 {
                (historical_opms[mid - 1] + historical_opms[mid]) / 2.0
            } else {
                historical_opms[mid]
            }
        } else {
            latest_inc.opm_percent
        };
        let opm_expansion_delta = latest_inc.opm_percent - median_opm;

        // 5. PE Mean Reversion Z-Score
        let pe_std = if profile.pe_5yr_std > 0.001 {
            profile.pe_5yr_std
        } else {
            1.0
        };
        let pe_mean_reversion_zscore = (profile.current_pe - profile.pe_5yr_median) / pe_std;

        // 6. CFO to Net Profit Ratio
        let cfo_to_net_profit = if let Some(cf) = latest_cf {
            if latest_inc.net_profit.abs() > 0.001 {
                cf.cfo / latest_inc.net_profit
            } else {
                1.0
            }
        } else {
            1.0 // Neutral fallback if CF statement not yet published
        };

        // 7. Current Ratio
        let current_ratio = if latest_bs.current_liabilities > 0.0 {
            latest_bs.current_assets / latest_bs.current_liabilities
        } else {
            1.5
        };

        // 8. Interest Coverage
        let interest_coverage = if latest_inc.interest_expense > 0.001 {
            ebit / latest_inc.interest_expense
        } else {
            99.0 // Virtually debt-free or zero interest
        };

        // 9. Fundamental Health Score (1.0 to 10.0)
        let health_score = Self::evaluate_health_score(
            roce_percent,
            debt_to_equity,
            opm_expansion_delta,
            pe_mean_reversion_zscore,
            cfo_to_net_profit,
            interest_coverage,
        );

        Some(FinancialRatios {
            roce_percent,
            debt_to_equity,
            opm_expansion_delta,
            pe_mean_reversion_zscore,
            cfo_to_net_profit,
            current_ratio,
            interest_coverage,
            fundamental_health_score: health_score,
        })
    }

    /// Evaluates fundamental health score (1.0 to 10.0 scale) based on deterministic weights
    fn evaluate_health_score(
        roce: f64,
        de: f64,
        opm_delta: f64,
        pe_z: f64,
        cfo_ratio: f64,
        int_cov: f64,
    ) -> f32 {
        let mut score = 5.0f32; // Baseline neutral score

        // ROCE scoring (> 20% is stellar, < 10% is poor)
        if roce >= 25.0 {
            score += 1.5;
        } else if roce >= 18.0 {
            score += 1.0;
        } else if roce < 10.0 {
            score -= 1.0;
        }

        // Debt-to-Equity (< 0.5 is safe, > 1.5 is risky)
        if de <= 0.1 {
            score += 1.0; // Virtually debt free
        } else if de <= 0.5 {
            score += 0.5;
        } else if de > 1.5 {
            score -= 1.5;
        }

        // OPM Expansion Delta
        if opm_delta > 2.0 {
            score += 0.8;
        } else if opm_delta < -2.0 {
            score -= 0.8;
        }

        // PE Mean Reversion (Negative z-score implies undervalued vs historical mean)
        if pe_z < -1.0 {
            score += 0.7; // Discounted valuation
        } else if pe_z > 2.0 {
            score -= 0.7; // Expensive stretched valuation
        }

        // CFO to Net Profit (Quality of Earnings)
        if cfo_ratio >= 1.0 {
            score += 0.5;
        } else if cfo_ratio < 0.5 {
            score -= 1.0; // Poor cash realization / high accruals
        }

        // Interest Coverage
        if int_cov > 8.0 {
            score += 0.5;
        } else if int_cov < 2.0 {
            score -= 1.0;
        }

        score.clamp(1.0, 10.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stock_database_in_memory() {
        let db = StockDatabase::new();
        let profile = StockProfile {
            ticker: "TCS".to_string(),
            exchange: "NSE".to_string(),
            company_name: "Tata Consultancy Services Ltd".to_string(),
            sector: "Information Technology".to_string(),
            current_price: 3850.0,
            market_cap_cr: 1400000.0,
            current_pe: 28.5,
            pe_5yr_median: 30.0,
            pe_5yr_std: 3.5,
            income_statements: vec![IncomeStatement {
                period_ending: "2024-03-31".to_string(),
                sales: 240893.0,
                ebitda: 64000.0,
                opm_percent: 26.5,
                depreciation: 4000.0,
                interest_expense: 500.0,
                profit_before_tax: 59500.0,
                tax_rate_percent: 25.0,
                net_profit: 44625.0,
                eps: 125.0,
            }],
            balance_sheets: vec![BalanceSheet {
                period_ending: "2024-03-31".to_string(),
                equity_share_capital: 366.0,
                reserves_and_surplus: 90000.0,
                total_debt: 0.0,
                fixed_assets: 25000.0,
                working_capital: 45000.0,
                current_assets: 80000.0,
                current_liabilities: 35000.0,
                cash_and_equivalents: 20000.0,
            }],
            cash_flows: vec![CashFlowStatement {
                period_ending: "2024-03-31".to_string(),
                cfo: 45000.0,
                cfi: -8000.0,
                cff: -35000.0,
                free_cash_flow: 38000.0,
            }],
            latest_ratios: None,
        };

        db.upsert_stock(profile);
        assert_eq!(db.count(), 1);

        let retrieved = db.get_stock("TCS").expect("TCS should be found");
        let ratios = FinancialRatioEngine::compute_ratios(&retrieved).expect("Ratios calculated");
        assert!(ratios.roce_percent > 50.0);
        assert_eq!(ratios.debt_to_equity, 0.0);
        assert!(ratios.fundamental_health_score >= 8.0);
    }
}
