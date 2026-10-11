use anyhow::{anyhow, Result};
use image::{ImageBuffer, Rgb};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType, Resolution};
use nokhwa::Camera;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Raw RGB video frame buffer.
#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    /// Interleaved RGB data: [R0, G0, B0, R1, G1, B1, ...]
    pub data: Vec<u8>,
    pub timestamp_ms: u64,
}

impl VideoFrame {
    /// Allocates an empty video frame of specified dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 3) as usize;
        Self {
            width,
            height,
            data: vec![0u8; size],
            timestamp_ms: 0,
        }
    }

    /// Creates a video frame from existing byte slice.
    pub fn from_rgb_slice(width: u32, height: u32, rgb_bytes: &[u8], timestamp_ms: u64) -> Result<Self> {
        let expected = (width * height * 3) as usize;
        if rgb_bytes.len() != expected {
            return Err(anyhow!(
                "Invalid RGB buffer size: expected {} bytes, got {}",
                expected,
                rgb_bytes.len()
            ));
        }
        Ok(Self {
            width,
            height,
            data: rgb_bytes.to_vec(),
            timestamp_ms,
        })
    }

    /// Converts to an ImageBuffer for utility operations.
    pub fn to_image_buffer(&self) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        ImageBuffer::from_raw(self.width, self.height, self.data.clone())
            .unwrap_or_else(|| ImageBuffer::new(self.width, self.height))
    }

    /// Computes pixel index in the interleaved RGB slice.
    #[inline(always)]
    pub fn pixel_offset(&self, x: u32, y: u32) -> usize {
        ((y * self.width + x) * 3) as usize
    }

    /// Retrieves an RGB pixel at coordinate (x, y).
    #[inline(always)]
    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 3] {
        let idx = self.pixel_offset(x, y);
        [self.data[idx], self.data[idx + 1], self.data[idx + 2]]
    }

    /// Writes an RGB pixel at coordinate (x, y).
    #[inline(always)]
    pub fn set_pixel(&mut self, x: u32, y: u32, rgb: [u8; 3]) {
        let idx = self.pixel_offset(x, y);
        self.data[idx] = rgb[0];
        self.data[idx + 1] = rgb[1];
        self.data[idx + 2] = rgb[2];
    }
}

/// Abstract video camera source trait supporting both physical webcams and virtual test generators.
pub trait CameraSource: Send {
    /// Starts the camera feed.
    fn start(&mut self) -> Result<()>;
    /// Captures the next raw RGB video frame.
    fn capture_frame(&mut self) -> Result<VideoFrame>;
    /// Stops the camera feed.
    fn stop(&mut self) -> Result<()>;
    /// Returns camera native width and height.
    fn resolution(&self) -> (u32, u32);
    /// Is the camera actively capturing.
    fn is_active(&self) -> bool;
}

/// Physical USB or built-in webcam capture backed by nokhwa.
pub struct NokhwaCamera {
    camera: Option<Camera>,
    width: u32,
    height: u32,
    index: u32,
    is_running: bool,
    start_time: Instant,
}

// Windows Media Foundation pointers are safe to transfer across threads with &mut exclusivity
unsafe impl Send for NokhwaCamera {}

impl NokhwaCamera {
    /// Queries available system camera devices.
    pub fn query_devices() -> Result<Vec<String>> {
        let devices = nokhwa::query(nokhwa::utils::ApiBackend::Auto)?;
        let mut list = Vec::new();
        for dev in devices {
            list.push(format!("[{}] {}", dev.index(), dev.human_name()));
        }
        Ok(list)
    }

    /// Initializes a physical webcam with target resolution.
    pub fn new(index: u32, width: u32, height: u32) -> Result<Self> {
        Ok(Self {
            camera: None,
            width,
            height,
            index,
            is_running: false,
            start_time: Instant::now(),
        })
    }
}

impl CameraSource for NokhwaCamera {
    fn start(&mut self) -> Result<()> {
        let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::HighestResolution(
            Resolution::new(self.width, self.height),
        ));

        let mut camera = Camera::new(CameraIndex::Index(self.index), format)
            .map_err(|e| anyhow!("Failed to initialize camera {}: {}", self.index, e))?;

        camera
            .open_stream()
            .map_err(|e| anyhow!("Failed to open camera stream: {}", e))?;

        self.camera = Some(camera);
        self.is_running = true;
        self.start_time = Instant::now();
        Ok(())
    }

    fn capture_frame(&mut self) -> Result<VideoFrame> {
        let camera = self
            .camera
            .as_mut()
            .ok_or_else(|| anyhow!("Camera is not started"))?;

        let frame = camera
            .frame()
            .map_err(|e| anyhow!("Failed to capture camera frame: {}", e))?;

        let decoded = frame
            .decode_image::<RgbFormat>()
            .map_err(|e| anyhow!("Failed to decode frame as RGB: {}", e))?;

        let timestamp_ms = self.start_time.elapsed().as_millis() as u64;
        VideoFrame::from_rgb_slice(self.width, self.height, decoded.as_raw(), timestamp_ms)
    }

    fn stop(&mut self) -> Result<()> {
        if let Some(mut camera) = self.camera.take() {
            let _ = camera.stop_stream();
        }
        self.is_running = false;
        Ok(())
    }

    fn resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn is_active(&self) -> bool {
        self.is_running
    }
}

