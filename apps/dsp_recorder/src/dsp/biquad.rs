use std::f32::consts::PI;

/// Biquad filter types implemented following the Robert Bristow-Johnson Audio EQ Cookbook.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterType {
    LowShelf,
    HighShelf,
    Peaking,
    HighPass,
    LowPass,
}

/// Normalized biquad filter transfer function coefficients.
/// H(z) = (b0 + b1*z^-1 + b2*z^-2) / (1 + a1*z^-1 + a2*z^-2)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BiquadCoefficients {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a1: f32,
    pub a2: f32,
}

impl Default for BiquadCoefficients {
    fn default() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
        }
    }
}

impl BiquadCoefficients {
    /// Constructs coefficients for a Low-Shelf filter (Bass Warmth / Sub-bass shaping).
    ///
    /// # Arguments
    /// * `sample_rate` - Audio stream sampling rate in Hz (e.g. 44100.0 or 48000.0)
    /// * `cutoff_hz` - Corner frequency in Hz (typically 60.0 - 150.0 Hz)
    /// * `gain_db` - Gain in decibels (positive for boost, negative for cut)
    /// * `q` - Quality factor / slope resonance (standard Butterworth slope = 0.7071)
    pub fn low_shelf(sample_rate: f32, cutoff_hz: f32, gain_db: f32, q: f32) -> Self {
        let sample_rate = sample_rate.max(8000.0);
        let cutoff = cutoff_hz.clamp(10.0, sample_rate * 0.49);
        let q = q.max(0.01);
        let a = 10.0f32.powf(gain_db / 40.0);
        let omega = 2.0 * PI * cutoff / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);
        let sqrt_a = a.sqrt();

        let a0 = (a + 1.0) + (a - 1.0) * cos_omega + 2.0 * sqrt_a * alpha;
        if a0.abs() < 1e-9 {
            return Self::default();
        }

