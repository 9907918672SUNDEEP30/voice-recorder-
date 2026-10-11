use crate::capture::VideoFrame;
use crate::tracking::face_detector::FaceLandmarks;

/// Bounding region and tracking state for a single eye socket.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EyeRegion {
    pub center_x: f32,
    pub center_y: f32,
    pub radius_x: f32,
    pub radius_y: f32,
    pub iris_center_x: f32,
    pub iris_center_y: f32,
    pub is_open: bool,
}

/// Iris and pupil tracking engine extracting eye positions from video frames.
pub struct IrisTracker;

impl IrisTracker {
    /// Extracts Left and Right eye socket regions and current pupil positions.
    pub fn locate_eyes(frame: &VideoFrame, landmarks: &FaceLandmarks) -> (EyeRegion, EyeRegion) {
        let fw = frame.width as f32;
        let fh = frame.height as f32;

        let left_eye_center = (landmarks.left_eye.0 * fw, landmarks.left_eye.1 * fh);
        let right_eye_center = (landmarks.right_eye.0 * fw, landmarks.right_eye.1 * fh);

        let eye_distance = ((right_eye_center.0 - left_eye_center.0).powi(2)
            + (right_eye_center.1 - left_eye_center.1).powi(2))
        .sqrt();

        let eye_rx = (eye_distance * 0.35).max(12.0);
        let eye_ry = (eye_distance * 0.25).max(8.0);

        let left_iris = Self::find_pupil_locus(frame, left_eye_center.0, left_eye_center.1, eye_rx, eye_ry);
        let right_iris = Self::find_pupil_locus(frame, right_eye_center.0, right_eye_center.1, eye_rx, eye_ry);

        let left = EyeRegion {
            center_x: left_eye_center.0,
            center_y: left_eye_center.1,
            radius_x: eye_rx,
            radius_y: eye_ry,
            iris_center_x: left_iris.0,
            iris_center_y: left_iris.1,
            is_open: left_iris.2,
        };

        let right = EyeRegion {
            center_x: right_eye_center.0,
            center_y: right_eye_center.1,
            radius_x: eye_rx,
            radius_y: eye_ry,
            iris_center_x: right_iris.0,
            iris_center_y: right_iris.1,
            is_open: right_iris.2,
        };

        (left, right)
    }

    /// Finds the darkest pupil intensity centroid within an eye bounding ellipse.
    fn find_pupil_locus(
        frame: &VideoFrame,
        cx: f32,
        cy: f32,
        rx: f32,
        ry: f32,
    ) -> (f32, f32, bool) {
        let min_x = (cx - rx).max(0.0) as u32;
        let max_x = (cx + rx).min((frame.width - 1) as f32) as u32;
        let min_y = (cy - ry).max(0.0) as u32;
        let max_y = (cy + ry).min((frame.height - 1) as f32) as u32;

        let mut min_lum = 255u8;
        let mut min_lum_x = cx;
        let mut min_lum_y = cy;
        let mut total_dark_pixels = 0;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let dx = (x as f32 - cx) / rx;
                let dy = (y as f32 - cy) / ry;
                if dx * dx + dy * dy <= 1.0 {
                    let [r, g, b] = frame.get_pixel(x, y);
                    // Standard luminance formula
                    let lum = ((r as u32 * 299 + g as u32 * 587 + b as u32 * 114) / 1000) as u8;
                    if lum < min_lum {
                        min_lum = lum;
                        min_lum_x = x as f32;
                        min_lum_y = y as f32;
                    }
                    if lum < 120 {
                        total_dark_pixels += 1;
                    }
                }
            }
        }

        // If dark pixels or lower luminance found, the eye is open
        let is_open = min_lum < 180 || total_dark_pixels >= 1;
        (min_lum_x, min_lum_y, is_open)
    }
}
