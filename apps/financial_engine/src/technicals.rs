//! # Module 2: Technical Indicators & Candlestick Pattern Recognition Engine
//!
//! Tokio-friendly mathematical technical analysis: RSI-14, MACD (12,26,9),
//! Bollinger Bands Squeeze/Expansion, EMAs (20, 50, 200), and deterministic
//! Candlestick Pattern Detection (Doji, Hammer, Engulfing, Morning Star).

use serde::{Deserialize, Serialize};

/// Standard OHLCV Price Candle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OhlcvCandle {
    pub timestamp_epoch_sec: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Bollinger Bands Output
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BollingerBands {
    pub upper_band: f64,
    pub middle_band: f64,
    pub lower_band: f64,
    pub bandwidth_percent: f64, // (Upper - Lower) / Middle * 100
    pub is_squeeze: bool,        // Bandwidth in lowest percentile
    pub is_expansion: bool,      // Price expanding beyond band
}

/// MACD Output
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacdOutput {
    pub macd_line: f64,
    pub signal_line: f64,
    pub histogram: f64,
    pub is_bullish_crossover: bool,
    pub is_bearish_crossover: bool,
}

/// Recognized Candlestick Pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternType {
    Doji,
    Hammer,
    InvertedHammer,
    BullishEngulfing,
    BearishEngulfing,
    MorningStar,
    EveningStar,
}

/// Pattern Match Detection Result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandlePatternMatch {
    pub pattern: PatternType,
    pub confidence: f32, // 0.0 to 1.0
    pub description: String,
}

/// Comprehensive Technical Summary
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TechnicalSummary {
    pub rsi_14: f64,
    pub macd: MacdOutput,
    pub bollinger: BollingerBands,
    pub ema_20: f64,
    pub ema_50: f64,
    pub ema_200: f64,
    pub is_golden_cross: bool, // EMA 50 > EMA 200
    pub detected_patterns: Vec<CandlePatternMatch>,
    pub technical_score: f32,  // 1.0 to 10.0 scale
}

pub struct TechnicalEngine;

impl TechnicalEngine {
    /// Computes Exponential Moving Average (EMA) for a series
    pub fn compute_ema(prices: &[f64], period: usize) -> Vec<f64> {
        if prices.len() < period || period == 0 {
            return Vec::new();
        }

        let mut ema_values = Vec::with_capacity(prices.len());
        let multiplier = 2.0 / (period as f64 + 1.0);

        // Seed with SMA
        let sma: f64 = prices[..period].iter().sum::<f64>() / (period as f64);
        ema_values.push(sma);

        let mut current_ema = sma;
        for &price in &prices[period..] {
            current_ema = (price - current_ema) * multiplier + current_ema;
            ema_values.push(current_ema);
        }

        ema_values
    }

    /// Computes Wilder's RSI (14-period standard)
    pub fn compute_rsi(prices: &[f64], period: usize) -> f64 {
        if prices.len() <= period {
            return 50.0;
        }

        let mut gains = Vec::with_capacity(prices.len() - 1);
        let mut losses = Vec::with_capacity(prices.len() - 1);

        for window in prices.windows(2) {
            let change = window[1] - window[0];
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(change.abs());
            }
        }

        if gains.len() < period {
            return 50.0;
        }

        let mut avg_gain = gains[..period].iter().sum::<f64>() / period as f64;
        let mut avg_loss = losses[..period].iter().sum::<f64>() / period as f64;

        for i in period..gains.len() {
            avg_gain = (avg_gain * (period as f64 - 1.0) + gains[i]) / period as f64;
            avg_loss = (avg_loss * (period as f64 - 1.0) + losses[i]) / period as f64;
        }

