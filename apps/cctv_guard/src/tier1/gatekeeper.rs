use crate::rtsp::frame::CctvFrame;
use crate::tier1::frame_diff::FrameDifferencer;
use crate::tier1::ssim::StructuralDissimilarity;
use std::collections::VecDeque;
use std::time::Instant;

/// Configuration parameters for Tier-1 edge gating, hysteresis, and alert cooldown.
#[derive(Debug, Clone)]
pub struct GatekeeperConfig {
    /// High anomaly threshold to trip alarm into ACTIVE state [0.0, 1.0]
    pub trigger_threshold_high: f32,
    /// Low threshold to drop back into QUIET state [0.0, 1.0]
    pub reset_threshold_low: f32,
    /// Minimum quiet interval in seconds before allowing another VLM invocation
    pub cooldown_duration_secs: f64,
    /// Weight assigned to pixel motion ratio
    pub motion_weight: f32,
    /// Weight assigned to structural dissimilarity
    pub ssim_weight: f32,
    /// Number of pre-event frames to retain in circular buffer
    pub pre_buffer_capacity: usize,
}

impl Default for GatekeeperConfig {
    fn default() -> Self {
        Self {
            trigger_threshold_high: 0.04,
            reset_threshold_low: 0.015,
            cooldown_duration_secs: 15.0,
            motion_weight: 0.60,
            ssim_weight: 0.40,
            pre_buffer_capacity: 10,
        }
    }
}

/// Evaluation decision produced by the Tier-1 gatekeeper on each incoming frame.
#[derive(Debug, Clone)]
pub enum GatekeeperDecision {
    /// Normal peaceful state; Tier-2 VLM remains completely dormant/unloaded.
    Normal {
        anomaly_score: f32,
    },
    /// Anomaly detected, but suppressed because the cooldown timer is active.
    SuppressedCooldown {
        anomaly_score: f32,
        remaining_secs: f64,
    },
    /// Active anomaly trigger: Tier-2 Qwen VLM must be invoked immediately!
    TriggerVlm {
        anomaly_score: f32,
        event_frame: CctvFrame,
        pre_buffer: Vec<CctvFrame>,
        trigger_reason: String,
    },
}

/// Tier-1 Edge Gatekeeper service maintaining rolling background models,
/// structural dissimilarity, hysteresis filters, and cooldown timers.
pub struct Tier1Gatekeeper {
    config: GatekeeperConfig,
    differencer: FrameDifferencer,
    ssim: StructuralDissimilarity,
    pre_buffer: VecDeque<CctvFrame>,
    /// State tracking: whether the gatekeeper is currently in an active alarm state
    is_alarm_active: bool,
    /// Timestamp of last successful Tier-2 VLM invocation
    last_trigger_instant: Option<Instant>,
}

impl Tier1Gatekeeper {
    /// Constructs a new Tier-1 Gatekeeper instance.
    pub fn new(config: GatekeeperConfig) -> Self {
        let differencer = FrameDifferencer::new(0.04, 14);
        let ssim = StructuralDissimilarity::new(300); // refresh reference every 300 frames (~10s)
        let pre_buffer = VecDeque::with_capacity(config.pre_buffer_capacity);

        Self {
            config,
            differencer,
            ssim,
            pre_buffer,
            is_alarm_active: false,
            last_trigger_instant: None,
        }
    }

    /// Evaluates an incoming high-resolution frame.
    /// Operates at < 5% CPU overhead by downscaling internally to 160x120 grayscale.
    pub fn evaluate_frame(&mut self, frame: CctvFrame) -> GatekeeperDecision {
        // 1. Fast downscaling to 160x120 grayscale
        let downscaled = frame.downscale_to_160x120();

        // 2. Compute motion differencing
        let motion_metrics = self.differencer.process(&downscaled);

        // 3. Compute structural dissimilarity (DSSIM)
        let dssim_score = self.ssim.compute(&downscaled);

        // 4. Reject global illumination flashes (e.g. lights suddenly turned on)
        if motion_metrics.is_global_illumination_flicker {
            self.ssim.set_reference(&downscaled);
            self.maintain_pre_buffer(frame);
            return GatekeeperDecision::Normal { anomaly_score: 0.0 };
        }

        // 5. Calculate composite anomaly score
        let composite_score = (self.config.motion_weight * motion_metrics.motion_ratio)
            + (self.config.ssim_weight * dssim_score);

        // 6. Apply Hysteresis state transitions
        if !self.is_alarm_active {
            if composite_score >= self.config.trigger_threshold_high {
                self.is_alarm_active = true;
            }
        } else if composite_score <= self.config.reset_threshold_low {
            self.is_alarm_active = false;
        }

        // 7. Check cooldown timer and determine action
        if self.is_alarm_active {
            let now = Instant::now();
            let is_in_cooldown = if let Some(last_time) = self.last_trigger_instant {
                let elapsed = now.duration_since(last_time).as_secs_f64();
                elapsed < self.config.cooldown_duration_secs
            } else {
                false
            };

            if is_in_cooldown {
                let elapsed = now.duration_since(self.last_trigger_instant.unwrap()).as_secs_f64();
                let remaining = (self.config.cooldown_duration_secs - elapsed).max(0.0);
                self.maintain_pre_buffer(frame);
                GatekeeperDecision::SuppressedCooldown {
                    anomaly_score: composite_score,
                    remaining_secs: remaining,
                }
            } else {
                // Trip alarm & trigger Tier-2 VLM!
                self.last_trigger_instant = Some(now);

                let pre_frames: Vec<CctvFrame> = self.pre_buffer.iter().cloned().collect();
                let reason = format!(
                    "Edge Anomaly Score {:.3} (Motion: {:.3}, DSSIM: {:.3}) exceeded threshold {:.3}",
                    composite_score,
                    motion_metrics.motion_ratio,
                    dssim_score,
                    self.config.trigger_threshold_high
                );

                self.maintain_pre_buffer(frame.clone());

                GatekeeperDecision::TriggerVlm {
                    anomaly_score: composite_score,
                    event_frame: frame,
                    pre_buffer: pre_frames,
                    trigger_reason: reason,
                }
            }
        } else {
            self.maintain_pre_buffer(frame);
            GatekeeperDecision::Normal {
                anomaly_score: composite_score,
            }
        }
    }

    /// Stores the latest frame into the pre-event circular buffer.
    fn maintain_pre_buffer(&mut self, frame: CctvFrame) {
        if self.pre_buffer.len() >= self.config.pre_buffer_capacity {
            self.pre_buffer.pop_front();
        }
        self.pre_buffer.push_back(frame);
    }
}
