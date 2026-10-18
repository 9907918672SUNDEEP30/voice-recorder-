//! # Clinical Acoustic Voice Conviction Scorer
//!
//! Provides deterministic clinical acoustic biomarker analysis:
//! - Local Pitch Jitter (cycle-to-cycle frequency variation under hesitation/doubt)
//! - Local Amplitude Shimmer (cycle-to-cycle amplitude perturbation)
//! - Harmonics-to-Noise Ratio (HNR in dB, resonant projection vs. turbulent noise)
//! - Composite Conviction Score (0.0 to 100.0) with clinical diagnostic tier.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

/// Clinical Conviction Level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConvictionRating {
    /// Rock-solid subconscious conviction, pristine resonance, zero tremor
    UnshakeableCertainty,
    /// Strong, confident vocal projection with minimal fluctuation
    StrongConviction,
    /// Noticeable hesitation or slight vocal instability
    ModerateHesitation,
    /// Trembling, high jitter/shimmer indicating internal resistance or doubt
    DoubtfulOrAnxious,
}

/// Comprehensive Acoustic Biomarker Metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcousticConvictionMetrics {
    /// Fundamental frequency (F0 in Hz, typical human speech: 85 - 260 Hz)
    pub fundamental_frequency_hz: f32,
    /// Pitch Jitter % (cycle-to-cycle period perturbation; lower is better, < 1.0% is clinical normal)
    pub pitch_jitter_pct: f32,
    /// Amplitude Shimmer % (cycle-to-cycle peak amplitude perturbation; lower is better, < 3.0% is clinical normal)
    pub amplitude_shimmer_pct: f32,
    /// Harmonics-to-Noise Ratio in dB (> 18 dB indicates strong authoritative resonance)
    pub hnr_db: f32,
    /// Overall Root-Mean-Square (RMS) loudness
    pub rms_level: f32,
    /// Normalized Conviction Score (0.0 - 100.0)
    pub conviction_score: f32,
    /// Clinical rating
    pub rating: ConvictionRating,
}

pub struct ConvictionAnalyzer;

impl ConvictionAnalyzer {
    /// Analyzes a mono f32 PCM audio slice at the given sample rate
    pub fn analyze_samples(samples: &[f32], sample_rate: f32) -> Result<AcousticConvictionMetrics> {
        // MM-02 Guard: Enforce valid sample rate to prevent division-by-zero or infinite loop
        if sample_rate < 8000.0 || sample_rate > 192000.0 || sample_rate.is_nan() {
            return Err(anyhow!("Invalid sample rate {}: must be between 8,000 and 192,000 Hz", sample_rate));
        }

        let n = samples.len();
        if n < (sample_rate * 0.1) as usize {
            return Err(anyhow!("Audio too short for clinical conviction analysis (min 100ms required)"));
        }

        // 1. Compute RMS energy
        let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
        let rms = (sum_sq / n as f32).sqrt();
        if rms < 0.005 {
            return Err(anyhow!("Audio signal too quiet or silent for vocal analysis"));
        }

        // 2. Pitch Detection via Normalized Autocorrelation Function (ACF)
        // Search range for human speech F0: 70 Hz to 400 Hz
        let min_lag = (sample_rate / 400.0).floor() as usize;
        let max_lag = (sample_rate / 70.0).ceil() as usize;

        if max_lag >= n {
            return Err(anyhow!("Sample buffer shorter than maximum pitch lag"));
        }

        let mut r_max = 0.0f32;
        let mut best_lag = min_lag;

        for lag in min_lag..=max_lag.min(n / 2) {
            let mut cross_sum = 0.0f32;
            let mut sub_e1 = 0.0f32;
            let mut sub_e2 = 0.0f32;

            for i in 0..(n - lag) {
                let s1 = samples[i];
                let s2 = samples[i + lag];
                cross_sum += s1 * s2;
                sub_e1 += s1 * s1;
                sub_e2 += s2 * s2;
            }

            let denom = (sub_e1 * sub_e2).sqrt().max(1e-9);
            let r_norm = cross_sum / denom;

            if r_norm > r_max {
                r_max = r_norm;
                best_lag = lag;
            }
        }

        let f0 = if best_lag > 0 {
            sample_rate / best_lag as f32
        } else {
            120.0
        };

        // 3. Harmonics-to-Noise Ratio (HNR in dB)
        // HNR = 10 * log10(r_max / (1.0 - r_max))
        let clamped_r = r_max.clamp(0.01, 0.999);
        let hnr_db = 10.0 * (clamped_r / (1.0 - clamped_r)).log10();

        // 4. Glottal cycle peak tracking for true cycle-to-cycle Jitter and Shimmer (Fix MM-01)
        let mut glottal_peaks: Vec<(usize, f32)> = Vec::new();
        let search_window = best_lag;
        let mut curr = 0;

        while curr + search_window <= n {
            let window_end = (curr + search_window).min(n);
            let mut local_max_val = 0.0f32;
            let mut local_max_idx = curr;

            for i in curr..window_end {
                let abs_val = samples[i].abs();
                if abs_val > local_max_val {
                    local_max_val = abs_val;
                    local_max_idx = i;
                }
            }

            if local_max_val > 0.001 {
                glottal_peaks.push((local_max_idx, local_max_val));
            }

            // Advance by approximately one pitch period from the detected peak
            curr = local_max_idx + (search_window * 3 / 4).max(1);
        }

        // Cycle periods and peak amplitudes
        let mut periods: Vec<f32> = Vec::new();
        let mut peak_amplitudes: Vec<f32> = Vec::new();

        for i in 0..glottal_peaks.len() {
            peak_amplitudes.push(glottal_peaks[i].1);
            if i + 1 < glottal_peaks.len() {
                let period = (glottal_peaks[i + 1].0 - glottal_peaks[i].0) as f32;
                // Filter out octave jumps (must be within 0.5x to 2.0x of nominal best_lag)
                if period >= (best_lag as f32 * 0.5) && period <= (best_lag as f32 * 2.0) {
                    periods.push(period);
                }
            }
        }

        // 5. Calculate Local Jitter % (true cycle-to-cycle lag-1 difference)
        let jitter_pct = if periods.len() >= 2 {
            let diff_sum: f32 = periods.windows(2).map(|w| (w[1] - w[0]).abs()).sum();
            let mean_p: f32 = periods.iter().sum::<f32>() / periods.len() as f32;
            let avg_diff = diff_sum / (periods.len() - 1) as f32;
            ((avg_diff / mean_p.max(1e-6)) * 100.0).clamp(0.05, 15.0)
        } else {
            ((1.0 - clamped_r) * 5.0).clamp(0.1, 10.0)
        };

        // 6. Calculate Local Shimmer % (cycle-to-cycle amplitude perturbation)
        let shimmer_pct = if peak_amplitudes.len() >= 2 {
            let diff_sum: f32 = peak_amplitudes.windows(2).map(|w| (w[1] - w[0]).abs()).sum();
            let mean_a: f32 = peak_amplitudes.iter().sum::<f32>() / peak_amplitudes.len() as f32;
            let avg_diff = diff_sum / (peak_amplitudes.len() - 1) as f32;
            ((avg_diff / mean_a.max(1e-6)) * 100.0).clamp(0.2, 25.0)
        } else {
            ((1.0 - clamped_r) * 10.0).clamp(0.5, 20.0)
        };

        // 7. Composite Conviction Score (0.0 to 100.0)
        // Score = [(1.0 - norm_jitter) * 0.35 + (1.0 - norm_shimmer) * 0.25 + (hnr / 25.0) * 0.40] * 100
        let norm_jitter = (jitter_pct / 5.0).clamp(0.0, 1.0);
        let norm_shimmer = (shimmer_pct / 15.0).clamp(0.0, 1.0);
        let norm_hnr = (hnr_db / 25.0).clamp(0.0, 1.0);

        let conviction_score = (
            (1.0 - norm_jitter) * 35.0
            + (1.0 - norm_shimmer) * 25.0
            + norm_hnr * 40.0
        ).clamp(0.0, 100.0);

        let rating = if conviction_score >= 82.0 {
            ConvictionRating::UnshakeableCertainty
        } else if conviction_score >= 65.0 {
            ConvictionRating::StrongConviction
        } else if conviction_score >= 45.0 {
            ConvictionRating::ModerateHesitation
        } else {
            ConvictionRating::DoubtfulOrAnxious
        };

        Ok(AcousticConvictionMetrics {
            fundamental_frequency_hz: f0,
            pitch_jitter_pct: jitter_pct,
            amplitude_shimmer_pct: shimmer_pct,
            hnr_db,
            rms_level: rms,
            conviction_score,
            rating,
        })
    }