        if avg_loss == 0.0 {
            return 100.0;
        }

        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    }

    /// Computes MACD (12, 26, 9)
    pub fn compute_macd(prices: &[f64]) -> MacdOutput {
        if prices.len() < 35 {
            return MacdOutput {
                macd_line: 0.0,
                signal_line: 0.0,
                histogram: 0.0,
                is_bullish_crossover: false,
                is_bearish_crossover: false,
            };
        }

        let ema_12 = Self::compute_ema(prices, 12);
        let ema_26 = Self::compute_ema(prices, 26);

        // Align series
        let offset = 26 - 12;
        let mut macd_series = Vec::new();
        for i in 0..ema_26.len() {
            macd_series.push(ema_12[i + offset] - ema_26[i]);
        }

        let signal_series = Self::compute_ema(&macd_series, 9);
        let latest_macd = *macd_series.last().unwrap_or(&0.0);
        let latest_signal = *signal_series.last().unwrap_or(&0.0);
        let histogram = latest_macd - latest_signal;

        let prev_macd = if macd_series.len() >= 2 {
            macd_series[macd_series.len() - 2]
        } else {
            0.0
        };
        let prev_signal = if signal_series.len() >= 2 {
            signal_series[signal_series.len() - 2]
        } else {
            0.0
        };

        let is_bullish_crossover = prev_macd <= prev_signal && latest_macd > latest_signal;
        let is_bearish_crossover = prev_macd >= prev_signal && latest_macd < latest_signal;

        MacdOutput {
            macd_line: latest_macd,
            signal_line: latest_signal,
            histogram,
            is_bullish_crossover,
            is_bearish_crossover,
        }
    }

    /// Computes Bollinger Bands (20-period SMA, 2 standard deviations)
    pub fn compute_bollinger_bands(prices: &[f64], period: usize, num_std: f64) -> BollingerBands {
        if prices.len() < period {
            let last_p = *prices.last().unwrap_or(&100.0);
            return BollingerBands {
                upper_band: last_p * 1.05,
                middle_band: last_p,
                lower_band: last_p * 0.95,
                bandwidth_percent: 10.0,
                is_squeeze: false,
                is_expansion: false,
            };
        }

        let window = &prices[prices.len() - period..];
        let sma = window.iter().sum::<f64>() / period as f64;
        let variance = window.iter().map(|&p| (p - sma).powi(2)).sum::<f64>() / period as f64;
        let std_dev = variance.sqrt();

        let upper_band = sma + (std_dev * num_std);
        let lower_band = sma - (std_dev * num_std);
        let bandwidth_percent = if sma > 0.0 {
            (upper_band - lower_band) / sma * 100.0
        } else {
            0.0
        };

        let latest_price = *prices.last().unwrap_or(&sma);
        let is_squeeze = bandwidth_percent < 5.0; // Volatility compression
        let is_expansion = latest_price > upper_band || latest_price < lower_band;

        BollingerBands {
            upper_band,
            middle_band: sma,
            lower_band,
            bandwidth_percent,
            is_squeeze,
            is_expansion,
        }
    }

    /// Detects Candlestick Patterns on recent OHLC candles
    pub fn detect_candlestick_patterns(candles: &[OhlcvCandle]) -> Vec<CandlePatternMatch> {
        let mut matches = Vec::new();
        if candles.is_empty() {
            return matches;
        }

        let n = candles.len();
        let last = &candles[n - 1];
        let range = (last.high - last.low).max(0.001);
        let body = (last.close - last.open).abs();
        let upper_wick = last.high - last.open.max(last.close);
        let lower_wick = last.open.min(last.close) - last.low;

        // 1. Doji Detection: Body <= 10% of total candle range
        if body / range <= 0.10 {
            matches.push(CandlePatternMatch {
                pattern: PatternType::Doji,
                confidence: 0.85,
                description: "Indecision Doji candle detected near market equilibrium.".to_string(),
            });
        }

        // 2. Hammer Detection: Lower wick >= 2x body, tiny upper wick, bullish context
        if lower_wick >= 2.0 * body && upper_wick <= 0.2 * body && last.close >= last.open {
            matches.push(CandlePatternMatch {
                pattern: PatternType::Hammer,
                confidence: 0.88,
                description: "Bullish Hammer detected with strong price rejection at lows.".to_string(),
            });
        }

        // 3. Engulfing Patterns (Requires 2 candles)
        if n >= 2 {
            let prev = &candles[n - 2];
            let prev_body = (prev.close - prev.open).abs();

            // Bullish Engulfing: prev is red, current is green and engulfs prev body
            if prev.close < prev.open
                && last.close > last.open
                && last.open <= prev.close
                && last.close >= prev.open
                && body > prev_body
            {
                matches.push(CandlePatternMatch {
                    pattern: PatternType::BullishEngulfing,
                    confidence: 0.92,
                    description: "High-conviction Bullish Engulfing reversal pattern.".to_string(),
                });
            }

            // Bearish Engulfing: prev is green, current is red and engulfs prev body
            if prev.close > prev.open
                && last.close < last.open
                && last.open >= prev.close
                && last.close <= prev.open
                && body > prev_body
            {
                matches.push(CandlePatternMatch {
                    pattern: PatternType::BearishEngulfing,
                    confidence: 0.90,
                    description: "Bearish Engulfing pattern indicating potential top reversal.".to_string(),
                });
            }
        }

        // 4. Morning Star Reversal (Requires 3 candles)
        if n >= 3 {
            let first = &candles[n - 3];
            let second = &candles[n - 2];
            let third = &candles[n - 1];

            let first_bearish = first.close < first.open;
            let second_small = (second.close - second.open).abs() / (second.high - second.low).max(0.001) < 0.35;
            let third_bullish = third.close > third.open;
            let first_midpoint = (first.open + first.close) / 2.0;

            if first_bearish && second_small && third_bullish && third.close > first_midpoint {
                matches.push(CandlePatternMatch {
                    pattern: PatternType::MorningStar,
                    confidence: 0.94,
                    description: "Elite 3-bar Morning Star bullish bottom reversal.".to_string(),
                });
            }
        }

        matches
    }

    /// Evaluates full technical suite and assigns an objective technical score (1.0 to 10.0)
    pub fn evaluate_technicals(candles: &[OhlcvCandle]) -> TechnicalSummary {
        let prices: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let rsi_14 = Self::compute_rsi(&prices, 14);
        let macd = Self::compute_macd(&prices);
        let bollinger = Self::compute_bollinger_bands(&prices, 20, 2.0);

        let ema_20_vec = Self::compute_ema(&prices, 20);
        let ema_50_vec = Self::compute_ema(&prices, 50);
        let ema_200_vec = Self::compute_ema(&prices, 200);

        let ema_20 = *ema_20_vec.last().unwrap_or(&0.0);
        let ema_50 = *ema_50_vec.last().unwrap_or(&0.0);
        let ema_200 = *ema_200_vec.last().unwrap_or(&0.0);

        let is_golden_cross = ema_50 > ema_200 && ema_200 > 0.0;
        let detected_patterns = Self::detect_candlestick_patterns(candles);

        // Technical Score Calculation
        let mut score = 5.0f32;

        // RSI Momentum
        if (50.0..=75.0).contains(&rsi_14) {
            score += 1.2; // Healthy bullish momentum
        } else if rsi_14 > 85.0 {
            score -= 0.3; // Extreme overbought caution
        } else if rsi_14 < 30.0 {
            score += 0.8; // Oversold potential bounce
        }

        // MACD Signal (Evaluate turning point crossovers before steady-state momentum levels)
        if macd.is_bullish_crossover {
            score += 1.5;
        } else if macd.is_bearish_crossover {
            score -= 1.5;
        } else if macd.histogram > 0.0 && macd.macd_line > 0.0 {
            score += 0.8;
        } else if macd.histogram < 0.0 && macd.macd_line < 0.0 {
            score -= 0.8;
        }

        // Bollinger Bandwidth
        if bollinger.is_squeeze {
            score += 0.8; // Upcoming breakout setup
        }

        // Moving Average Alignment
        let latest_price = *prices.last().unwrap_or(&0.0);
        if ema_20 > ema_50 && ema_50 > 0.0 {
            score += 1.0; // Bullish short/medium-term trend alignment
        }
        if latest_price > ema_20 && ema_20 > 0.0 {
            score += 0.5; // Trading above 20 EMA
        }
        if is_golden_cross {
            score += 1.2;
        }

        // Pattern Boosts
        for p in &detected_patterns {
            match p.pattern {
                PatternType::MorningStar | PatternType::BullishEngulfing | PatternType::Hammer => {
                    score += 1.0 * p.confidence;
                }
                PatternType::BearishEngulfing | PatternType::EveningStar => {
                    score -= 1.0 * p.confidence;
                }
                _ => {}
            }
        }

        let technical_score = score.clamp(1.0, 10.0);

        TechnicalSummary {
            rsi_14,
            macd,
            bollinger,
            ema_20,
            ema_50,
            ema_200,
            is_golden_cross,
            detected_patterns,
            technical_score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_technical_calculations() {
        let mut candles = Vec::new();
        let base_price = 100.0;
        for i in 0..60 {
            let p = base_price + (i as f64 * 0.8);
            candles.push(OhlcvCandle {
                timestamp_epoch_sec: 1700000000 + (i * 86400),
                open: p - 0.2,
                high: p + 1.0,
                low: p - 0.5,
                close: p,
                volume: 50000.0,
            });
        }

        let summary = TechnicalEngine::evaluate_technicals(&candles);
        assert!(summary.rsi_14 > 50.0);
        assert!(summary.technical_score >= 5.0);
    }
}
