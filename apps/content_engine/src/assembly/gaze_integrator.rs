use anyhow::Result;
use video_studio::capture::VideoFrame;
use video_studio::gaze::{GazeCorrectionConfig, GazeCorrector, IrisTracker};
use video_studio::tracking::FaceLandmarks;

pub struct GazeIntegrator {
    corrector: GazeCorrector,
}

impl GazeIntegrator {
    pub fn new(strength: f32) -> Self {
        let config = GazeCorrectionConfig {
            correction_strength: strength,
            vertical_lift_ratio: 0.35,
            horizontal_align_ratio: 0.05,
            downward_pitch_deg: 12.0,
        };

        Self {
            corrector: GazeCorrector::new(config),
        }
    }

    /// Warps avatar eyes so the presenter looks directly into the camera lens
    pub fn apply_gaze_correction(
        &mut self,
        frame: &mut VideoFrame,
        landmarks: &FaceLandmarks,
    ) -> Result<()> {
        let (left_eye, right_eye) = IrisTracker::locate_eyes(frame, landmarks);
        self.corrector.apply_eye_contact(frame, &left_eye, &right_eye);
        Ok(())
    }
}
