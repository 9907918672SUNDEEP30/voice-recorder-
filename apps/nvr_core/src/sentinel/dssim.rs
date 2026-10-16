//! 8x8 Block Structural Dissimilarity (DSSIM) Calculator
//! DSSIM = (1 - SSIM) / 2
//! Gauges structural geometric shifts (intruders, moving vehicles, forced entry) while rejecting lighting shifts.

use super::luma::{LumaGrid, TARGET_HEIGHT, TARGET_WIDTH};

pub struct StructuralComparator;

impl StructuralComparator {
    /// Calculate mean DSSIM difference between two 160x120 Luma grids using 8x8 sliding blocks
    pub fn compute_dssim(a: &LumaGrid, b: &LumaGrid) -> f32 {
        const BLOCK_SIZE: usize = 8;
        const C1: f32 = 6.5025;  // (0.01 * 255)^2
        const C2: f32 = 58.5225; // (0.03 * 255)^2

        let blocks_x = TARGET_WIDTH / BLOCK_SIZE;
        let blocks_y = TARGET_HEIGHT / BLOCK_SIZE;
        let total_blocks = (blocks_x * blocks_y) as f32;

        let mut total_ssim = 0.0f32;

        for by in 0..blocks_y {
            for bx in 0..blocks_x {
                let start_x = bx * BLOCK_SIZE;
                let start_y = by * BLOCK_SIZE;

                let mut sum_a = 0.0f32;
                let mut sum_b = 0.0f32;
                let mut sum_sq_a = 0.0f32;
                let mut sum_sq_b = 0.0f32;
                let mut sum_ab = 0.0f32;

                for y in 0..BLOCK_SIZE {
                    for x in 0..BLOCK_SIZE {
                        let idx = (start_y + y) * TARGET_WIDTH + (start_x + x);
                        let val_a = a.pixels[idx] as f32;
                        let val_b = b.pixels[idx] as f32;

                        sum_a += val_a;
                        sum_b += val_b;
                        sum_sq_a += val_a * val_a;
                        sum_sq_b += val_b * val_b;
                        sum_ab += val_a * val_b;
                    }
                }

                let n = (BLOCK_SIZE * BLOCK_SIZE) as f32;
                let mean_a = sum_a / n;
                let mean_b = sum_b / n;

                let var_a = (sum_sq_a / n) - (mean_a * mean_a);
                let var_b = (sum_sq_b / n) - (mean_b * mean_b);
                let cov_ab = (sum_ab / n) - (mean_a * mean_b);

                // Standard SSIM formula for 8x8 block
                let numerator = (2.0 * mean_a * mean_b + C1) * (2.0 * cov_ab + C2);
                let denominator = (mean_a * mean_a + mean_b * mean_b + C1) * (var_a + var_b + C2);
                let ssim = (numerator / denominator.max(0.0001)).clamp(-1.0, 1.0);

                total_ssim += ssim;
            }
        }

        let mean_ssim = total_ssim / total_blocks;
        // Dissimilarity: 0.0 = Identical, 1.0 = Completely Dissimilar
        ((1.0 - mean_ssim) / 2.0).clamp(0.0, 1.0)
    }
}
