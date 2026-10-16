//! Direct HDMI On-Screen Display (OSD / HUD)
//! Renders camera names, live clock, recording indicators, and AI alert overlays.

use super::grid::CameraViewport;
use chrono::Utc;

pub struct HudRenderer;

impl HudRenderer {
    /// Format OSD text header for a camera tile
    pub fn format_tile_header(viewport: &CameraViewport, fps: u32, is_recording: bool) -> String {
        let rec_tag = if is_recording { "● REC" } else { "IDLE" };
        let alert_tag = if viewport.is_alert_active { " ⚠️ ALERT" } else { "" };
        let time_str = Utc::now().format("%H:%M:%S");

        format!(
            "[{}] {}fps | {} | {}{}",
            viewport.camera_id, fps, rec_tag, time_str, alert_tag
        )
    }
}
