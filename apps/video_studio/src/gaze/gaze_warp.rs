use std::f32::consts::PI;

use crate::capture::VideoFrame;
use crate::gaze::iris_tracker::EyeRegion;

/// Configuration for real-time AI Eye Contact Correction.
#[derive(Debug, Clone)]
pub struct GazeCorrectionConfig {
    /// Correction strength [0.0 = off, 0.5 = subtle natural, 0.85 = standard teleprompter, 1.0 = maximum].
    pub correction_strength: f32,
    /// Vertical optical shift bias (negative lifts gaze upwards towards camera lens).
    pub vertical_lift_ratio: f32,
    /// Lateral inward convergence ratio.
    pub horizontal_align_ratio: f32,
}

impl Default for GazeCorrectionConfig {
    fn default() -> Self {
        Self {
            correction_strength: 0.80,
            vertical_lift_ratio: 0.35,
            horizontal_align_ratio: 0.05,
        }
    }
}

/// Real-time Gaze Warping Engine implementing natural teleprompter eye contact correction.
pub struct GazeCorrector {
    config: GazeCorrectionConfig,
    active_blend: f32,
}

impl GazeCorrector {
    pub fn new(config: GazeCorrectionConfig) -> Self {
        Self {
            config,
            active_blend: 0.0,
        }
    }

    /// Configures correction strength dynamically.
    pub fn set_strength(&mut self, strength: f32) {
        self.config.correction_strength = strength.clamp(0.0, 1.0);
    }

    /// Applies real-time natural eye contact correction on both eyes.
    pub fn apply_eye_contact(
        &mut self,
        frame: &mut VideoFrame,
        left_eye: &EyeRegion,
        right_eye: &EyeRegion,
    ) {
        if self.config.correction_strength <= 0.01 {
            return;
        }

        // If either eye is closed (blinking), gently fade out warp to prevent tearing artifacts
        let target_blend = if left_eye.is_open && right_eye.is_open {
            self.config.correction_strength
        } else {
            0.0
        };

        // Smooth transition damping (0.25 alpha per frame)
        self.active_blend += 0.25 * (target_blend - self.active_blend);
        if self.active_blend <= 0.01 {
            return;
        }

        self.warp_single_eye(frame, left_eye, true);
        self.warp_single_eye(frame, right_eye, false);
    }

    /// Warps a single eye socket region towards the camera optical axis.
    fn warp_single_eye(&self, frame: &mut VideoFrame, eye: &EyeRegion, is_left: bool) {
        let rx = eye.radius_x;
        let ry = eye.radius_y;
        if rx < 2.0 || ry < 2.0 {
            return;
        }

        let cx = eye.center_x;
        let cy = eye.center_y;

        // Calculate shift vector: lift pupil upwards towards lens
        let dy_shift = -ry * self.config.vertical_lift_ratio * self.active_blend;
        let dx_shift = if is_left {
            rx * self.config.horizontal_align_ratio * self.active_blend
        } else {
            -rx * self.config.horizontal_align_ratio * self.active_blend
        };

        let min_x = (cx - rx).max(0.0) as u32;
        let max_x = (cx + rx).min((frame.width - 1) as f32) as u32;
        let min_y = (cy - ry).max(0.0) as u32;
        let max_y = (cy + ry).min((frame.height - 1) as f32) as u32;

        // Snapshot original eye socket pixels to sample from during backward warping
        let box_w = (max_x - min_x + 1) as usize;
        let box_h = (max_y - min_y + 1) as usize;
        let mut patch = Vec::with_capacity(box_w * box_h * 3);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let rgb = frame.get_pixel(x, y);
                patch.extend_from_slice(&rgb);
            }
        }

        let sample_patch = |sx: f32, sy: f32| -> [u8; 3] {
            let lx = (sx - min_x as f32).clamp(0.0, (box_w - 1) as f32) as usize;
            let ly = (sy - min_y as f32).clamp(0.0, (box_h - 1) as f32) as usize;
            let idx = (ly * box_w + lx) * 3;
            [patch[idx], patch[idx + 1], patch[idx + 2]]
        };

        // Radial backward mapping with cosine feathering
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let norm_dx = (x as f32 - cx) / rx;
                let norm_dy = (y as f32 - cy) / ry;
                let r_sq = norm_dx * norm_dx + norm_dy * norm_dy;

                if r_sq < 1.0 {
                    let r = r_sq.sqrt();
                    // Cosine feathering mask: 1.0 at center, smoothly decays to 0.0 at outer ellipse
                    let weight = 0.5 * (1.0 + (PI * r).cos());

                    // Backward warp lookup coordinate
                    let src_x = x as f32 - dx_shift * weight;
                    let src_y = y as f32 - dy_shift * weight;

                    let warped_rgb = sample_patch(src_x, src_y);
                    let orig_rgb = frame.get_pixel(x, y);

                    // Blend warped pixel with original pixel according to weight
                    let mut final_rgb = [0u8; 3];
                    for c in 0..3 {
                        let blended = warped_rgb[c] as f32 * weight + orig_rgb[c] as f32 * (1.0 - weight);
                        final_rgb[c] = blended.round().clamp(0.0, 255.0) as u8;
                    }

                    frame.set_pixel(x, y, final_rgb);
                }
            }
        }
    }
}
