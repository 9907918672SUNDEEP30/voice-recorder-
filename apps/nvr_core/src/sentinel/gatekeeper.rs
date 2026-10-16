//! Tier-1 Sentinel Edge Gatekeeper ("पहरेदार")
//! Runs continuously at < 3% CPU overhead, gating quiet frames and triggering Tier-2 AI only on anomalies.

use super::dssim::StructuralComparator;
use super::luma::{LumaDownscaler, LumaGrid, TARGET_HEIGHT, TARGET_WIDTH};
use std::time::{Duration, Instant};

/// Output decision of the Tier-1 Gatekeeper
#[derive(Debug, Clone, PartialEq)]
pub enum GatekeeperDecision {
    /// Room or zone is quiet/static. Keep Tier-2 AI asleep to conserve CPU/RAM.
    SafeIdle,
    /// Anomaly or breach detected above threshold. Wake up Tier-2 AI for threat classification.
    CandidateAnomaly {
        dssim_score: f32,
        changed_pixels_ratio: f32,
    },
}

/// Tier-1 Edge Gatekeeper State Machine
pub struct SentinelGatekeeper {
    pub camera_id: String,
    pub motion_threshold: f32,
    pub cooldown_duration: Duration,
    background_model: Option<LumaGrid>,
    last_trigger_time: Option<Instant>,
    learning_rate: f32,
}

impl SentinelGatekeeper {
    pub fn new(camera_id: impl Into<String>, motion_threshold: f32, cooldown_secs: f32) -> Self {
        Self {
            camera_id: camera_id.into(),
            motion_threshold,
            cooldown_duration: Duration::from_secs_f32(cooldown_secs),
            background_model: None,
            last_trigger_time: None,
            learning_rate: 0.04, // Smooth running background update
        }
    }

    /// Process a new incoming camera frame
    pub fn process_frame(&mut self, rgb_data: &[u8], width: u32, height: u32) -> GatekeeperDecision {
        let current_luma = LumaDownscaler::downscale_rgb(rgb_data, width, height);

        let bg = match &mut self.background_model {
            Some(bg) => bg,
            None => {
                // Initialize baseline background on first frame
                self.background_model = Some(current_luma);
                return GatekeeperDecision::SafeIdle;
            }
        };

        // 1. Compute pixel difference
        let mut changed_pixels = 0usize;
        const DIFF_THRESHOLD: u8 = 20;
        let total_pixels = TARGET_WIDTH * TARGET_HEIGHT;

        for i in 0..total_pixels {
            let curr = current_luma.pixels[i];
            let prev = bg.pixels[i];
            let diff = (curr as i16 - prev as i16).abs() as u8;

            if diff > DIFF_THRESHOLD {
                changed_pixels += 1;
            }
        }

        let changed_ratio = changed_pixels as f32 / total_pixels as f32;

        // 2. Compute Structural Dissimilarity (DSSIM) against reference background
        let dssim = StructuralComparator::compute_dssim(bg, &current_luma);

        // 3. Update adaptive background average: BG = (1 - a)*BG + a*Curr
        for i in 0..total_pixels {
            let curr = current_luma.pixels[i];
            let prev = bg.pixels[i];
            let updated = (1.0 - self.learning_rate) * (prev as f32) + self.learning_rate * (curr as f32);
            bg.pixels[i] = updated as u8;
        }

        // 4. Check Cooldown
        let now = Instant::now();
        if let Some(last_trigger) = self.last_trigger_time {
            if now.duration_since(last_trigger) < self.cooldown_duration {
                return GatekeeperDecision::SafeIdle;
            }
        }

        // 5. Hysteresis Gating Decision (Breach or Structural Shift)
        if dssim > self.motion_threshold || changed_ratio > (self.motion_threshold * 0.5) {
            self.last_trigger_time = Some(now);
            GatekeeperDecision::CandidateAnomaly {
                dssim_score: dssim.max(changed_ratio),
                changed_pixels_ratio: changed_ratio,
            }
        } else {
            GatekeeperDecision::SafeIdle
        }
    }
}
