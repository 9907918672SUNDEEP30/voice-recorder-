use video_studio::capture::VideoFrame;
use video_studio::grading::{Lut3D, LutPreset};

pub struct LutIntegrator {
    lut: Lut3D,
}

impl LutIntegrator {
    pub fn new(preset: LutPreset) -> Self {
        Self {
            lut: Lut3D::from_preset(preset),
        }
    }

    /// Applies cinematic color grading (e.g. Teal & Orange / Film Punch) to the frame
    pub fn grade_frame(&self, frame: &mut VideoFrame) {
        self.lut.apply_to_frame(frame);
    }
}
