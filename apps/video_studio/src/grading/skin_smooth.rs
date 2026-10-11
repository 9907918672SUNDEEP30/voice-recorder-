use crate::capture::VideoFrame;
use crate::tracking::face_detector::FaceBox;

/// Studio Skin Smoothing & Relighting Engine.
pub struct SkinFilter {
    pub smooth_strength: f32,
    pub relighting_intensity: f32,
}

impl Default for SkinFilter {
    fn default() -> Self {
        Self {
            smooth_strength: 0.60,
            relighting_intensity: 0.25,
        }
    }
}

impl SkinFilter {
    pub fn new(smooth_strength: f32, relighting_intensity: f32) -> Self {
        Self {
            smooth_strength: smooth_strength.clamp(0.0, 1.0),
            relighting_intensity: relighting_intensity.clamp(0.0, 1.0),
        }
    }

    /// Applies localized edge-preserving smoothing over the detected face region.
    pub fn apply_skin_smoothing(&self, frame: &mut VideoFrame, face_opt: Option<&FaceBox>) {
        if self.smooth_strength <= 0.05 {
            return;
        }

        let Some(face) = face_opt else { return };
        let (px, py, pw, ph) = face.to_pixel_rect(frame.width, frame.height);
        if pw < 16 || ph < 16 {
            return;
        }

        let radius = 2i32;
        let edge_variance_threshold = 450.0f32; // Higher variance indicates sharp eye/lip/hair edges
        let blend = self.smooth_strength * 0.70;

        let max_x = (px + pw).min(frame.width);
        let max_y = (py + ph).min(frame.height);

        // Snapshot face region to prevent recursive sampling
        let roi_w = (max_x - px) as usize;
        let roi_h = (max_y - py) as usize;
        let mut roi_snapshot = Vec::with_capacity(roi_w * roi_h * 3);

        for y in py..max_y {
            for x in px..max_x {
                roi_snapshot.extend_from_slice(&frame.get_pixel(x, y));
            }
        }

        let sample_roi = |x: i32, y: i32| -> [f32; 3] {
            let cx = (x - px as i32).clamp(0, roi_w as i32 - 1) as usize;
            let cy = (y - py as i32).clamp(0, roi_h as i32 - 1) as usize;
            let idx = (cy * roi_w + cx) * 3;
            [
                roi_snapshot[idx] as f32,
                roi_snapshot[idx + 1] as f32,
                roi_snapshot[idx + 2] as f32,
            ]
        };

        for y in py..max_y {
            for x in px..max_x {
                let orig_rgb = frame.get_pixel(x, y);

                // Quick skin-tone test: Only smooth pixels resembling skin chrominance
                let r = orig_rgb[0] as f32;
                let g = orig_rgb[1] as f32;
                let b = orig_rgb[2] as f32;

                if r > 80.0 && g > 40.0 && b > 20.0 && r > g && r > b {
                    // Calculate local neighborhood mean and variance
                    let mut sum_r = 0.0f32;
                    let mut sum_g = 0.0f32;
                    let mut sum_b = 0.0f32;
                    let mut sum_sq_r = 0.0f32;
                    let mut count = 0.0f32;

                    for dy in -radius..=radius {
                        for dx in -radius..=radius {
                            let neighbor = sample_roi(x as i32 + dx, y as i32 + dy);
                            sum_r += neighbor[0];
                            sum_g += neighbor[1];
                            sum_b += neighbor[2];
                            sum_sq_r += neighbor[0] * neighbor[0];
                            count += 1.0;
                        }
                    }

                    let mean_r = sum_r / count;
                    let mean_g = sum_g / count;
                    let mean_b = sum_b / count;
                    let variance_r = (sum_sq_r / count) - (mean_r * mean_r);

                    // Edge-preserving weighting: If variance is low, smooth heavily; if high (edge), preserve original
                    if variance_r < edge_variance_threshold {
                        let edge_factor = (variance_r / edge_variance_threshold).clamp(0.0, 1.0);
                        let effective_blend = blend * (1.0 - edge_factor);

                        let new_r = (r * (1.0 - effective_blend) + mean_r * effective_blend).round() as u8;
                        let new_g = (g * (1.0 - effective_blend) + mean_g * effective_blend).round() as u8;
                        let new_b = (b * (1.0 - effective_blend) + mean_b * effective_blend).round() as u8;

                        frame.set_pixel(x, y, [new_r, new_g, new_b]);
                    }
                }
            }
        }
    }

    /// Applies Studio Relighting and gentle radial face illumination.
    pub fn apply_studio_relighting(&self, frame: &mut VideoFrame, face_opt: Option<&FaceBox>) {
        if self.relighting_intensity <= 0.05 {
            return;
        }

        let fw = frame.width as f32;
        let fh = frame.height as f32;

        let (light_cx, light_cy) = match face_opt {
            Some(face) => (face.center().0 * fw, face.center().1 * fh),
            None => (fw * 0.5, fh * 0.45),
        };

        let max_radius = (fw * 0.55).max(fh * 0.55);
        let boost = self.relighting_intensity * 0.18;

        for y in 0..frame.height {
            let dy = (y as f32 - light_cy) / max_radius;
            let dy_sq = dy * dy;

            for x in 0..frame.width {
                let dx = (x as f32 - light_cx) / max_radius;
                let r_sq = (dx * dx + dy_sq).min(1.5);

                // Illumination falloff: slight boost at face center (+15%), gentle falloff towards corners (-8%)
                let factor = 1.0 + boost * (1.0 - r_sq * 1.4);

                let pixel = frame.get_pixel(x, y);
                let r = (pixel[0] as f32 * factor).round().clamp(0.0, 255.0) as u8;
                let g = (pixel[1] as f32 * factor).round().clamp(0.0, 255.0) as u8;
                let b = (pixel[2] as f32 * factor).round().clamp(0.0, 255.0) as u8;

                frame.set_pixel(x, y, [r, g, b]);
            }
        }
    }
}
