use crate::capture::VideoFrame;

/// Predefined studio and cinema 3D LUT presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LutPreset {
    TealAndOrange,
    WarmStudio,
    CleanMinimal,
    FilmPunch,
    None,
}

/// 3D Look-Up Table for cinematic color grading.
/// Contains a N x N x N discrete sampling of the 3D RGB color space with Trilinear Interpolation.
pub struct Lut3D {
    size: usize,
    /// Flat array of [R, G, B] floats in range [0.0, 1.0], indexed by [r * size * size + g * size + b]
    table: Vec<[f32; 3]>,
}

impl Lut3D {
    /// Generates a standard identity LUT of dimension N (typically 17 or 33).
    pub fn new_identity(size: usize) -> Self {
        let total = size * size * size;
        let mut table = Vec::with_capacity(total);
        let step = 1.0 / (size - 1) as f32;

        for r_idx in 0..size {
            for g_idx in 0..size {
                for b_idx in 0..size {
                    table.push([r_idx as f32 * step, g_idx as f32 * step, b_idx as f32 * step]);
                }
            }
        }

        Self { size, table }
    }

    /// Constructs a 3D LUT tailored to a specific cinematic preset.
    pub fn from_preset(preset: LutPreset) -> Self {
        let size = 17usize;
        let mut lut = Self::new_identity(size);

        match preset {
            LutPreset::TealAndOrange => {
                // Hollywood Teal & Orange: Cool cyan/teal shadows, warm golden skin tones
                for entry in lut.table.iter_mut() {
                    let r = entry[0];
                    let g = entry[1];
                    let b = entry[2];

                    let lum = 0.299 * r + 0.587 * g + 0.114 * b;

                    // Push shadows towards teal (reduce red, increase blue and green slightly)
                    let shadow_weight = (1.0 - lum).powi(2);
                    let highlight_weight = lum.powi(2);

                    let new_r = (r * (1.0 - shadow_weight * 0.25) + highlight_weight * 0.12).clamp(0.0, 1.0);
                    let new_g = (g * (1.0 + shadow_weight * 0.05)).clamp(0.0, 1.0);
                    let new_b = (b * (1.0 + shadow_weight * 0.35) - highlight_weight * 0.10).clamp(0.0, 1.0);

                    // S-curve contrast boost
                    let s_curve = |v: f32| -> f32 { (v * v * (3.0 - 2.0 * v)).clamp(0.0, 1.0) };
                    *entry = [s_curve(new_r), s_curve(new_g), s_curve(new_b)];
                }
            }
            LutPreset::WarmStudio => {
                // Warm, flattering podcast skin tone lift
                for entry in lut.table.iter_mut() {
                    let r = entry[0];
                    let g = entry[1];
                    let b = entry[2];

                    let new_r = (r * 1.08 + 0.02).clamp(0.0, 1.0);
                    let new_g = (g * 1.03 + 0.01).clamp(0.0, 1.0);
                    let new_b = (b * 0.94).clamp(0.0, 1.0);

                    *entry = [new_r, new_g, new_b];
                }
            }
            LutPreset::CleanMinimal => {
                // Punchy high-contrast neutral look
                for entry in lut.table.iter_mut() {
                    let r = entry[0];
                    let g = entry[1];
                    let b = entry[2];

                    let contrast = |v: f32| -> f32 { ((v - 0.5) * 1.15 + 0.5).clamp(0.0, 1.0) };
                    *entry = [contrast(r), contrast(g), contrast(b)];
                }
            }
            LutPreset::FilmPunch => {
                // Kodachrome-style vibrant colors with rich saturation
                for entry in lut.table.iter_mut() {
                    let r = entry[0];
                    let g = entry[1];
                    let b = entry[2];

                    let lum = 0.299 * r + 0.587 * g + 0.114 * b;
                    let sat_boost = 1.25;

                    let new_r = (lum + (r - lum) * sat_boost).clamp(0.0, 1.0);
                    let new_g = (lum + (g - lum) * sat_boost).clamp(0.0, 1.0);
                    let new_b = (lum + (b - lum) * sat_boost).clamp(0.0, 1.0);

                    *entry = [new_r, new_g, new_b];
                }
            }
            LutPreset::None => {}
        }

        lut
    }

    /// Evaluates trilinear interpolation for an input RGB color in [0.0, 1.0].
    #[inline(always)]
    pub fn sample_trilinear(&self, r: f32, g: f32, b: f32) -> [f32; 3] {
        let max_idx = (self.size - 1) as f32;

        let r_scaled = (r.clamp(0.0, 1.0) * max_idx).min(max_idx - 1e-5);
        let g_scaled = (g.clamp(0.0, 1.0) * max_idx).min(max_idx - 1e-5);
        let b_scaled = (b.clamp(0.0, 1.0) * max_idx).min(max_idx - 1e-5);

        let r0 = r_scaled as usize;
        let g0 = g_scaled as usize;
        let b0 = b_scaled as usize;

        let r1 = (r0 + 1).min(self.size - 1);
        let g1 = (g0 + 1).min(self.size - 1);
        let b1 = (b0 + 1).min(self.size - 1);

        let tr = r_scaled - r0 as f32;
        let tg = g_scaled - g0 as f32;
        let tb = b_scaled - b0 as f32;

        let idx = |ri: usize, gi: usize, bi: usize| -> usize {
            ri * self.size * self.size + gi * self.size + bi
        };

        // 8 corner values of the color cube
        let c000 = self.table[idx(r0, g0, b0)];
        let c001 = self.table[idx(r0, g0, b1)];
        let c010 = self.table[idx(r0, g0 + 1, b0)];
        let c011 = self.table[idx(r0, g0 + 1, b1)];
        let c100 = self.table[idx(r1, g0, b0)];
        let c101 = self.table[idx(r1, g0, b1)];
        let c110 = self.table[idx(r1, g1, b0)];
        let c111 = self.table[idx(r1, g1, b1)];

        let mut out = [0.0f32; 3];
        for c in 0..3 {
            let i00 = c000[c] * (1.0 - tr) + c100[c] * tr;
            let i01 = c001[c] * (1.0 - tr) + c101[c] * tr;
            let i10 = c010[c] * (1.0 - tr) + c110[c] * tr;
            let i11 = c011[c] * (1.0 - tr) + c111[c] * tr;

            let j0 = i00 * (1.0 - tg) + i10 * tg;
            let j1 = i01 * (1.0 - tg) + i11 * tg;

            out[c] = j0 * (1.0 - tb) + j1 * tb;
        }

        out
    }

    /// Applies the 3D LUT to all pixels in a video frame in-place.
    pub fn apply_to_frame(&self, frame: &mut VideoFrame) {
        let inv_255 = 1.0 / 255.0f32;

        for chunk in frame.data.chunks_exact_mut(3) {
            let r_f = chunk[0] as f32 * inv_255;
            let g_f = chunk[1] as f32 * inv_255;
            let b_f = chunk[2] as f32 * inv_255;

            let graded = self.sample_trilinear(r_f, g_f, b_f);

            chunk[0] = (graded[0] * 255.0).round().clamp(0.0, 255.0) as u8;
            chunk[1] = (graded[1] * 255.0).round().clamp(0.0, 255.0) as u8;
            chunk[2] = (graded[2] * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }
}