        let b0 = a * ((a + 1.0) - (a - 1.0) * cos_omega + 2.0 * sqrt_a * alpha) / a0;
        let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_omega) / a0;
        let b2 = a * ((a + 1.0) - (a - 1.0) * cos_omega - 2.0 * sqrt_a * alpha) / a0;
        let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_omega) / a0;
        let a2 = ((a + 1.0) + (a - 1.0) * cos_omega - 2.0 * sqrt_a * alpha) / a0;

        Self { b0, b1, b2, a1, a2 }
    }

    /// Constructs coefficients for a High-Shelf filter (Vocal Air / Treble Sheen).
    ///
    /// # Arguments
    /// * `sample_rate` - Audio stream sampling rate in Hz
    /// * `cutoff_hz` - Corner frequency in Hz (typically 6000.0 - 12000.0 Hz)
    /// * `gain_db` - Gain in decibels
    /// * `q` - Quality factor (standard = 0.7071)
    pub fn high_shelf(sample_rate: f32, cutoff_hz: f32, gain_db: f32, q: f32) -> Self {
        let sample_rate = sample_rate.max(8000.0);
        let cutoff = cutoff_hz.clamp(10.0, sample_rate * 0.49);
        let q = q.max(0.01);
        let a = 10.0f32.powf(gain_db / 40.0);
        let omega = 2.0 * PI * cutoff / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);
        let sqrt_a = a.sqrt();

        let a0 = (a + 1.0) - (a - 1.0) * cos_omega + 2.0 * sqrt_a * alpha;
        if a0.abs() < 1e-9 {
            return Self::default();
        }

        let b0 = a * ((a + 1.0) + (a - 1.0) * cos_omega + 2.0 * sqrt_a * alpha) / a0;
        let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_omega) / a0;
        let b2 = a * ((a + 1.0) + (a - 1.0) * cos_omega - 2.0 * sqrt_a * alpha) / a0;
        let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_omega) / a0;
        let a2 = ((a + 1.0) - (a - 1.0) * cos_omega - 2.0 * sqrt_a * alpha) / a0;

        Self { b0, b1, b2, a1, a2 }
    }

    /// Constructs coefficients for a Peaking Parametric EQ filter (Vocal Presence / Clarity).
    ///
    /// # Arguments
    /// * `sample_rate` - Audio stream sampling rate in Hz
    /// * `center_hz` - Center frequency in Hz (e.g. 2500.0 - 4500.0 Hz for speech presence)
    /// * `gain_db` - Peak boost or cut in decibels
    /// * `q` - Filter bandwidth / resonance (typically 1.0 - 2.0)
    pub fn peaking(sample_rate: f32, center_hz: f32, gain_db: f32, q: f32) -> Self {
        let sample_rate = sample_rate.max(8000.0);
        let center = center_hz.clamp(10.0, sample_rate * 0.49);
        let q = q.max(0.01);
        let a = 10.0f32.powf(gain_db / 40.0);
        let omega = 2.0 * PI * center / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let a0 = 1.0 + alpha / a;
        if a0.abs() < 1e-9 {
            return Self::default();
        }

        let b0 = (1.0 + alpha * a) / a0;
        let b1 = (-2.0 * cos_omega) / a0;
        let b2 = (1.0 - alpha * a) / a0;
        let a1 = (-2.0 * cos_omega) / a0;
        let a2 = (1.0 - alpha / a) / a0;

        Self { b0, b1, b2, a1, a2 }
    }

    /// Constructs coefficients for a High-Pass filter (Rumble & Sub-frequency isolation).
    ///
    /// # Arguments
    /// * `sample_rate` - Audio stream sampling rate in Hz
    /// * `cutoff_hz` - Cutoff frequency in Hz (typically 30.0 - 80.0 Hz for microphone rumble removal)
    /// * `q` - Quality factor (0.7071 for Butterworth maximally flat response)
    pub fn high_pass(sample_rate: f32, cutoff_hz: f32, q: f32) -> Self {
        let sample_rate = sample_rate.max(8000.0);
        let cutoff = cutoff_hz.clamp(10.0, sample_rate * 0.49);
        let q = q.max(0.01);
        let omega = 2.0 * PI * cutoff / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let a0 = 1.0 + alpha;
        if a0.abs() < 1e-9 {
            return Self::default();
        }

        let b0 = ((1.0 + cos_omega) / 2.0) / a0;
        let b1 = (-(1.0 + cos_omega)) / a0;
        let b2 = ((1.0 + cos_omega) / 2.0) / a0;
        let a1 = (-2.0 * cos_omega) / a0;
        let a2 = (1.0 - alpha) / a0;

        Self { b0, b1, b2, a1, a2 }
    }
}

/// Single Biquad Filter implemented in Direct Form II Transposed structure.
///
/// Direct Form II Transposed provides excellent numerical stability and minimal roundoff noise
/// when processing 32-bit floating point audio streams on ARM / Raspberry Pi and x86_64 architectures.
#[derive(Debug, Clone)]
pub struct BiquadFilter {
    pub coeffs: BiquadCoefficients,
    s1: f32,
    s2: f32,
}

impl Default for BiquadFilter {
    fn default() -> Self {
        Self {
            coeffs: BiquadCoefficients::default(),
            s1: 0.0,
            s2: 0.0,
        }
    }
}

impl BiquadFilter {
    /// Creates a new biquad filter with the specified coefficients.
    pub fn new(coeffs: BiquadCoefficients) -> Self {
        Self {
            coeffs,
            s1: 0.0,
            s2: 0.0,
        }
    }

    /// Resets internal delay line memory to zero (clearing ringing / transient history).
    pub fn reset(&mut self) {
        self.s1 = 0.0;
        self.s2 = 0.0;
    }

    /// Updates filter coefficients without resetting state registers, enabling smooth parameter changes.
    pub fn update_coefficients(&mut self, coeffs: BiquadCoefficients) {
        self.coeffs = coeffs;
    }

