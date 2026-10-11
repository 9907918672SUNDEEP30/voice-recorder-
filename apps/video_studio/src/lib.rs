//! # AI Video Studio Engine
//!
//! Production-grade, zero-latency local video processing, auto-framing,
//! teleprompter gaze correction, and cinematic 3D LUT grading engine built in pure Rust.

pub mod capture;
pub mod gaze;
pub mod grading;
pub mod pipeline;
pub mod storage;
pub mod tracking;

// Public Convenience Re-exports
pub use capture::{CameraSource, NokhwaCamera, VideoFrame, VirtualCamera};
pub use gaze::{EyeRegion, GazeCorrectionConfig, GazeCorrector, IrisTracker};
pub use grading::{Lut3D, LutPreset, SkinFilter};
pub use pipeline::{PipelineTelemetry, StudioPipelineConfig, StudioVideoPipeline};
pub use storage::{FrameWriter, RecordMetrics};
pub use tracking::{AutoFramer, AutoFramingConfig, CropWindow, FaceBox, FaceDetector, FaceLandmarks};
