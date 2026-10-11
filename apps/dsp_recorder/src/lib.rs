//! # Pure Rust Audio Recorder & Tone Cloner (DSP Engine)
//!
//! A production-ready, zero-latency local audio processing and recording engine
//! built on top of the AI Core Framework.

pub mod capture;
pub mod dsp;
pub mod pipeline;
pub mod storage;

// Public Convenience Re-exports
pub use capture::{AudioCaptureStream, CaptureConfig, StreamMetrics};
pub use dsp::{
    BiquadCascade, BiquadCoefficients, BiquadFilter, FilterType, ParameterSmoother, ToneCloner,
    ToneEqMatch, ToneProfile,
};
pub use pipeline::{
    LivePipelineStats, PipelineConfig, PipelineMode, StudioAudioPipeline,
};
pub use storage::{AsyncWavRecorder, BitDepth, WavWriterConfig, WriterMetrics};
