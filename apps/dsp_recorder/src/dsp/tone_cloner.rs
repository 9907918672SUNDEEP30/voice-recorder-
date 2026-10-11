use anyhow::Result;
use rustfft::num_complex::Complex;
use rustfft::{Fft, FftPlanner};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::path::Path;
use std::sync::Arc;

use super::biquad::BiquadCascade;

/// FFT frame size for high-resolution audio spectral analysis (2048 samples = ~23.4Hz/bin at 48kHz).
pub const FFT_SIZE: usize = 2048;

/// Acoustic sub-band frequency intervals in Hertz.
pub const SUB_BASS_RANGE: (f32, f32) = (20.0, 60.0);
pub const BASS_RANGE: (f32, f32) = (60.0, 250.0);
pub const LOW_MID_RANGE: (f32, f32) = (250.0, 500.0);
pub const MID_RANGE: (f32, f32) = (500.0, 2000.0);
pub const HIGH_MID_RANGE: (f32, f32) = (2000.0, 4000.0);
pub const PRESENCE_RANGE: (f32, f32) = (4000.0, 8000.0);
pub const AIR_RANGE: (f32, f32) = (8000.0, 20000.0);

/// Acoustic signature and spectral energy envelope of an audio stream or reference recording.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToneProfile {
    /// 7-band normalized acoustic energy distribution:
    /// [0] Sub-Bass (20-60Hz)
    /// [1] Bass (60-250Hz)
    /// [2] Low-Mids (250-500Hz)
    /// [3] Mids (500-2000Hz)
    /// [4] High-Mids (2000-4000Hz)
    /// [5] Presence (4000-8000Hz)
    /// [6] Air/Brilliance (8000-20000Hz)
    pub band_energies: [f32; 7],
    /// Spectral centroid (perceived timbre brightness in Hz)
    pub spectral_centroid_hz: f32,
    /// Total RMS energy level
    pub rms_level: f32,
    /// Number of analyzed frames accumulated in this profile
    pub frames_analyzed: usize,
}

impl Default for ToneProfile {
    fn default() -> Self {
        Self {
            band_energies: [0.0; 7],
            spectral_centroid_hz: 1000.0,
            rms_level: 0.0,
            frames_analyzed: 0,
        }
    }
}

/// Equalization delta curve calculated to match an input recording's tone to a reference recording.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToneEqMatch {
    pub bass_shelf_db: f32,
    pub low_mid_db: f32,
    pub presence_db: f32,
    pub high_shelf_air_db: f32,
}

impl ToneEqMatch {
    /// Applies the calculated EQ match gains directly to a BiquadCascade DSP engine.
    pub fn apply_to(&self, cascade: &mut BiquadCascade) {
        cascade.set_bass_gain_db(self.bass_shelf_db);
        cascade.set_low_mid_gain_db(self.low_mid_db);
        cascade.set_presence_gain_db(self.presence_db);
        cascade.set_air_gain_db(self.high_shelf_air_db);
    }
}

/// Tone Cloning & Spectral Analysis Engine powered by pure Rust Fast Fourier Transform (rustfft).
pub struct ToneCloner {
    fft: Arc<dyn Fft<f32>>,
    hann_window: Vec<f32>,
    fft_scratch: Vec<Complex<f32>>,
    fft_buffer: Vec<Complex<f32>>,
}