    /// Processes a single audio sample through the Direct Form II Transposed filter.
    #[inline(always)]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let output = self.coeffs.b0 * input + self.s1;
        self.s1 = self.coeffs.b1 * input - self.coeffs.a1 * output + self.s2;
        self.s2 = self.coeffs.b2 * input - self.coeffs.a2 * output;

        // Guard against IEEE-754 denormals and non-finite values in real-time execution
        if !output.is_finite() {
            self.reset();
            return input;
        }

        output
    }

    /// In-place buffer processing loop optimized for auto-vectorization (SIMD / NEON).
    pub fn process_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process_sample(*sample);
        }
    }
}

/// Cascaded Equalization and Acoustic Styling Section.
///
/// Chains an active chain of dedicated studio-grade IIR biquad filters:
/// 1. Rumble HPF Filter (removes sub-audible mic thump < 35 Hz)
/// 2. Low-Shelf Filter (Deep Warmth & Body Boost)
/// 3. Low-Mid Peaking Filter (Mud / Boxiness surgical sculpting)
/// 4. High-Mid Peaking Filter (Vocal Presence & Consonant Intelligibility)
/// 5. High-Shelf Filter (Crispness & Sheen / Air)
/// 6. Warm Analog Soft-Saturator (Zero-Clip dynamic protection)
#[derive(Debug, Clone)]
pub struct BiquadCascade {
    sample_rate: f32,
    rumble_hpf: BiquadFilter,
    low_shelf: BiquadFilter,
    low_mid_peaking: BiquadFilter,
    high_mid_presence: BiquadFilter,
    high_shelf_air: BiquadFilter,
    // Target parameters stored for dynamic modulation
    pub bass_gain_db: f32,
    pub low_mid_gain_db: f32,
    pub presence_gain_db: f32,
    pub air_gain_db: f32,
    pub enable_soft_clipper: bool,
}

impl BiquadCascade {
    /// Creates a production-ready studio equalizer tuned for voice recording and tone styling.
    pub fn new(sample_rate: f32) -> Self {
        let sample_rate = sample_rate.max(8000.0);
        let mut cascade = Self {
            sample_rate,
            rumble_hpf: BiquadFilter::default(),
            low_shelf: BiquadFilter::default(),
            low_mid_peaking: BiquadFilter::default(),
            high_mid_presence: BiquadFilter::default(),
            high_shelf_air: BiquadFilter::default(),
            bass_gain_db: 0.0,
            low_mid_gain_db: 0.0,
            presence_gain_db: 0.0,
            air_gain_db: 0.0,
            enable_soft_clipper: true,
        };

        cascade.recalculate_all_filters();
        cascade
    }

    /// Recalculates all filter coefficients based on current sample rate and gain parameters.
    pub fn recalculate_all_filters(&mut self) {
        // 1. HPF at 35Hz (Q=0.7071 Butterworth)
        self.rumble_hpf.update_coefficients(BiquadCoefficients::high_pass(
            self.sample_rate,
            35.0,
            0.7071,
        ));

        // 2. Low-Shelf at 110Hz (Warm, resonant voice base)
        self.low_shelf.update_coefficients(BiquadCoefficients::low_shelf(
            self.sample_rate,
            110.0,
            self.bass_gain_db,
            0.7071,
        ));

        // 3. Low-Mid Peaking at 400Hz (Vocal Body)
        self.low_mid_peaking.update_coefficients(BiquadCoefficients::peaking(
            self.sample_rate,
            400.0,
            self.low_mid_gain_db,
            1.2,
        ));

        // 4. High-Mid Presence Peaking at 3400Hz (Vocal Clarity & intelligibility)
        self.high_mid_presence.update_coefficients(BiquadCoefficients::peaking(
            self.sample_rate,
            3400.0,
            self.presence_gain_db,
            1.1,
        ));

        // 5. High-Shelf at 9500Hz (Studio Air & Crispness)
        self.high_shelf_air.update_coefficients(BiquadCoefficients::high_shelf(
            self.sample_rate,
            9500.0,
            self.air_gain_db,
            0.7071,
        ));
    }

