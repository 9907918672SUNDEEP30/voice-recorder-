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
    /// Teleprompter downward angle in degrees (strictly clamped to -15.0° to +5.0°).
    pub downward_pitch_deg: f32,
}

impl Default for GazeCorrectionConfig {
    fn default() -> Self {
        Self {
            correction_strength: 0.80,
            vertical_lift_ratio: 0.32,
            horizontal_align_ratio: 0.04,
            downward_pitch_deg: 12.0,
        }
    }
}

/// Real-time Gaze Warping Engine implementing natural teleprompter eye contact correction.
/// Features safe ±15° hard limiter, Cubic Hermite smoothstep damping, and bilinear sampling.
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

    /// Sets the teleprompter angle with safe hard clamping between -15° and +5°.
    pub fn set_downward_angle(&mut self, deg: f32) {
        self.config.downward_pitch_deg = deg.clamp(-15.0, 5.0);
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

        // Smooth transition damping (0.25 alpha per frame = 60 FPS exponential ease)
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

        // 1. Safe-Angle Hard Limiter: clamp between -15° and +5°
        let safe_deg = self.config.downward_pitch_deg.clamp(-15.0, 5.0);

        // 2. Cubic Hermite Smoothstep Damping: f(t) = 3t^2 - 2t^3
        // Prevents linear rubber-banding or extreme eyeball stretching
        let t = (safe_deg.abs() / 15.0).clamp(0.0, 1.0);
        let hermite_scale = 3.0 * t * t - 2.0 * t * t * t;

        // 3. Shift Vector bounded strictly to 0.35 * ry maximum displacement
        let max_displacement = ry * 0.35;
        let raw_dy = -ry * self.config.vertical_lift_ratio * hermite_scale * self.active_blend;
        let dy_shift = raw_dy.clamp(-max_displacement, max_displacement);

        let dx_shift = if is_left {
            (rx * self.config.horizontal_align_ratio * self.active_blend).clamp(-rx * 0.15, rx * 0.15)
        } else {
            (-rx * self.config.horizontal_align_ratio * self.active_blend).clamp(-rx * 0.15, rx * 0.15)
        };

        let min_x = (cx - rx).max(0.0) as u32;
        let max_x = (cx + rx).min((frame.width - 1) as f32) as u32;
        let min_y = (cy - ry).max(0.0) as u32;
        let max_y = (cy + ry).min((frame.height - 1) as f32) as u32;

        if max_x <= min_x || max_y <= min_y {
            return;
        }

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

        // 4. Bilinear Sub-Pixel Interpolation for artifact-free broadcast quality
        let sample_patch_bilinear = |sx: f32, sy: f32| -> [f32; 3] {
            let lx = (sx - min_x as f32).clamp(0.0, (box_w - 1) as f32);
            let ly = (sy - min_y as f32).clamp(0.0, (box_h - 1) as f32);

            let x0 = lx.floor() as usize;
            let x1 = (x0 + 1).min(box_w - 1);
            let y0 = ly.floor() as usize;
            let y1 = (y0 + 1).min(box_h - 1);

            let u = lx - x0 as f32;
            let v = ly - y0 as f32;

            let idx00 = (y0 * box_w + x0) * 3;
            let idx10 = (y0 * box_w + x1) * 3;
            let idx01 = (y1 * box_w + x0) * 3;
            let idx11 = (y1 * box_w + x1) * 3;

            let mut out = [0.0f32; 3];
            for c in 0..3 {
                let p00 = patch[idx00 + c] as f32;
                let p10 = patch[idx10 + c] as f32;
                let p01 = patch[idx01 + c] as f32;
                let p11 = patch[idx11 + c] as f32;

                let top = (1.0 - u) * p00 + u * p10;
                let bot = (1.0 - u) * p01 + u * p11;
                out[c] = (1.0 - v) * top + v * bot;
            }
            out
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

                    let warped_rgb = sample_patch_bilinear(src_x, src_y);
                    let orig_rgb = frame.get_pixel(x, y);

                    // Blend warped pixel with original pixel according to weight
                    let mut final_rgb = [0u8; 3];
                    for c in 0..3 {
                        let blended = warped_rgb[c] * weight + orig_rgb[c] as f32 * (1.0 - weight);
                        final_rgb[c] = blended.round().clamp(0.0, 255.0) as u8;
                    }

                    frame.set_pixel(x, y, final_rgb);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaze_correction_safe_angle_clamping() {
        let mut corrector = GazeCorrector::new(GazeCorrectionConfig::default());

        // Extreme downward reading angle should be clamped to -15.0
        corrector.set_downward_angle(-45.0);
        assert_eq!(corrector.config.downward_pitch_deg, -15.0);

        // Extreme upward angle should be clamped to 5.0
        corrector.set_downward_angle(30.0);
        assert_eq!(corrector.config.downward_pitch_deg, 5.0);

        // Safe reading angle within range remains unchanged
        corrector.set_downward_angle(-12.0);
        assert_eq!(corrector.config.downward_pitch_deg, -12.0);
    }
}
