//! # Module 3: TAM Velocity & Market Share Growth Engine
//!
//! Mathematical deterministic simulation model for Total Addressable Market (TAM)
//! expansion, market share velocity, and multi-scenario (Conservative, Base, Best)
//! 3-to-5 Year Earnings Trajectories.

use serde::{Deserialize, Serialize};

/// Yearly Projected Output Metrics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct YearProjection {
    pub year: u32,
    pub projected_tam_cr: f64,
    pub projected_market_share_pct: f64,
    pub projected_sales_cr: f64,
    pub projected_ebitda_cr: f64,
    pub projected_pat_cr: f64,
    pub projected_eps: f64,
}

/// Scenario-based Trajectory (Conservative / Base / Best)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScenarioTrajectory {
    pub scenario_name: String,
    pub years: Vec<YearProjection>,
    pub cagr_sales_pct: f64,
    pub cagr_ebitda_pct: f64,
    pub cagr_pat_pct: f64,
}

/// Complete Set of Simulated Scenarios
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TamScenarioSet {
    pub industry_tam_now_cr: f64,
    pub current_market_share_pct: f64,
    pub conservative: ScenarioTrajectory,
    pub base: ScenarioTrajectory,
    pub best: ScenarioTrajectory,
    pub tam_growth_score: f32, // 1.0 to 10.0 scale
}

/// Deterministic TAM Simulation Engine
pub struct TamSimulationEngine;

impl TamSimulationEngine {
    /// Simulates 3-to-5 Year Trajectories across Conservative, Base, and Best cases.
    ///
    /// Formula:
    /// `Projected_Sales_t = Industry_TAM_t * (Projected_Market_Share_pct_t / 100.0)`
    pub fn simulate(
        base_sales_cr: f64,
        base_ebitda_margin_pct: f64,
        base_pat_margin_pct: f64,
        shares_outstanding_cr: f64,
        industry_tam_cr: f64,
        current_market_share_pct: f64,
        industry_cagr_pct: f64,
    ) -> TamScenarioSet {
        let shares = if shares_outstanding_cr > 0.001 {
            shares_outstanding_cr
        } else {
            1.0
        };

        // 1. Conservative Case: Industry grows slower, market share remains stagnant or slightly erodes
        let conservative = Self::compute_trajectory(
            "Conservative (Bear)",
            base_sales_cr,
            base_ebitda_margin_pct * 0.90, // Margin compression
            base_pat_margin_pct * 0.85,
            shares,
            industry_tam_cr,
            current_market_share_pct * 0.98,
            (industry_cagr_pct * 0.60).max(4.0),
            0.0, // 0 bps annual market share delta
            5,
        );

        // 2. Base Case: Industry grows at median CAGR, moderate organic share gain
        let base = Self::compute_trajectory(
            "Base (Expected)",
            base_sales_cr,
            base_ebitda_margin_pct,
            base_pat_margin_pct,
            shares,
            industry_tam_cr,
            current_market_share_pct,
            industry_cagr_pct,
            0.5, // 50 bps annual market share gain
            5,
        );

        // 3. Best Case: Industry super-cycle (PLI/Capex/Tailwind), aggressive share gain
        let best = Self::compute_trajectory(
            "Best (Super-Cycle)",
            base_sales_cr,
            base_ebitda_margin_pct * 1.15, // Operating leverage expansion
            base_pat_margin_pct * 1.20,
            shares,
            industry_tam_cr,
            current_market_share_pct,
            industry_cagr_pct * 1.40,
            1.5, // 150 bps annual market share gain
            5,
        );

        // Calculate TAM Growth Score (1.0 to 10.0) based on base sales CAGR and headroom
        let base_cagr = base.cagr_sales_pct;
        let tam_headroom = if current_market_share_pct > 0.0 {
            100.0 / current_market_share_pct
        } else {
            10.0
        };

        let mut score = 5.0f32;
        if base_cagr >= 25.0 {
            score += 2.5;
        } else if base_cagr >= 18.0 {
            score += 1.8;
        } else if base_cagr >= 12.0 {
            score += 1.0;
        } else if base_cagr < 6.0 {
            score -= 1.5;
        }

        // Headroom multiplier (High headroom to grow inside large TAM)
        if tam_headroom >= 10.0 {
            score += 1.5; // Has <10% market share with massive runway
        } else if tam_headroom >= 4.0 {
            score += 0.8;
        } else if tam_headroom < 1.5 {
            score -= 1.0; // Saturated market (>66% market share)
        }

        let tam_growth_score = score.clamp(1.0, 10.0);

        TamScenarioSet {
            industry_tam_now_cr: industry_tam_cr,
            current_market_share_pct,
            conservative,
            base,
            best,
            tam_growth_score,
        }
    }

