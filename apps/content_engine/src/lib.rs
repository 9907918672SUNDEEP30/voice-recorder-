//! # Universal Autonomous Content & Media Engine
//!
//! Production-grade, 100% niche-neutral, zero-paid-API media production pipeline.
//! Cross-platform trend reconnaissance, omnichannel script/blog generation,
//! voiceover synthesis, ASS karaoke subtitles, and hybrid cloud GPU video production.

pub mod assembly;
pub mod audio;
pub mod cloud;
pub mod config;
pub mod generator;
pub mod pipeline;
pub mod recon;

pub use assembly::{GazeIntegrator, LocalCompositor, LutIntegrator};
pub use audio::{SubtitleEngine, TtsVoiceEngine, WordTimestamp};
pub use cloud::{CloudWorkerGenerator, SshBridge};
pub use config::{CdpConfig, ContentEngineConfig, ExecutionEnv, LightningCloudConfig, LocalConfig};
pub use generator::{OmnichannelGenerator, OmnichannelPackage, PromptEngine, ShortVideoScript};
pub use pipeline::ContentEngineOrchestrator;
pub use recon::{CompetitorVideo, DossierBuilder, NewsArticle, TrendDossier, TrendReconEngine};