    /// Analyzes raw WAV audio bytes from user's microphone recording
    pub fn analyze_wav_bytes(wav_bytes: &[u8]) -> Result<AcousticConvictionMetrics> {
        let cursor = Cursor::new(wav_bytes);
        let mut reader = hound::WavReader::new(cursor)?;
        let spec = reader.spec();

        let samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Float => reader.samples::<f32>().filter_map(|s| s.ok()).collect(),
            hound::SampleFormat::Int => {
                let max_val = (1i32 << (spec.bits_per_sample - 1)) as f32;
                reader
                    .samples::<i32>()
                    .filter_map(|s| s.ok())
                    .map(|s| s as f32 / max_val)
                    .collect()
            }
        };

        let mono_samples: Vec<f32> = if spec.channels > 1 {
            let ch = spec.channels as usize;
            samples
                .chunks_exact(ch)
                .map(|frame| frame.iter().sum::<f32>() / ch as f32)
                .collect()
        } else {
            samples
        };

        Self::analyze_samples(&mono_samples, spec.sample_rate as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_pure_sine_wave_conviction() {
        // High periodicity, zero tremor -> High Conviction
        let sample_rate = 16000.0;
        let freq = 120.0; // Typical male F0
        let mut samples = Vec::with_capacity(16000);

        for i in 0..16000 {
            let t = i as f32 / sample_rate;
            let val = (2.0 * std::f32::consts::PI * freq * t).sin() * 0.7;
            samples.push(val);
        }

        let metrics = ConvictionAnalyzer::analyze_samples(&samples, sample_rate)
            .expect("Should analyze clean sine wave");

        assert!(metrics.fundamental_frequency_hz > 115.0 && metrics.fundamental_frequency_hz < 125.0);
        assert!(metrics.hnr_db > 15.0);
        assert!(metrics.conviction_score >= 70.0);
        assert!(metrics.rating == ConvictionRating::UnshakeableCertainty || metrics.rating == ConvictionRating::StrongConviction);
    }

    #[test]
    fn test_short_audio_error() {
        let samples = vec![0.1f32; 100]; // Far less than 100ms
        assert!(ConvictionAnalyzer::analyze_samples(&samples, 16000.0).is_err());
    }

    #[test]
    fn test_silent_audio_error() {
        let samples = vec![0.0f32; 3200];
        assert!(ConvictionAnalyzer::analyze_samples(&samples, 16000.0).is_err());
    }
}
