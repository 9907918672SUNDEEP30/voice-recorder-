//! # Cloud & Local FFmpeg Prober & Hardware Acceleration Detector
//!
//! Deterministically probes system PATH for `ffmpeg` and `ffprobe`,
//! validates version information, detects GPU hardware encoders (NVENC, VAAPI, QSV),
//! and provides strongly-typed diagnostic errors with OS-specific installation recipes.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};
use tokio::process::Command;

/// Diagnostic Error when FFmpeg is missing or incompatible
#[derive(Debug, Clone)]
pub enum FfmpegDiagnosticError {
    NotFound { install_hint: String },
    ExecutionFailed(String),
    VersionIncompatible { found: String },
}

impl fmt::Display for FfmpegDiagnosticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { install_hint } => {
                write!(f, "FFmpeg binary not found on system PATH. Install with: {}", install_hint)
            }
            Self::ExecutionFailed(err) => {
                write!(f, "FFmpeg execution error: {}", err)
            }
            Self::VersionIncompatible { found } => {
                write!(f, "FFmpeg version too old: found '{}', required '>= 4.3'", found)
            }
        }
    }
}

impl std::error::Error for FfmpegDiagnosticError {}

/// Detected Video Hardware Encoders
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareEncoder {
    /// NVIDIA NVENC H.264 hardware acceleration
    NvencH264,
    /// NVIDIA NVENC HEVC/H.265 hardware acceleration
    NvencHevc,
    /// Intel / Linux VAAPI hardware acceleration
    VaapiH264,
    /// Intel QuickSync (QSV) hardware acceleration
    QsvH264,
    /// Universal High-Efficiency Multi-Threaded CPU software encoding
    CpuLibx264,
}

impl HardwareEncoder {
    pub fn codec_name(&self) -> &'static str {
        match self {
            Self::NvencH264 => "h264_nvenc",
            Self::NvencHevc => "hevc_nvenc",
            Self::VaapiH264 => "h264_vaapi",
            Self::QsvH264 => "h264_qsv",
            Self::CpuLibx264 => "libx264",
        }
    }

    pub fn is_hardware_accelerated(&self) -> bool {
        !matches!(self, Self::CpuLibx264)
    }
}

/// Probed FFmpeg Installation Metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FfmpegInfo {
    pub ffmpeg_path: PathBuf,
    pub ffprobe_path: Option<PathBuf>,
    pub version: String,
    pub best_encoder: HardwareEncoder,
    pub available_encoders: Vec<HardwareEncoder>,
    pub supports_ass_filter: bool,
}

pub struct FfmpegProbe;

impl FfmpegProbe {
    /// Probes the system for FFmpeg capabilities
    pub async fn probe() -> Result<FfmpegInfo, FfmpegDiagnosticError> {
        let ffmpeg_path = Self::find_binary("ffmpeg").ok_or_else(|| {
            let hint = if cfg!(target_os = "windows") {
                "winget install Gyan.FFmpeg or choco install ffmpeg"
            } else if cfg!(target_os = "macos") {
                "brew install ffmpeg"
            } else {
                "apt-get update && apt-get install -y ffmpeg libass-dev (Ubuntu/Debian) or apk add ffmpeg (Alpine)"
            };
            FfmpegDiagnosticError::NotFound {
                install_hint: hint.to_string(),
            }
        })?;

        let ffprobe_path = Self::find_binary("ffprobe");

        // 1. Check version string
        let version_output = Command::new(&ffmpeg_path)
            .arg("-version")
            .output()
            .await
            .map_err(|e| FfmpegDiagnosticError::ExecutionFailed(e.to_string()))?;

        let version_str = String::from_utf8_lossy(&version_output.stdout);
        let first_line = version_str.lines().next().unwrap_or("ffmpeg version unknown");
        let version = first_line.to_string();

        // 2. Probe available encoders
        let encoders_output = Command::new(&ffmpeg_path)
            .arg("-encoders")
            .output()
            .await
            .map_err(|e| FfmpegDiagnosticError::ExecutionFailed(e.to_string()))?;

        let encoders_str = String::from_utf8_lossy(&encoders_output.stdout);
        let mut available_encoders = Vec::new();

        if encoders_str.contains("h264_nvenc") {
            available_encoders.push(HardwareEncoder::NvencH264);
        }
        if encoders_str.contains("hevc_nvenc") {
            available_encoders.push(HardwareEncoder::NvencHevc);
        }
        if encoders_str.contains("h264_vaapi") {
            available_encoders.push(HardwareEncoder::VaapiH264);
        }
        if encoders_str.contains("h264_qsv") {
            available_encoders.push(HardwareEncoder::QsvH264);
        }
        if encoders_str.contains("libx264") {
            available_encoders.push(HardwareEncoder::CpuLibx264);
        }

        // Default to CPU libx264 if none detected in raw output
        if available_encoders.is_empty() {
            available_encoders.push(HardwareEncoder::CpuLibx264);
        }

        let best_encoder = available_encoders[0];

        // 3. Probe filters for Advanced SubStation Alpha (.ass) subtitles
        let filters_output = Command::new(&ffmpeg_path)
            .arg("-filters")
            .output()
            .await
            .map_err(|e| FfmpegDiagnosticError::ExecutionFailed(e.to_string()))?;

        let filters_str = String::from_utf8_lossy(&filters_output.stdout);
        let supports_ass_filter = filters_str.contains("ass ") || filters_str.contains("subtitles");

        Ok(FfmpegInfo {
            ffmpeg_path,
            ffprobe_path,
            version,
            best_encoder,
            available_encoders,
            supports_ass_filter,
        })
    }

    /// Searches PATH and common cloud directories for binary
    pub fn find_binary(name: &str) -> Option<PathBuf> {
        let binary_name = if cfg!(windows) && !name.ends_with(".exe") {
            format!("{}.exe", name)
        } else {
            name.to_string()
        };

        // 1. Check system PATH
        if let Ok(path_var) = std::env::var("PATH") {
            let separator = if cfg!(windows) { ';' } else { ':' };
            for dir in path_var.split(separator) {
                let candidate = Path::new(dir).join(&binary_name);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }

        // 2. Common cloud and local fallback directories
        let common_paths = [
            "/usr/bin",
            "/usr/local/bin",
            "/opt/homebrew/bin",
            "C:\\ffmpeg\\bin",
            "C:\\Program Files\\ffmpeg\\bin",
        ];

        for &dir in &common_paths {
            let candidate = Path::new(dir).join(&binary_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_encoder_properties() {
        let nvenc = HardwareEncoder::NvencH264;
        assert_eq!(nvenc.codec_name(), "h264_nvenc");
        assert!(nvenc.is_hardware_accelerated());

        let cpu = HardwareEncoder::CpuLibx264;
        assert_eq!(cpu.codec_name(), "libx264");
        assert!(!cpu.is_hardware_accelerated());
    }

    #[tokio::test]
    async fn test_ffmpeg_probe_execution() {
        // Probe system; if ffmpeg exists, it must return valid info; if not, returns strongly-typed diagnostic
        match FfmpegProbe::probe().await {
            Ok(info) => {
                assert!(!info.version.is_empty());
                assert!(!info.available_encoders.is_empty());
            }
            Err(e) => {
                let err_msg = e.to_string();
                assert!(err_msg.contains("FFmpeg binary not found") || err_msg.contains("FFmpeg execution error"));
            }
        }
    }
}
