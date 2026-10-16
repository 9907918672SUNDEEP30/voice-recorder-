//! Multi-Camera Monitor Grid Compositor
//! Computes matrix layouts for 4, 8, 16, and 32 camera channels driving direct HDMI output.

use serde::{Deserialize, Serialize};

/// Viewport rectangle for a camera on the physical HDMI monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraViewport {
    pub camera_id: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub is_alert_active: bool,
    pub border_color_rgb: [u8; 3],
}

pub struct DisplayGridCompositor {
    pub screen_width: u32,
    pub screen_height: u32,
}

impl DisplayGridCompositor {
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        Self {
            screen_width,
            screen_height,
        }
    }

    /// Calculate viewport tiles for N camera streams
    pub fn compute_layout(&self, camera_ids: &[String], active_alert_camera: Option<&str>) -> Vec<CameraViewport> {
        let count = camera_ids.len();
        let (cols, rows) = match count {
            0..=4 => (2, 2),
            5..=9 => (3, 3),
            10..=16 => (4, 4),
            _ => (6, 6),
        };

        let tile_width = self.screen_width / cols;
        let tile_height = self.screen_height / rows;

        let mut viewports = Vec::with_capacity(count);

        for (idx, cam_id) in camera_ids.iter().enumerate() {
            let col = (idx as u32) % cols;
            let row = (idx as u32) / cols;

            let is_alert = active_alert_camera == Some(cam_id.as_str());
            let border_color_rgb = if is_alert {
                [255, 20, 20] // Emergency Red flashing border
            } else {
                [40, 45, 52]  // Clean industrial slate border
            };

            viewports.push(CameraViewport {
                camera_id: cam_id.clone(),
                x: col * tile_width,
                y: row * tile_height,
                width: tile_width,
                height: tile_height,
                is_alert_active: is_alert,
                border_color_rgb,
            });
        }

        viewports
    }
}
