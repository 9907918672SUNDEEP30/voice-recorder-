//! Edge AI Threat Intelligence Subsystem for Shambhavi AI Labs Pehredar Box

pub mod model;
pub mod plugin;

pub use model::{BoundingBox, EdgeVisionModel, ThreatVerdict};
pub use plugin::{AiPluginStatus, PluggableAiEngine};
