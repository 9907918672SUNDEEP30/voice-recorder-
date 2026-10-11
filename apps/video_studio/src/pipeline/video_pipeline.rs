use anyhow::Result;
use std::time::Instant;

use crate::capture::{CameraSource, VideoFrame};
use crate::gaze::{GazeCorrectionConfig, GazeCorrector, IrisTracker};
use crate::grading::{Lut3D, LutPreset, SkinFilter};
use crate::tracking::{AutoFramer, AutoFramingConfig, FaceDetector};

/// Studio pipeline configuration options.
#[derive(Debug, Clone)]
pub struct StudioPipelineConfig {
    pub output_width: u32,
    pub output_height: u32,
    pub auto_framing_enabled: bool,
    pub gaze_correction_enabled: bool,
    pub gaze_strength: f32,
    pub skin_smoothing_enabled: bool,
    pub skin_strength: f32,
    pub relighting_enabled: bool,
    pub lut_preset: LutPreset,
}

impl Default for StudioPipelineConfig {
    fn default() -> Self {
        Self {
            output_width: 1280,
            output_height: 720,
            auto_framing_enabled: true,
            gaze_correction_enabled: true,
            gaze_strength: 0.85,
            skin_smoothing_enabled: true,
            skin_strength: 0.60,
            relighting_enabled: true,
            lut_preset: LutPreset::TealAndOrange,
        }
    }
}

/// Real-time live video pipeline telemetry.
#[derive(Debug, Clone, Copy, Default)]
pub struct PipelineTelemetry {
    pub frame_index: u64,
    pub fps: f32,
    pub latency_ms: f32,
    pub face_detected: bool,
    pub eye_contact_active: bool,
    pub zoom_level: f32,
}

/// Central Studio Video Pipeline.
pub struct StudioVideoPipeline {
    config: StudioPipelineConfig,
    camera: Box<dyn CameraSource>,
    face_detector: FaceDetector,
    auto_framer: AutoFramer,
    gaze_corrector: GazeCorrector,
    skin_filter: SkinFilter,
    lut_engine: Lut3D,
    frame_count: u64,
    last_frame_time: Instant,
    telemetry: PipelineTelemetry,
}

impl StudioVideoPipeline {
    /// Creates a new Studio Video Pipeline with the specified camera and configuration.
    pub fn new(camera: Box<dyn CameraSource>, config: StudioPipelineConfig) -> Self {
        let auto_framer = AutoFramer::new(AutoFramingConfig::default());
        let gaze_corrector = GazeCorrector::new(GazeCorrectionConfig {
            correction_strength: config.gaze_strength,
            ..Default::default()
        });
        let skin_filter = SkinFilter::new(config.skin_strength, 0.25);
        let lut_engine = Lut3D::from_preset(config.lut_preset);

        Self {
            config,
            camera,
            face_detector: FaceDetector::new(0.50),
            auto_framer,
            gaze_corrector,
            skin_filter,
            lut_engine,
            frame_count: 0,
            last_frame_time: Instant::now(),
            telemetry: PipelineTelemetry::default(),
        }
    }

    /// Starts camera capture.
    pub fn start(&mut self) -> Result<()> {
        self.camera.start()?;
        self.last_frame_time = Instant::now();
        Ok(())
    }

    /// Stops camera capture.
    pub fn stop(&mut self) -> Result<()> {
        self.camera.stop()
    }

    /// Sets 3D LUT preset on the fly.
    pub fn set_lut_preset(&mut self, preset: LutPreset) {
        self.config.lut_preset = preset;
        self.lut_engine = Lut3D::from_preset(preset);
    }

    /// Sets eye contact correction strength.
    pub fn set_gaze_strength(&mut self, strength: f32) {
        self.config.gaze_strength = strength;
        self.gaze_corrector.set_strength(strength);
    }

    /// Toggles auto-framing on or off.
    pub fn set_auto_framing(&mut self, enabled: bool) {
        self.config.auto_framing_enabled = enabled;
        self.auto_framer.set_active(enabled);
    }

    /// Processes a single frame through the complete AI vision and grading pipeline.
    pub fn process_next_frame(&mut self) -> Result<VideoFrame> {
        let start_time = Instant::now();

        // 1. Ingest raw frame from camera source
        let raw_frame = self.camera.capture_frame()?;

        // 2. Real-time Face & Landmark Detection
        let face_info = self.face_detector.detect_primary_face(&raw_frame);
        let has_face = face_info.is_some();

        // 3. Auto-Framing & Virtual PTZ Cameraman
        let framed_frame = if self.config.auto_framing_enabled {
            let face_box_ref = face_info.as_ref().map(|(b, _)| b);
            let _crop = self.auto_framer.update(face_box_ref);
            self.auto_framer.render_framed_output(
                &raw_frame,
                self.config.output_width,
                self.config.output_height,
            )
        } else {
            raw_frame
        };

        let mut output_frame = framed_frame;

        // 4. Re-detect face on framed output for eye contact & skin styling
        let framed_face_info = self.face_detector.detect_primary_face(&output_frame);

        // 5. AI Eye Contact Correction (Teleprompter Iris Warping)
        if self.config.gaze_correction_enabled {
            if let Some((_, ref landmarks)) = framed_face_info {
                let (left_eye, right_eye) = IrisTracker::locate_eyes(&output_frame, landmarks);
                self.gaze_corrector.apply_eye_contact(
                    &mut output_frame,
                    &left_eye,
                    &right_eye,
                );
            }
        }

        // 6. Skin Smoothing & Edge-Preserving Studio Relighting
        if self.config.skin_smoothing_enabled || self.config.relighting_enabled {
            let face_ref = framed_face_info.as_ref().map(|(b, _)| b);
            if self.config.skin_smoothing_enabled {
                self.skin_filter.apply_skin_smoothing(&mut output_frame, face_ref);
            }
            if self.config.relighting_enabled {
                self.skin_filter.apply_studio_relighting(&mut output_frame, face_ref);
            }
        }

        // 7. 3D LUT Cinematic Color Grading
        if self.config.lut_preset != LutPreset::None {
            self.lut_engine.apply_to_frame(&mut output_frame);
        }

        // Update telemetry
        let elapsed_ms = start_time.elapsed().as_secs_f32() * 1000.0;
        self.frame_count += 1;

        let frame_dt = self.last_frame_time.elapsed().as_secs_f32();
        let fps = if frame_dt > 0.0001 { 1.0 / frame_dt } else { 30.0 };
        self.last_frame_time = Instant::now();

        self.telemetry = PipelineTelemetry {
            frame_index: self.frame_count,
            fps,
            latency_ms: elapsed_ms,
            face_detected: has_face,
            eye_contact_active: self.config.gaze_correction_enabled && has_face,
            zoom_level: 1.35,
        };

        Ok(output_frame)
    }

    /// Retrieves current telemetry snapshot.
    pub fn telemetry(&self) -> PipelineTelemetry {
        self.telemetry
    }
}
