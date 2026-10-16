//! Camera Stream Source Abstraction
//! Supports both live RTSP feeds and local video files for WSL/Linux movie and CCTV simulation.

use image::RgbImage;

/// Decoded RGB Video Frame passed to Display and AI Sentinel
#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub camera_id: String,
    pub width: u32,
    pub height: u32,
    pub timestamp_ms: u64,
    pub image: RgbImage,
}

/// Simulated scenario type for offline validation in WSL
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationScenario {
    /// Room remains calm and empty (tests false alarm suppression and low CPU idling)
    QuietRoom,
    /// Room starts calm, then masked intruder breaks in at frame 50
    ShutterBreakIn,
    /// Authorized delivery courier leaves a parcel during daytime
    PackageDelivery,
}

/// Generator for simulated test feeds (WSL movie/CCTV validation)
pub struct SimulatedCameraFeed {
    pub camera_id: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub scenario: SimulationScenario,
    frame_counter: u64,
}

impl SimulatedCameraFeed {
    pub fn new(camera_id: impl Into<String>, scenario: SimulationScenario) -> Self {
        Self {
            camera_id: camera_id.into(),
            width: 640,
            height: 360,
            fps: 25,
            scenario,
            frame_counter: 0,
        }
    }

    /// Generate the next video frame simulating camera capture
    pub fn next_frame(&mut self) -> VideoFrame {
        self.frame_counter += 1;
        let elapsed_ms = (self.frame_counter * 1000) / (self.fps as u64);

        let mut img = RgbImage::new(self.width, self.height);

        // Generate baseline background (e.g. dark bank/store floor)
        for pixel in img.pixels_mut() {
            *pixel = image::Rgb([35, 38, 42]);
        }

        // Apply scenario-specific patterns
        match self.scenario {
            SimulationScenario::QuietRoom => {
                // Subtle static sensor noise only
                let noise = (self.frame_counter % 3) as u8;
                for pixel in img.pixels_mut() {
                    pixel[0] = pixel[0].saturating_add(noise);
                }
            }
            SimulationScenario::ShutterBreakIn => {
                // Calm before frame 40, then sudden intruder breach
                if self.frame_counter > 40 {
                    let intruder_x = 280 + ((self.frame_counter - 40) * 3).min(100) as u32;
                    let intruder_y = 100;
                    // Render moving intruder silhouette
                    for y in intruder_y..(intruder_y + 120).min(self.height) {
                        for x in intruder_x..(intruder_x + 60).min(self.width) {
                            img.put_pixel(x, y, image::Rgb([220, 20, 30]));
                        }
                    }
                }
            }
            SimulationScenario::PackageDelivery => {
                // Person walks in from frame 30 to 80 and places box
                if self.frame_counter > 30 && self.frame_counter < 90 {
                    let courier_x = 200 + ((self.frame_counter - 30) * 2) as u32;
                    for y in 150..230 {
                        for x in courier_x..(courier_x + 40).min(self.width) {
                            img.put_pixel(x, y, image::Rgb([40, 160, 220]));
                        }
                    }
                }
            }
        }

        VideoFrame {
            camera_id: self.camera_id.clone(),
            width: self.width,
            height: self.height,
            timestamp_ms: elapsed_ms,
            image: img,
        }
    }
}