    fn compute_trajectory(
        name: &str,
        base_sales: f64,
        ebitda_margin: f64,
        pat_margin: f64,
        shares_cr: f64,
        tam_cr: f64,
        start_market_share: f64,
        tam_growth_rate_pct: f64,
        annual_share_gain_pct: f64,
        years: usize,
    ) -> ScenarioTrajectory {
        let mut projections = Vec::with_capacity(years);
        let mut cur_tam = tam_cr;
        let mut cur_share = start_market_share;

        for y in 1..=(years as u32) {
            cur_tam *= 1.0 + (tam_growth_rate_pct / 100.0);
            cur_share = (cur_share + annual_share_gain_pct).min(90.0); // Capped at realistic 90%

            let projected_sales = cur_tam * (cur_share / 100.0);
            let projected_ebitda = projected_sales * (ebitda_margin / 100.0);
            let projected_pat = projected_sales * (pat_margin / 100.0);
            let projected_eps = projected_pat / shares_cr;

            projections.push(YearProjection {
                year: y,
                projected_tam_cr: cur_tam,
                projected_market_share_pct: cur_share,
                projected_sales_cr: projected_sales,
                projected_ebitda_cr: projected_ebitda,
                projected_pat_cr: projected_pat,
                projected_eps,
            });
        }

        let final_sales = projections.last().map(|p| p.projected_sales_cr).unwrap_or(base_sales);
        let final_ebitda = projections.last().map(|p| p.projected_ebitda_cr).unwrap_or(base_sales * 0.15);
        let final_pat = projections.last().map(|p| p.projected_pat_cr).unwrap_or(base_sales * 0.08);

        let initial_ebitda = (base_sales * (ebitda_margin / 100.0)).max(0.01);
        let initial_pat = (base_sales * (pat_margin / 100.0)).max(0.01);

        let cagr_sales_pct = Self::compute_cagr(base_sales, final_sales, years);
        let cagr_ebitda_pct = Self::compute_cagr(initial_ebitda, final_ebitda, years);
        let cagr_pat_pct = Self::compute_cagr(initial_pat, final_pat, years);

        ScenarioTrajectory {
            scenario_name: name.to_string(),
            years: projections,
            cagr_sales_pct,
            cagr_ebitda_pct,
            cagr_pat_pct,
        }
    }

    fn compute_cagr(start_val: f64, end_val: f64, years: usize) -> f64 {
        if start_val <= 0.0 || end_val <= 0.0 || years == 0 {
            return 0.0;
        }
        let ratio = end_val / start_val;
        ((ratio.powf(1.0 / years as f64)) - 1.0) * 100.0
    }

    /// Returns realistic (Industry TAM in Crores, Industry Historic/Projected CAGR %) for Indian equity sectors.
    pub fn get_sector_tam_and_cagr(sector: &str) -> (f64, f64) {
        let s = sector.to_lowercase();
        if s.contains("it") || s.contains("information") || s.contains("software") || s.contains("tech") {
            (2_100_000.0, 9.5) // Indian IT & Global Tech Services
        } else if s.contains("bank") || s.contains("financial") || s.contains("nbfc") || s.contains("insurance") {
            (18_000_000.0, 13.0) // Indian Banking, Credit & Financial Services
        } else if s.contains("auto") || s.contains("motor") || s.contains("ev") || s.contains("vehicle") {
            (1_200_000.0, 11.5) // Indian Automotive & Mobility
        } else if s.contains("pharma") || s.contains("health") || s.contains("drug") || s.contains("bio") {
            (450_000.0, 10.5) // Indian Pharmaceuticals & Healthcare
        } else if s.contains("fmcg") || s.contains("consumer") || s.contains("food") || s.contains("beverage") {
            (1_100_000.0, 9.0) // Indian FMCG & Retail
        } else if s.contains("infra") || s.contains("capital") || s.contains("power") || s.contains("energy") || s.contains("engineer") {
            (1_500_000.0, 15.0) // Indian Infrastructure, Energy & Capex
        } else if s.contains("chem") || s.contains("specialty") || s.contains("paint") {
            (350_000.0, 12.0) // Indian Specialty Chemicals & Materials
        } else if s.contains("metal") || s.contains("steel") || s.contains("mining") {
            (800_000.0, 7.5) // Indian Metals, Steel & Mining
        } else if s.contains("telecom") || s.contains("digital") || s.contains("media") {
            (400_000.0, 12.5) // Indian Telecom & Digital Ecosystem
        } else {
            (2_500_000.0, 10.0) // Multi-sector Conglomerates / General Indian Industry
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tam_simulation_engine() {
        let scenario_set = TamSimulationEngine::simulate(
            1000.0,  // Base Sales ₹1,000 Cr
            20.0,    // 20% EBITDA margin
            12.0,    // 12% PAT margin
            10.0,    // 10 Cr shares
            50000.0, // ₹50,000 Cr TAM
            2.0,     // 2% current market share (huge headroom)
            15.0,    // 15% Industry CAGR
        );

        assert_eq!(scenario_set.base.years.len(), 5);
        assert!(scenario_set.base.cagr_sales_pct > 15.0);
        assert!(scenario_set.tam_growth_score >= 8.0);
    }
}