impl ToneCloner {
    /// Initializes the ToneCloner with precomputed Hann window and FFT execution plans.
    pub fn new() -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);

        // Precompute Hann Window to minimize per-frame computation
        let mut hann_window = Vec::with_capacity(FFT_SIZE);
        for i in 0..FFT_SIZE {
            let w = 0.5 * (1.0 - (2.0 * PI * i as f32 / (FFT_SIZE as f32 - 1.0)).cos());
            hann_window.push(w);
        }

        let scratch_len = fft.get_inplace_scratch_len();
        let fft_scratch = vec![Complex::new(0.0, 0.0); scratch_len];
        let fft_buffer = vec![Complex::new(0.0, 0.0); FFT_SIZE];

        Self {
            fft,
            hann_window,
            fft_scratch,
            fft_buffer,
        }
    }

    /// Analyzes a single 2048-sample audio window and returns the magnitude spectrum.
    pub fn compute_magnitude_spectrum(&mut self, windowed_samples: &[f32]) -> Vec<f32> {
        let n = windowed_samples.len().min(FFT_SIZE);

        for i in 0..n {
            self.fft_buffer[i] = Complex::new(windowed_samples[i] * self.hann_window[i], 0.0);
        }
        for i in n..FFT_SIZE {
            self.fft_buffer[i] = Complex::new(0.0, 0.0);
        }

        self.fft.process_with_scratch(&mut self.fft_buffer, &mut self.fft_scratch);

        // Compute magnitude of positive frequencies (first FFT_SIZE / 2 bins)
        let num_bins = FFT_SIZE / 2;
        let mut magnitudes = Vec::with_capacity(num_bins);
        for i in 0..num_bins {
            let mag = self.fft_buffer[i].norm() / (FFT_SIZE as f32 / 2.0);
            magnitudes.push(mag);
        }

        magnitudes
    }

    /// Extracts acoustic band energy distribution from a magnitude spectrum.
    fn integrate_bands(magnitudes: &[f32], sample_rate: f32) -> ([f32; 7], f32) {
        let bin_resolution = sample_rate / FFT_SIZE as f32;
        let mut bands = [1e-7f32; 7]; // small epsilon to avoid log(0)
        let mut centroid_numerator = 0.0f32;
        let mut centroid_denominator = 1e-7f32;

        for (bin_idx, &mag) in magnitudes.iter().enumerate() {
            let freq = bin_idx as f32 * bin_resolution;
            let power = mag * mag;

            centroid_numerator += freq * power;
            centroid_denominator += power;

            if freq >= SUB_BASS_RANGE.0 && freq < SUB_BASS_RANGE.1 {
                bands[0] += power;
            } else if freq >= BASS_RANGE.0 && freq < BASS_RANGE.1 {
                bands[1] += power;
            } else if freq >= LOW_MID_RANGE.0 && freq < LOW_MID_RANGE.1 {
                bands[2] += power;
            } else if freq >= MID_RANGE.0 && freq < MID_RANGE.1 {
                bands[3] += power;
            } else if freq >= HIGH_MID_RANGE.0 && freq < HIGH_MID_RANGE.1 {
                bands[4] += power;
            } else if freq >= PRESENCE_RANGE.0 && freq < PRESENCE_RANGE.1 {
                bands[5] += power;
            } else if freq >= AIR_RANGE.0 && freq < AIR_RANGE.1 {
                bands[6] += power;
            }
        }

        let centroid = centroid_numerator / centroid_denominator;
        (bands, centroid)
    }

    /// Performs complete spectral profiling across a continuous PCM f32 stream (e.g. speech utterance).
    pub fn analyze_audio(&mut self, audio: &[f32], sample_rate: f32) -> ToneProfile {
        if audio.len() < FFT_SIZE {
            return ToneProfile::default();
        }

        let hop_size = FFT_SIZE / 2; // 50% overlap Hann windowing
        let mut accumulated_bands = [0.0f32; 7];
        let mut accumulated_centroid = 0.0f32;
        let mut total_rms_sq = 0.0f32;
        let mut frame_count = 0;

        let mut offset = 0;
        while offset + FFT_SIZE <= audio.len() {
            let frame = &audio[offset..offset + FFT_SIZE];
            let mags = self.compute_magnitude_spectrum(frame);
            let (bands, centroid) = Self::integrate_bands(&mags, sample_rate);

            for i in 0..7 {
                accumulated_bands[i] += bands[i];
            }
            accumulated_centroid += centroid;

            let frame_energy: f32 = frame.iter().map(|s| s * s).sum();
            total_rms_sq += frame_energy / FFT_SIZE as f32;

            frame_count += 1;
            offset += hop_size;
        }

        if frame_count == 0 {
            return ToneProfile::default();
        }

        let inv_frames = 1.0 / frame_count as f32;
        let mut avg_bands = [0.0f32; 7];
        let mut total_energy = 1e-7f32;

        for i in 0..7 {
            avg_bands[i] = accumulated_bands[i] * inv_frames;
            total_energy += avg_bands[i];
        }

        // Normalize band energy distribution so total sum equals 1.0 (shape-only, independent of absolute volume)
        for i in 0..7 {
            avg_bands[i] /= total_energy;
        }

        ToneProfile {
            band_energies: avg_bands,
            spectral_centroid_hz: accumulated_centroid * inv_frames,
            rms_level: (total_rms_sq * inv_frames).sqrt(),
            frames_analyzed: frame_count,
        }
    }

    /// Reads and analyzes a reference WAV file from disk.
    pub fn analyze_wav_file<P: AsRef<Path>>(&mut self, wav_path: P) -> Result<ToneProfile> {
        let path = wav_path.as_ref();
        let mut reader = hound::WavReader::open(path)?;
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

        // Convert multi-channel audio to mono if necessary
        let mono_samples: Vec<f32> = if spec.channels > 1 {
            let ch = spec.channels as usize;
            samples
                .chunks_exact(ch)
                .map(|frame| frame.iter().sum::<f32>() / ch as f32)
                .collect()
        } else {
            samples
        };

        Ok(self.analyze_audio(&mono_samples, spec.sample_rate as f32))
    }

    /// Calculates acoustic tone matching EQ adjustments by comparing input profile to a reference target profile.
    pub fn calculate_match_eq(
        &self,
        input_profile: &ToneProfile,
        target_profile: &ToneProfile,
    ) -> ToneEqMatch {
        // Calculate logarithmic spectral power delta: delta_dB = 10 * log10(target / input)
        let calc_delta_db = |target: f32, input: f32| -> f32 {
            let ratio = (target + 1e-6) / (input + 1e-6);
            let db = 10.0 * ratio.log10();
            // Musical safety clamp: never boost or cut more than +/- 9.0 dB
            db.clamp(-9.0, 9.0)
        };

        // 1. Bass Shelf (Band 1: Bass 60-250Hz)
        let bass_shelf_db = calc_delta_db(target_profile.band_energies[1], input_profile.band_energies[1]);

        // 2. Low-Mid Peaking (Band 2: 250-500Hz)
        let low_mid_db = calc_delta_db(target_profile.band_energies[2], input_profile.band_energies[2]);

        // 3. High-Mid / Presence (Bands 4 & 5 average: 2000-8000Hz)
        let input_presence = (input_profile.band_energies[4] + input_profile.band_energies[5]) * 0.5;
        let target_presence = (target_profile.band_energies[4] + target_profile.band_energies[5]) * 0.5;
        let presence_db = calc_delta_db(target_presence, input_presence);

        // 4. Air / High Shelf (Band 6: 8000-20000Hz)
        let high_shelf_air_db = calc_delta_db(target_profile.band_energies[6], input_profile.band_energies[6]);

        ToneEqMatch {
            bass_shelf_db,
            low_mid_db,
            presence_db,
            high_shelf_air_db,
        }
    }
}