/// Virtual synthetic video frame generator.
///
/// Generates realistic 1080p studio test frames with a moving virtual subject/face.
/// Enables deterministic unit testing and CI verification without requiring physical hardware.
pub struct VirtualCamera {
    width: u32,
    height: u32,
    is_running: Arc<AtomicBool>,
    start_time: Instant,
    frame_counter: u64,
}

unsafe impl Send for VirtualCamera {}

impl VirtualCamera {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            is_running: Arc::new(AtomicBool::new(false)),
            start_time: Instant::now(),
            frame_counter: 0,
        }
    }
}

impl CameraSource for VirtualCamera {
    fn start(&mut self) -> Result<()> {
        self.is_running.store(true, Ordering::SeqCst);
        self.start_time = Instant::now();
        self.frame_counter = 0;
        Ok(())
    }

    fn capture_frame(&mut self) -> Result<VideoFrame> {
        if !self.is_running.load(Ordering::Relaxed) {
            return Err(anyhow!("Virtual camera is stopped"));
        }

        self.frame_counter += 1;
        let mut frame = VideoFrame::new(self.width, self.height);
        let time_sec = self.start_time.elapsed().as_secs_f32();

        // Simulate studio background with subtle vertical vignette
        for y in 0..self.height {
            let norm_y = y as f32 / self.height as f32;
            let bg_r = (25.0 + norm_y * 15.0) as u8;
            let bg_g = (30.0 + norm_y * 20.0) as u8;
            let bg_b = (45.0 + norm_y * 30.0) as u8;

            for x in 0..self.width {
                frame.set_pixel(x, y, [bg_r, bg_g, bg_b]);
            }
        }

        // Simulate a moving subject/face (smooth sine motion to test AI Cameraman tracking)
        let center_x = (self.width as f32 * 0.5) + (time_sec * 0.8).sin() * (self.width as f32 * 0.15);
        let center_y = (self.height as f32 * 0.45) + (time_sec * 0.4).cos() * (self.height as f32 * 0.05);
        let face_radius_x = self.width as f32 * 0.12;
        let face_radius_y = self.height as f32 * 0.18;

        // Render synthetic face oval with skin tone
        let min_x = (center_x - face_radius_x).max(0.0) as u32;
        let max_x = (center_x + face_radius_x).min((self.width - 1) as f32) as u32;
        let min_y = (center_y - face_radius_y).max(0.0) as u32;
        let max_y = (center_y + face_radius_y).min((self.height - 1) as f32) as u32;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let dx = (x as f32 - center_x) / face_radius_x;
                let dy = (y as f32 - center_y) / face_radius_y;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq <= 1.0 {
                    // Base natural skin tone (RGB: 228, 185, 155) with 3D gradient shading
                    let shade = 1.0 - dist_sq * 0.25;
                    let r = ((228.0 * shade) as u8).min(255);
                    let g = ((185.0 * shade) as u8).min(255);
                    let b = ((155.0 * shade) as u8).min(255);
                    frame.set_pixel(x, y, [r, g, b]);
                }
            }
        }

        // Render synthetic eyes on the face (to test Eye Contact & Gaze Warping)
        let left_eye_x = center_x - face_radius_x * 0.35;
        let right_eye_x = center_x + face_radius_x * 0.35;
        let eye_y = center_y - face_radius_y * 0.15;
        let eye_radius = face_radius_x * 0.10;

        for &(eye_cx, eye_cy) in &[(left_eye_x, eye_y), (right_eye_x, eye_y)] {
            let emx = (eye_cx - eye_radius).max(0.0) as u32;
            let emx_max = (eye_cx + eye_radius).min((self.width - 1) as f32) as u32;
            let emy = (eye_cy - eye_radius * 0.7).max(0.0) as u32;
            let emy_max = (eye_cy + eye_radius * 0.7).min((self.height - 1) as f32) as u32;

            for y in emy..=emy_max {
                for x in emx..=emx_max {
                    let edx = (x as f32 - eye_cx) / eye_radius;
                    let edy = (y as f32 - eye_cy) / (eye_radius * 0.7);
                    if edx * edx + edy * edy <= 1.0 {
                        // Sclera (white of the eye)
                        frame.set_pixel(x, y, [245, 245, 250]);
                    }
                    // Pupil (looking slightly down by default to simulate reading a teleprompter)
                    let pupil_offset_y = eye_radius * 0.28;
                    let pdx = (x as f32 - eye_cx) / (eye_radius * 0.45);
                    let pdy = (y as f32 - (eye_cy + pupil_offset_y)) / (eye_radius * 0.45);
                    if pdx * pdx + pdy * pdy <= 1.0 {
                        frame.set_pixel(x, y, [30, 20, 15]); // Dark iris
                    }
                }
            }
        }

        frame.timestamp_ms = self.start_time.elapsed().as_millis() as u64;
        Ok(frame)
    }

    fn stop(&mut self) -> Result<()> {
        self.is_running.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn is_active(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }
}
