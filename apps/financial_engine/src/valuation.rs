//! # Phase 4: Forward S-Curve Multiple Expansion & Valuation Re-Rating Engine (`valuation.rs`)
//!
//! Deterministic forward valuation modeling that replaces backward-looking trailing P/E.
//! Models multi-scenario EPS projections (Bear, Base, Super-Cycle Bull) and quantifies
//! "Institutional Blind Spot" disparity when rapid earnings growth coexists with depressed valuations.

use serde::{Deserialize, Serialize};

/// 3-Year Forward Valuation Projections under Variant Perception Scenarios
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ForwardValuationProjection {
    pub ticker: String,
    pub current_price: f64,
    pub current_trailing_pe: f64,
    pub trailing_eps: f64,
    pub bear_case: ValuationScenarioOutput,
    pub base_case: ValuationScenarioOutput,
    pub bull_supercycle: ValuationScenarioOutput,
    pub institutional_blind_spot_score: f64,
    pub is_institutional_blind_spot: bool,
    pub statutory_disclaimer: String,
}

/// Scenario Outcome Metrics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValuationScenarioOutput {
    pub scenario_name: String,
    pub forward_3yr_eps: f64,
    pub projected_exit_pe: f64,
    pub target_price: f64,
    pub upside_potential_pct: f64,
    pub cagr_pct: f64,
}

/// Parameters for Valuation Modeling
#[derive(Debug, Clone)]
pub struct ValuationInput {
    pub current_price: f64,
    pub trailing_eps: f64,
    pub trailing_pe: f64,
    pub roce_pct: f64,
    pub expected_3yr_revenue_cagr_pct: f64,
    pub current_market_share_pct: f64,
    pub target_market_share_pct: f64,
    pub promoter_pledge_pct: f64,
    pub has_tech_or_policy_moat: bool,
}

/// Pure Rust Valuation & Re-Rating Engine
pub struct ValuationReRatingEngine;

