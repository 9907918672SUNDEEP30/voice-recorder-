use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionEnv {
    Local,
    LightningCloud,
}

impl Default for ExecutionEnv {
    fn default() -> Self {
        Self::Local
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    pub models_dir: PathBuf,
    pub qwen_path: PathBuf,
    pub tts_path: PathBuf,
    pub whisper_path: PathBuf,
    pub avatar_default: PathBuf,
    pub preview_dir: PathBuf,
    pub content_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpConfig {
    pub host: String,
    pub port: u16,
    pub timeout_secs: u64,
}

impl CdpConfig {
    pub fn json_version_url(&self) -> String {
        format!("http://{}:{}/json/version", self.host, self.port)
    }

    pub fn json_list_url(&self) -> String {
        format!("http://{}:{}/json/list", self.host, self.port)
    }

    pub fn json_new_url(&self) -> String {
        format!("http://{}:{}/json/new", self.host, self.port)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningCloudConfig {
    pub ssh_host: String,
    pub ssh_port: u16,
    pub ssh_user: String,
    pub ssh_key_path: PathBuf,
    pub remote_dir: String,
    pub remote_models_dir: String,
    pub gpu_active: bool,
    pub direct_cloud_publish: bool,
    pub download_preview_only: bool,
    pub preview_max_height: u32,
    pub poll_interval_secs: u64,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentEngineConfig {
    pub execution_env: ExecutionEnv,
    pub local: LocalConfig,
    pub cdp: CdpConfig,
    pub cloud: LightningCloudConfig,
    pub default_topic: String,
    pub publish_platforms: Vec<String>,
}

impl ContentEngineConfig {
    pub fn load() -> Result<Self> {
        // Attempt to load .env from current dir or ancestor dirs
        let _ = dotenvy::dotenv();

        let env_mode_str = env::var("EXECUTION_ENV").unwrap_or_else(|_| "LOCAL".to_string());
        let execution_env = match env_mode_str.to_uppercase().as_str() {
            "LIGHTNING_CLOUD" | "CLOUD" => ExecutionEnv::LightningCloud,
            _ => ExecutionEnv::Local,
        };

        let models_dir = PathBuf::from(
            env::var("LOCAL_MODELS_DIR").unwrap_or_else(|_| "C:/AI_Models".to_string()),
        );
        let qwen_path = PathBuf::from(
            env::var("LOCAL_QWEN_PATH").unwrap_or_else(|_| "C:/AI_Models/qwen2-vl-2b".to_string()),
        );
        let tts_path = PathBuf::from(
            env::var("LOCAL_TTS_PATH").unwrap_or_else(|_| "C:/AI_Models/piper_hi".to_string()),
        );
        let whisper_path = PathBuf::from(
            env::var("LOCAL_WHISPER_PATH")
                .unwrap_or_else(|_| "C:/AI_Models/whisper_tiny_onnx".to_string()),
        );
        let avatar_default = PathBuf::from(
            env::var("LOCAL_AVATAR_DEFAULT")
                .unwrap_or_else(|_| "assets/avatars/default_anchor.png".to_string()),
        );
        let preview_dir = PathBuf::from(
            env::var("LOCAL_PREVIEW_DIR").unwrap_or_else(|_| "output/previews".to_string()),
        );
        let content_dir = PathBuf::from(
            env::var("LOCAL_CONTENT_DIR")
                .unwrap_or_else(|_| "output/omnichannel_content".to_string()),
        );

        let cdp_host = env::var("CDP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let cdp_port: u16 = env::var("CDP_PORT")
            .unwrap_or_else(|_| "9222".to_string())
            .parse()
            .unwrap_or(9222);
        let cdp_timeout_secs: u64 = env::var("CDP_TIMEOUT_SECS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);

        let ssh_host =
            env::var("LIGHTNING_SSH_HOST").unwrap_or_else(|_| "lightning-ai-node".to_string());
        let ssh_port: u16 = env::var("LIGHTNING_SSH_PORT")
            .unwrap_or_else(|_| "22".to_string())
            .parse()
            .unwrap_or(22);
        let ssh_user = env::var("LIGHTNING_SSH_USER").unwrap_or_else(|_| "zeus".to_string());
        let ssh_key_path = PathBuf::from(
            env::var("LIGHTNING_SSH_KEY_PATH").unwrap_or_else(|_| "~/.ssh/lightning_rsa".to_string()),
        );
        let remote_dir = env::var("LIGHTNING_REMOTE_DIR")
            .unwrap_or_else(|_| "/home/zeus/content_engine_workspace".to_string());
        let remote_models_dir = env::var("LIGHTNING_REMOTE_MODELS_DIR")
            .unwrap_or_else(|_| "/home/zeus/AI_Models".to_string());
        let gpu_active = env::var("LIGHTNING_GPU_ACTIVE")
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(false);
        let direct_cloud_publish = env::var("DIRECT_CLOUD_PUBLISH")
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(true);
        let download_preview_only = env::var("DOWNLOAD_PREVIEW_ONLY")
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(true);
        let preview_max_height: u32 = env::var("PREVIEW_MAX_HEIGHT")
            .unwrap_or_else(|_| "360".to_string())
            .parse()
            .unwrap_or(360);
        let poll_interval_secs: u64 = env::var("LIGHTNING_POLL_INTERVAL_SECS")
            .unwrap_or_else(|_| "3".to_string())
            .parse()
            .unwrap_or(3);
        let timeout_secs: u64 = env::var("LIGHTNING_TIMEOUT_SECS")
            .unwrap_or_else(|_| "300".to_string())
            .parse()
            .unwrap_or(300);

        let default_topic = env::var("DEFAULT_TOPIC")
            .unwrap_or_else(|_| "Artificial Intelligence Breakthroughs".to_string());
        let publish_platforms = env::var("PUBLISH_PLATFORMS")
            .unwrap_or_else(|_| "youtube_shorts,instagram_reels,x_twitter,blog".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let cfg = Self {
            execution_env,
            local: LocalConfig {
                models_dir,
                qwen_path,
                tts_path,
                whisper_path,
                avatar_default,
                preview_dir,
                content_dir,
            },
            cdp: CdpConfig {
                host: cdp_host,
                port: cdp_port,
                timeout_secs: cdp_timeout_secs,
            },
            cloud: LightningCloudConfig {
                ssh_host,
                ssh_port,
                ssh_user,
                ssh_key_path,
                remote_dir,
                remote_models_dir,
                gpu_active,
                direct_cloud_publish,
                download_preview_only,
                preview_max_height,
                poll_interval_secs,
                timeout_secs,
            },
            default_topic,
            publish_platforms,
        };

        cfg.ensure_local_directories()
            .context("Failed to initialize output directory structure")?;

        Ok(cfg)
    }

    pub fn ensure_local_directories(&self) -> Result<()> {
        if !self.local.preview_dir.exists() {
            std::fs::create_dir_all(&self.local.preview_dir)?;
        }
        if !self.local.content_dir.exists() {
            std::fs::create_dir_all(&self.local.content_dir)?;
        }
        Ok(())
    }

    pub fn is_cloud_active(&self) -> bool {
        self.execution_env == ExecutionEnv::LightningCloud && self.cloud.gpu_active
    }
}
