//! # Cloud Video Rendering & Transcoder Engine
//!
//! Provides hardware-accelerated rendering, FFmpeg probe diagnostics,
//! and automated cloud rendering pipelines.

pub mod ffmpeg_probe;

pub use ffmpeg_probe::{FfmpegDiagnosticError, FfmpegInfo, FfmpegProbe, HardwareEncoder};