impl ValuationReRatingEngine {
    /// Computes full forward multi-scenario projections and institutional blind-spot index
    pub fn compute_projections(ticker: &str, input: ValuationInput) -> ForwardValuationProjection {
        let curr_p = input.current_price.max(1.0);
        let t_eps = input.trailing_eps;

        // MM-03 Loss-Making Isolation Guard:
        // Negative or zero trailing EPS indicates loss-making operations.
        // S-Curve PE multiple expansion is invalid for loss-making companies.
        if t_eps <= 0.0 {
            let bear_target = (curr_p * 0.5 * 100.0).round() / 100.0;
            let bear_case = ValuationScenarioOutput {
                scenario_name: "Bear Case (Continued Cash Burn / Share Dilution)".to_string(),
                forward_3yr_eps: (t_eps * 1.5 * 100.0).round() / 100.0,
                projected_exit_pe: 0.0,
                target_price: bear_target,
                upside_potential_pct: -50.0,
                cagr_pct: Self::compute_3yr_cagr(curr_p, bear_target),
            };

            let base_case = ValuationScenarioOutput {
                scenario_name: "Base Case (Break-Even Trajectory / Operational Restructuring)".to_string(),
                forward_3yr_eps: 0.0,
                projected_exit_pe: 0.0,
                target_price: curr_p,
                upside_potential_pct: 0.0,
                cagr_pct: 0.0,
            };

            let bull_target = (curr_p * 1.35 * 100.0).round() / 100.0;
            let bull_supercycle = ValuationScenarioOutput {
                scenario_name: "Turnaround Speculative Case (Turnaround to Modest Profitability)".to_string(),
                forward_3yr_eps: 1.0,
                projected_exit_pe: 15.0,
                target_price: bull_target,
                upside_potential_pct: 35.0,
                cagr_pct: Self::compute_3yr_cagr(curr_p, bull_target),
            };

            return ForwardValuationProjection {
                ticker: ticker.to_string(),
                current_price: curr_p,
                current_trailing_pe: 0.0,
                trailing_eps: t_eps,
                bear_case,
                base_case,
                bull_supercycle,
                institutional_blind_spot_score: 0.0,
                is_institutional_blind_spot: false,
                statutory_disclaimer: format!(
                    "RESEARCH & QUANTITATIVE SCENARIO SIMULATION ONLY: [ALERT: {} is currently loss-making with trailing EPS of {:.2}. Trailing P/E is N/A.]",
                    ticker, t_eps
                ),
            };
        }

        let curr_pe = (curr_p / t_eps).max(1.0);

        // 1. Bear Case: S-Curve stall, margins contract, PE compresses by 20%
        let bear_cagr = (input.expected_3yr_revenue_cagr_pct * 0.4).max(0.0);
        let bear_eps_growth = (1.0 + bear_cagr / 100.0).powi(3);
        let bear_3yr_eps = (t_eps * bear_eps_growth * 0.9 * 100.0).round() / 100.0;
        let bear_exit_pe = (curr_pe * 0.8).max(8.0);
        let bear_target = (bear_3yr_eps * bear_exit_pe * 100.0).round() / 100.0;
        let bear_upside = (((bear_target - curr_p) / curr_p) * 100.0 * 10.0).round() / 10.0;
        let bear_cagr_ret = Self::compute_3yr_cagr(curr_p, bear_target);

        let bear_case = ValuationScenarioOutput {
            scenario_name: "Bear Case (Industry Deceleration / Margin Compression)".to_string(),
            forward_3yr_eps: bear_3yr_eps,
            projected_exit_pe: (bear_exit_pe * 10.0).round() / 10.0,
            target_price: bear_target,
            upside_potential_pct: bear_upside,
            cagr_pct: bear_cagr_ret,
        };

        // 2. Base Case: Steady TAM growth, mild market share accretion, PE stays flat
        let base_cagr = input.expected_3yr_revenue_cagr_pct;
        let base_eps_growth = (1.0 + base_cagr / 100.0).powi(3);
        let base_3yr_eps = (t_eps * base_eps_growth * 100.0).round() / 100.0;
        let base_exit_pe = curr_pe;
        let base_target = (base_3yr_eps * base_exit_pe * 100.0).round() / 100.0;
        let base_upside = (((base_target - curr_p) / curr_p) * 100.0 * 10.0).round() / 10.0;
        let base_cagr_ret = Self::compute_3yr_cagr(curr_p, base_target);

        let base_case = ValuationScenarioOutput {
            scenario_name: "Base Case (Organic Industry Growth + Steady Margins)".to_string(),
            forward_3yr_eps: base_3yr_eps,
            projected_exit_pe: (base_exit_pe * 10.0).round() / 10.0,
            target_price: base_target,
            upside_potential_pct: base_upside,
            cagr_pct: base_cagr_ret,
        };

        // 3. Super-Cycle Bull Case: S-Curve Inflection, Market Share doubles, Operating Leverage + PE Re-Rating
        // High operating leverage expands EPS faster than revenue (1.35x multiplier)
        let share_gain_bonus = if input.target_market_share_pct > input.current_market_share_pct {
            (input.target_market_share_pct - input.current_market_share_pct) * 2.0
        } else {
            0.0
        };

        let bull_cagr = input.expected_3yr_revenue_cagr_pct * 1.3 + share_gain_bonus;
        let bull_eps_growth = (1.0 + (bull_cagr * 1.35) / 100.0).powi(3);
        let bull_3yr_eps = (t_eps * bull_eps_growth * 100.0).round() / 100.0;

        // Multiple expansion: Re-rates from modest PE (e.g. 15x-20x) up towards institutional quality PE (30x-45x)
        let fair_rerated_pe = if input.roce_pct >= 25.0 && input.promoter_pledge_pct == 0.0 {
            (curr_pe * 1.6).clamp(25.0, 45.0)
        } else if input.roce_pct >= 18.0 {
            (curr_pe * 1.35).clamp(20.0, 35.0)
        } else {
            (curr_pe * 1.15).clamp(15.0, 28.0)
        };

        let bull_target = (bull_3yr_eps * fair_rerated_pe * 100.0).round() / 100.0;
        let bull_upside = (((bull_target - curr_p) / curr_p) * 100.0 * 10.0).round() / 10.0;
        let bull_cagr_ret = Self::compute_3yr_cagr(curr_p, bull_target);

        let bull_supercycle = ValuationScenarioOutput {
            scenario_name: "Super-Cycle Bull Case (Market Share Accretion + Institutional Multiple Expansion)".to_string(),
            forward_3yr_eps: bull_3yr_eps,
            projected_exit_pe: (fair_rerated_pe * 10.0).round() / 10.0,
            target_price: bull_target,
            upside_potential_pct: bull_upside,
            cagr_pct: bull_cagr_ret,
        };

        // 4. Institutional Blind Spot Score (0.0 to 100.0)
        // High ROCE (>20%) + High Growth (>25%) + Depressed Low PE (< 22x) + Zero Pledge
        let mut blind_spot_score = 0.0;

        if input.roce_pct >= 20.0 {
            blind_spot_score += 30.0;
        } else if input.roce_pct >= 15.0 {
            blind_spot_score += 15.0;
        }

        if input.expected_3yr_revenue_cagr_pct >= 25.0 {
            blind_spot_score += 25.0;
        } else if input.expected_3yr_revenue_cagr_pct >= 15.0 {
            blind_spot_score += 15.0;
        }

        // Low PE disparity relative to growth
        if curr_pe <= 18.0 {
            blind_spot_score += 25.0;
        } else if curr_pe <= 25.0 {
            blind_spot_score += 15.0;
        }

        // Clean governance bonus
        if input.promoter_pledge_pct == 0.0 {
            blind_spot_score += 10.0;
        }

        // Moat bonus
        if input.has_tech_or_policy_moat {
            blind_spot_score += 10.0;
        }

        let is_blind_spot = blind_spot_score >= 70.0;

        ForwardValuationProjection {
            ticker: ticker.to_string(),
            current_price: curr_p,
            current_trailing_pe: (curr_pe * 10.0).round() / 10.0,
            trailing_eps: t_eps,
            bear_case,
            base_case,
            bull_supercycle,
            institutional_blind_spot_score: blind_spot_score,
            is_institutional_blind_spot: is_blind_spot,
            statutory_disclaimer: "RESEARCH & QUANTITATIVE SCENARIO SIMULATION ONLY: Under SEBI (Research Analysts) Regulations, 2014, these mathematical scenario outputs are educational simulations based on historic financial statements and probabilistic multiple expansion. They do not constitute formal investment advice or a solicitation to buy or sell securities.".to_string(),
        }
    }

