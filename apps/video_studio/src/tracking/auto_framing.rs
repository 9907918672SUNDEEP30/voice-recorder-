use crate::capture::VideoFrame;
use crate::tracking::face_detector::FaceBox;

/// Configuration for the AI Cameraman Auto-Framing Engine.
#[derive(Debug, Clone)]
pub struct AutoFramingConfig {
    /// Deadzone threshold in normalized units (e.g. 0.04 = 4% screen width).
    /// Prevents micro-jitter from head nods, natural breathing, or fidgeting.
    pub deadzone_threshold: f32,
    /// Motion damping smoothing factor alpha [0.01 - 0.20].
    /// Lower values yield smoother, more cinematic pan/tilt motions.
    pub smoothing_alpha: f32,
    /// Headroom margin above the head (e.g. 0.15 = 15% frame padding).
    pub headroom_factor: f32,
    /// Digital zoom multiplier factor [1.0 = wide lens, 1.4 = medium close-up, 1.8 = tight close-up].
    pub zoom_multiplier: f32,
    /// Target aspect ratio (default: 16.0 / 9.0).
    pub target_aspect_ratio: f32,
}

impl Default for AutoFramingConfig {
    fn default() -> Self {
        Self {
            deadzone_threshold: 0.035,
            smoothing_alpha: 0.08,
            headroom_factor: 0.18,
            zoom_multiplier: 1.35,
            target_aspect_ratio: 16.0 / 9.0,
        }
    }
}

/// Normalized viewport crop window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CropWindow {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for CropWindow {
    fn default() -> Self {
        Self {
            left: 0.0,
            top: 0.0,
            width: 1.0,
            height: 1.0,
        }
    }
}

/// AI Cameraman Engine implementing dynamic Pan-Tilt-Zoom (PTZ) with motion damping.
pub struct AutoFramer {
    config: AutoFramingConfig,
    current_window: CropWindow,
    target_window: CropWindow,
    last_face_center: Option<(f32, f32)>,
    is_active: bool,
}

impl AutoFramer {
    /// Creates a new AutoFramer with initial full-frame view.
    pub fn new(config: AutoFramingConfig) -> Self {
        Self {
            config,
            current_window: CropWindow::default(),
            target_window: CropWindow::default(),
            last_face_center: None,
            is_active: true,
        }
    }

    /// Toggles active tracking on or off.
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
        if !active {
            self.target_window = CropWindow::default();
        }
    }

    /// Updates tracking based on detected face bounding box and calculates smoothed crop box.
    pub fn update(&mut self, face_opt: Option<&FaceBox>) -> CropWindow {
        if !self.is_active {
            self.current_window = CropWindow::default();
            return self.current_window;
        }

        if let Some(face) = face_opt {
            let (face_cx, face_cy) = face.center();

            // Evaluate deadzone threshold against previous stable center
            let should_update = match self.last_face_center {
                Some((last_cx, last_cy)) => {
                    let dist = ((face_cx - last_cx).powi(2) + (face_cy - last_cy).powi(2)).sqrt();
                    dist > self.config.deadzone_threshold
                }
                None => true,
            };

            if should_update {
                self.last_face_center = Some((face_cx, face_cy));

                // Calculate desired crop window dimensions based on face size & zoom multiplier
                let target_h = (face.height * 2.4 / self.config.zoom_multiplier).clamp(0.40, 1.0);
                let target_w = (target_h * (1.0 / self.config.target_aspect_ratio)).clamp(0.35, 1.0);

                // Center crop window on subject with headroom bias (eyes slightly above vertical center)
                let ideal_cy = face_cy - (target_h * self.config.headroom_factor);
                let target_top = (ideal_cy - target_h * 0.40).clamp(0.0, 1.0 - target_h);
                let target_left = (face_cx - target_w * 0.50).clamp(0.0, 1.0 - target_w);

                self.target_window = CropWindow {
                    left: target_left,
                    top: target_top,
                    width: target_w,
                    height: target_h,
                };
            }
        }

        // Apply Exponential Moving Average (EMA) damping to smoothly animate camera pan & zoom
        let alpha = self.config.smoothing_alpha;
        self.current_window.left += alpha * (self.target_window.left - self.current_window.left);
        self.current_window.top += alpha * (self.target_window.top - self.current_window.top);
        self.current_window.width += alpha * (self.target_window.width - self.current_window.width);
        self.current_window.height += alpha * (self.target_window.height - self.current_window.height);

        self.current_window
    }

    /// Performs pure Rust sub-pixel bilinear cropping and resampling to target output resolution.
    ///
    /// # Arguments
    /// * `src` - Input high-resolution raw camera frame
    /// * `out_w` - Target canvas width (e.g. 1920 or 1280)
    /// * `out_h` - Target canvas height (e.g. 1080 or 720)
    pub fn render_framed_output(
        &self,
        src: &VideoFrame,
        out_w: u32,
        out_h: u32,
    ) -> VideoFrame {
        let mut dst = VideoFrame::new(out_w, out_h);
        dst.timestamp_ms = src.timestamp_ms;

        let crop = self.current_window;
        let src_w = src.width as f32;
        let src_h = src.height as f32;

        let crop_x0 = crop.left * src_w;
        let crop_y0 = crop.top * src_h;
        let crop_w = (crop.width * src_w).max(10.0);
        let crop_h = (crop.height * src_h).max(10.0);

        let scale_x = crop_w / out_w as f32;
        let scale_y = crop_h / out_h as f32;

        let max_src_x = (src.width - 2) as f32;
        let max_src_y = (src.height - 2) as f32;

        // Bilinear sub-pixel sampling loop
        for dy in 0..out_h {
            let sy_f = (crop_y0 + dy as f32 * scale_y).clamp(0.0, max_src_y);
            let sy0 = sy_f as u32;
            let sy1 = sy0 + 1;
            let ty = sy_f - sy0 as f32;

            for dx in 0..out_w {
                let sx_f = (crop_x0 + dx as f32 * scale_x).clamp(0.0, max_src_x);
                let sx0 = sx_f as u32;
                let sx1 = sx0 + 1;
                let tx = sx_f - sx0 as f32;

                // Sample 4 neighbor pixels
                let p00 = src.get_pixel(sx0, sy0);
                let p10 = src.get_pixel(sx1, sy0);
                let p01 = src.get_pixel(sx0, sy1);
                let p11 = src.get_pixel(sx1, sy1);

                // Bilinear interpolation per channel
                let mut out_rgb = [0u8; 3];
                for c in 0..3 {
                    let top = p00[c] as f32 * (1.0 - tx) + p10[c] as f32 * tx;
                    let bottom = p01[c] as f32 * (1.0 - tx) + p11[c] as f32 * tx;
                    let val = top * (1.0 - ty) + bottom * ty;
                    out_rgb[c] = val.round().clamp(0.0, 255.0) as u8;
                }

                dst.set_pixel(dx, dy, out_rgb);
            }
        }

        dst
    }
}
