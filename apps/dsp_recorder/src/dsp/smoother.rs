/// Single-pole exponential parameter smoother.
///
/// Prevents audible zipper noise, clicks, and phase discontinuities during
/// live DSP parameter changes (e.g. gain slider movements, dynamic auto-matching).
#[derive(Debug, Clone)]
pub struct ParameterSmoother {
    current_value: f32,
    target_value: f32,
    smoothing_factor: f32,
}

impl ParameterSmoother {
    /// Creates a new parameter smoother initialized with a starting value and time constant.
    ///
    /// # Arguments
    /// * `initial_value` - Starting parameter value
    /// * `smoothing_time_ms` - Transition duration in milliseconds (e.g. 20.0 ms)
    /// * `sample_rate` - Stream sampling rate in Hz
    pub fn new(initial_value: f32, smoothing_time_ms: f32, sample_rate: f32) -> Self {
        let sample_rate = sample_rate.max(8000.0);
        let time_seconds = (smoothing_time_ms / 1000.0).max(0.001);
        // Exponential smoothing coefficient: alpha = 1 - exp(-1 / (sample_rate * tau))
        let factor = 1.0 - (-1.0 / (sample_rate * time_seconds)).exp();

        Self {
            current_value: initial_value,
            target_value: initial_value,
            smoothing_factor: factor.clamp(0.0001, 1.0),
        }
    }

    /// Sets a new target parameter value to smoothly ramp towards.
    pub fn set_target(&mut self, target: f32) {
        self.target_value = target;
    }

    /// Immediately resets both current and target to a specific value without smoothing.
    pub fn set_immediate(&mut self, value: f32) {
        self.current_value = value;
        self.target_value = value;
    }

    /// Advances the smoother by one sample and returns the smoothed value.
    #[inline(always)]
    pub fn next_sample(&mut self) -> f32 {
        let diff = (self.target_value - self.current_value).abs();
        if diff > 1e-4 {
            self.current_value += self.smoothing_factor * (self.target_value - self.current_value);
        } else {
            self.current_value = self.target_value;
        }
        self.current_value
    }

    /// Returns the current smoothed value without advancing time.
    #[inline(always)]
    pub fn current(&self) -> f32 {
        self.current_value
    }

    /// Returns the target value.
    #[inline(always)]
    pub fn target(&self) -> f32 {
        self.target_value
    }

    /// Checks if the parameter has arrived at the target within tolerance.
    pub fn is_settled(&self) -> bool {
        (self.target_value - self.current_value).abs() < 1e-3
    }
}