    fn compute_3yr_cagr(start_price: f64, end_price: f64) -> f64 {
        if start_price <= 0.0 || end_price <= 0.0 {
            return 0.0;
        }
        let ratio = end_price / start_price;
        let cagr = (ratio.powf(1.0 / 3.0) - 1.0) * 100.0;
        (cagr * 10.0).round() / 10.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_institutional_blind_spot_detection() {
        // High ROCE 28%, Rapid growth 30%, Undervalued PE 15x, Zero pledge, Moat present
        let input = ValuationInput {
            current_price: 150.0,
            trailing_eps: 10.0,
            trailing_pe: 15.0,
            roce_pct: 28.0,
            expected_3yr_revenue_cagr_pct: 30.0,
            current_market_share_pct: 3.0,
            target_market_share_pct: 6.0,
            promoter_pledge_pct: 0.0,
            has_tech_or_policy_moat: true,
        };

        let proj = ValuationReRatingEngine::compute_projections("EMS_ALPHA", input);
        assert!(proj.is_institutional_blind_spot);
        assert!(proj.institutional_blind_spot_score >= 80.0);
        // In bull case, target price should represent massive upside
        assert!(proj.bull_supercycle.upside_potential_pct > 150.0);
        assert!(proj.bull_supercycle.projected_exit_pe >= 25.0);
    }

    #[test]
    fn test_bear_case_preserves_capital_realism() {
        let input = ValuationInput {
            current_price: 500.0,
            trailing_eps: 10.0,
            trailing_pe: 50.0,
            roce_pct: 12.0,
            expected_3yr_revenue_cagr_pct: 8.0,
            current_market_share_pct: 5.0,
            target_market_share_pct: 5.0,
            promoter_pledge_pct: 15.0,
            has_tech_or_policy_moat: false,
        };

        let proj = ValuationReRatingEngine::compute_projections("EXPENSIVE_CORP", input);
        assert!(!proj.is_institutional_blind_spot);
        assert!(proj.bear_case.upside_potential_pct < 0.0); // Predicts multiple contraction drawdown
    }
}
