use crate::rtsp::frame::{DownscaledFrame, DOWNSCALE_HEIGHT, DOWNSCALE_WIDTH};

const C1: f32 = 6.5025;   // (0.01 * 255)^2
const C2: f32 = 58.5225;  // (0.03 * 255)^2
const BLOCK_SIZE: usize = 8;
const BLOCKS_X: usize = (DOWNSCALE_WIDTH as usize) / BLOCK_SIZE;   // 160 / 8 = 20
const BLOCKS_Y: usize = (DOWNSCALE_HEIGHT as usize) / BLOCK_SIZE;  // 120 / 8 = 15

/// Structural Dissimilarity (DSSIM) analyzer operating on downscaled grayscale frames.
/// Distinguishes real object entry (intruder, weapon) from simple lighting shifts.
pub struct StructuralDissimilarity {
    reference_frame: Option<Vec<u8>>,
    refresh_counter: usize,
    refresh_period: usize,
}

impl StructuralDissimilarity {
    /// Constructs a new structural dissimilarity analyzer.
    /// `refresh_period`: frames between reference background snapshot updates.
    pub fn new(refresh_period: usize) -> Self {
        Self {
            reference_frame: None,
            refresh_counter: 0,
            refresh_period,
        }
    }

    /// Sets or updates the reference baseline frame.
    pub fn set_reference(&mut self, frame: &DownscaledFrame) {
        self.reference_frame = Some(frame.gray_data.clone());
        self.refresh_counter = 0;
    }

    /// Computes Structural Dissimilarity (DSSIM in [0.0, 1.0]) between the current frame
    /// and the stored reference baseline.
    /// Returns 0.0 when identical, higher when a foreign structure enters the field of view.
    pub fn compute(&mut self, current: &DownscaledFrame) -> f32 {
        let reference = match &self.reference_frame {
            Some(ref_data) => ref_data,
            None => {
                self.reference_frame = Some(current.gray_data.clone());
                return 0.0;
            }
        };

        let cur_data = &current.gray_data;
        let mut ssim_sum = 0.0f32;
        let mut block_count = 0;

        for by in 0..BLOCKS_Y {
            let start_y = by * BLOCK_SIZE;
            for bx in 0..BLOCKS_X {
                let start_x = bx * BLOCK_SIZE;

                // Compute block statistics
                let mut sum_x = 0.0f32;
                let mut sum_y = 0.0f32;
                let mut sum_sq_x = 0.0f32;
                let mut sum_sq_y = 0.0f32;
                let mut sum_xy = 0.0f32;
                let n = (BLOCK_SIZE * BLOCK_SIZE) as f32;

                for dy in 0..BLOCK_SIZE {
                    let py = start_y + dy;
                    let row_offset = py * (DOWNSCALE_WIDTH as usize);
                    for dx in 0..BLOCK_SIZE {
                        let px = start_x + dx;
                        let idx = row_offset + px;

                        let val_x = reference[idx] as f32;
                        let val_y = cur_data[idx] as f32;

                        sum_x += val_x;
                        sum_y += val_y;
                        sum_sq_x += val_x * val_x;
                        sum_sq_y += val_y * val_y;
                        sum_xy += val_x * val_y;
                    }
                }

                let mu_x = sum_x / n;
                let mu_y = sum_y / n;
                let sigma_sq_x = (sum_sq_x / n) - (mu_x * mu_x);
                let sigma_sq_y = (sum_sq_y / n) - (mu_y * mu_y);
                let sigma_xy = (sum_xy / n) - (mu_x * mu_y);

                let numerator = (2.0 * mu_x * mu_y + C1) * (2.0 * sigma_xy + C2);
                let denominator = (mu_x * mu_x + mu_y * mu_y + C1) * (sigma_sq_x + sigma_sq_y + C2);

                let block_ssim = if denominator.abs() > 1e-6 {
                    (numerator / denominator).clamp(0.0, 1.0)
                } else {
                    1.0
                };

                ssim_sum += block_ssim;
                block_count += 1;
            }
        }

        let mean_ssim = if block_count > 0 {
            ssim_sum / (block_count as f32)
        } else {
            1.0
        };

        // Periodic reference refresh
        self.refresh_counter += 1;
        if self.refresh_counter >= self.refresh_period {
            self.reference_frame = Some(cur_data.clone());
            self.refresh_counter = 0;
        }

        // DSSIM = (1 - SSIM) / 2
        ((1.0 - mean_ssim) / 2.0).clamp(0.0, 1.0)
    }
}