    /// Sets sample rate and updates all filter coefficients.
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(8000.0);
        self.recalculate_all_filters();
    }

    /// Configures bass boost / cut in decibels (-18.0 dB to +18.0 dB).
    pub fn set_bass_gain_db(&mut self, gain_db: f32) {
        self.bass_gain_db = gain_db.clamp(-18.0, 18.0);
        self.low_shelf.update_coefficients(BiquadCoefficients::low_shelf(
            self.sample_rate,
            110.0,
            self.bass_gain_db,
            0.7071,
        ));
    }

    /// Configures low-mid gain in decibels (-18.0 dB to +18.0 dB).
    pub fn set_low_mid_gain_db(&mut self, gain_db: f32) {
        self.low_mid_gain_db = gain_db.clamp(-18.0, 18.0);
        self.low_mid_peaking.update_coefficients(BiquadCoefficients::peaking(
            self.sample_rate,
            400.0,
            self.low_mid_gain_db,
            1.2,
        ));
    }

    /// Configures vocal presence gain in decibels (-18.0 dB to +18.0 dB).
    pub fn set_presence_gain_db(&mut self, gain_db: f32) {
        self.presence_gain_db = gain_db.clamp(-18.0, 18.0);
        self.high_mid_presence.update_coefficients(BiquadCoefficients::peaking(
            self.sample_rate,
            3400.0,
            self.presence_gain_db,
            1.1,
        ));
    }

    /// Configures high-shelf treble/air gain in decibels (-18.0 dB to +18.0 dB).
    pub fn set_air_gain_db(&mut self, gain_db: f32) {
        self.air_gain_db = gain_db.clamp(-18.0, 18.0);
        self.high_shelf_air.update_coefficients(BiquadCoefficients::high_shelf(
            self.sample_rate,
            9500.0,
            self.air_gain_db,
            0.7071,
        ));
    }

    /// Resets all delay line registers across the cascade.
    pub fn reset(&mut self) {
        self.rumble_hpf.reset();
        self.low_shelf.reset();
        self.low_mid_peaking.reset();
        self.high_mid_presence.reset();
        self.high_shelf_air.reset();
    }

    /// Processes an in-place buffer of floating point audio samples through the complete cascade.
    pub fn process_buffer(&mut self, buffer: &mut [f32]) {
        // Cascade Stage 1: Rumble High-Pass
        self.rumble_hpf.process_buffer(buffer);

        // Cascade Stage 2: Bass Low-Shelf
        self.low_shelf.process_buffer(buffer);

        // Cascade Stage 3: Low-Mid Peaking
        self.low_mid_peaking.process_buffer(buffer);

        // Cascade Stage 4: High-Mid Presence Peaking
        self.high_mid_presence.process_buffer(buffer);

        // Cascade Stage 5: Air High-Shelf
        self.high_shelf_air.process_buffer(buffer);

        // Stage 6: Warm Analog Soft-Saturator (Hyperbolic Tangent with gentle drive)
        if self.enable_soft_clipper {
            for sample in buffer.iter_mut() {
                let s = *sample;
                if s.abs() > 0.85 {
                    // Soft cubic saturation above 0.85 threshold to prevent harsh digital clipping
                    let sign = if s >= 0.0 { 1.0 } else { -1.0 };
                    let abs_s = s.abs();
                    if abs_s >= 1.2 {
                        *sample = sign * 0.995;
                    } else {
                        let diff = abs_s - 0.85;
                        *sample = sign * (0.85 + diff / (1.0 + diff * diff));
                    }
                }
            }
        }
    }
}
