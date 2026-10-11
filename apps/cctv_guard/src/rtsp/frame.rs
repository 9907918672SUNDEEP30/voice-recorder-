use anyhow::{anyhow, Result};
use image::{ImageBuffer, Rgb};
use ndarray::Array4;
use std::io::Cursor;

/// Target dimensions for ultra-fast Tier-1 edge motion and structural similarity gating.
pub const DOWNSCALE_WIDTH: u32 = 160;
pub const DOWNSCALE_HEIGHT: u32 = 120;
pub const DOWNSCALE_SIZE: usize = (DOWNSCALE_WIDTH * DOWNSCALE_HEIGHT) as usize;

/// High-resolution CCTV video frame consumed from the RTSP stream.
#[derive(Debug, Clone)]
pub struct CctvFrame {
    pub width: u32,
    pub height: u32,
    /// Interleaved 24-bit RGB pixel data: [R0, G0, B0, R1, G1, B1, ...]
    pub rgb_data: Vec<u8>,
    pub timestamp_ms: u64,
    pub frame_idx: u64,
}

/// Lightweight 160x120 single-channel 8-bit grayscale buffer for low-overhead (< 5% CPU) Tier-1 analysis.
#[derive(Debug, Clone)]
pub struct DownscaledFrame {
    pub width: u32,
    pub height: u32,
    /// Grayscale luminance values: [Y0, Y1, ..., Y19199]
    pub gray_data: Vec<u8>,
    pub timestamp_ms: u64,
    pub frame_idx: u64,
}

impl CctvFrame {
    /// Constructs a new CCTV frame with raw interleaved RGB bytes.
    pub fn new(width: u32, height: u32, rgb_data: Vec<u8>, timestamp_ms: u64, frame_idx: u64) -> Result<Self> {
        let expected_len = (width * height * 3) as usize;
        if rgb_data.len() != expected_len {
            return Err(anyhow!(
                "RGB buffer length mismatch: expected {} bytes ({}x{}x3), got {}",
                expected_len,
                width,
                height,
                rgb_data.len()
            ));
        }
        Ok(Self {
            width,
            height,
            rgb_data,
            timestamp_ms,
            frame_idx,
        })
    }

    /// Creates a synthetic test frame filled with a specific color or pattern (used in tests and simulation).
    pub fn synthetic_pattern(width: u32, height: u32, r: u8, g: u8, b: u8, timestamp_ms: u64, frame_idx: u64) -> Self {
        let mut rgb = Vec::with_capacity((width * height * 3) as usize);
        for _ in 0..(width * height) {
            rgb.push(r);
            rgb.push(g);
            rgb.push(b);
        }
        Self {
            width,
            height,
            rgb_data: rgb,
            timestamp_ms,
            frame_idx,
        }
    }

    /// Downscales the high-resolution RGB frame directly into a 160x120 grayscale buffer.
    /// Uses integer fixed-point arithmetic for the standard Rec.601 luma conversion:
    /// Y = (299*R + 587*G + 114*B) / 1000.
    pub fn downscale_to_160x120(&self) -> DownscaledFrame {
        let mut gray = vec![0u8; DOWNSCALE_SIZE];
        let x_ratio = ((self.width as u64) << 16) / DOWNSCALE_WIDTH as u64;
        let y_ratio = ((self.height as u64) << 16) / DOWNSCALE_HEIGHT as u64;

        for dy in 0..DOWNSCALE_HEIGHT {
            let sy = ((dy as u64 * y_ratio) >> 16) as u32;
            let src_row_offset = (sy * self.width * 3) as usize;
            let dst_row_offset = (dy * DOWNSCALE_WIDTH) as usize;

            for dx in 0..DOWNSCALE_WIDTH {
                let sx = ((dx as u64 * x_ratio) >> 16) as u32;
                let src_idx = src_row_offset + (sx * 3) as usize;

                if src_idx + 2 < self.rgb_data.len() {
                    let r = self.rgb_data[src_idx] as u32;
                    let g = self.rgb_data[src_idx + 1] as u32;
                    let b = self.rgb_data[src_idx + 2] as u32;
                    let luma = (299 * r + 587 * g + 114 * b + 500) / 1000;
                    gray[dst_row_offset + dx as usize] = luma as u8;
                }
            }
        }

        DownscaledFrame {
            width: DOWNSCALE_WIDTH,
            height: DOWNSCALE_HEIGHT,
            gray_data: gray,
            timestamp_ms: self.timestamp_ms,
            frame_idx: self.frame_idx,
        }
    }

    /// Compresses the frame into a JPEG byte vector for disk storage and evidence preservation.
    pub fn to_jpeg(&self, quality: u8) -> Result<Vec<u8>> {
        let img_buf: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::from_raw(self.width, self.height, self.rgb_data.clone())
                .ok_or_else(|| anyhow!("Failed to construct ImageBuffer from raw RGB slice"))?;

        let mut jpeg_bytes = Vec::new();
        let mut cursor = Cursor::new(&mut jpeg_bytes);
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, quality);
        encoder.encode_image(&img_buf)?;

        Ok(jpeg_bytes)
    }

    /// Converts the high-resolution frame into a normalized 4D float tensor `[1, 3, target_h, target_w]`
    /// compatible with Qwen2-VL ONNX vision encoders.
    /// Applies ImageNet channel normalization:
    /// Mean: [0.48145466, 0.4578275, 0.40821073], Std: [0.26862954, 0.26130258, 0.27577711].
    pub fn to_onnx_tensor(&self, target_w: usize, target_h: usize) -> Result<Array4<f32>> {
        let mean = [0.48145466f32, 0.4578275f32, 0.40821073f32];
        let std = [0.26862954f32, 0.26130258f32, 0.27577711f32];

        let mut tensor = Array4::<f32>::zeros((1, 3, target_h, target_w));

        let x_scale = self.width as f32 / target_w as f32;
        let y_scale = self.height as f32 / target_h as f32;

        for th in 0..target_h {
            let sh = ((th as f32 * y_scale) as u32).min(self.height - 1);
            let row_offset = (sh * self.width * 3) as usize;

            for tw in 0..target_w {
                let sw = ((tw as f32 * x_scale) as u32).min(self.width - 1);
                let px_idx = row_offset + (sw * 3) as usize;

                let r_norm = ((self.rgb_data[px_idx] as f32 / 255.0) - mean[0]) / std[0];
                let g_norm = ((self.rgb_data[px_idx + 1] as f32 / 255.0) - mean[1]) / std[1];
                let b_norm = ((self.rgb_data[px_idx + 2] as f32 / 255.0) - mean[2]) / std[2];

                tensor[[0, 0, th, tw]] = r_norm;
                tensor[[0, 1, th, tw]] = g_norm;
                tensor[[0, 2, th, tw]] = b_norm;
            }
        }

        Ok(tensor)
    }
}
