use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use tokio::process::Command;

pub struct SshBridge {
    host: String,
    port: u16,
    user: String,
    key_path: PathBuf,
    remote_dir: String,
}

impl SshBridge {
    pub fn new(
        host: &str,
        port: u16,
        user: &str,
        key_path: &Path,
        remote_dir: &str,
    ) -> Self {
        Self {
            host: host.to_string(),
            port,
            user: user.to_string(),
            key_path: key_path.to_path_buf(),
            remote_dir: remote_dir.to_string(),
        }
    }

    /// Verifies if the Lightning AI cloud node is reachable and GPU is available
    pub async fn test_gpu_connection(&self) -> Result<String> {
        let destination = format!("{}@{}", self.user, self.host);
        let port_str = self.port.to_string();

        let mut cmd = Command::new("ssh");
        cmd.arg("-p")
            .arg(&port_str)
            .arg("-o")
            .arg("ConnectTimeout=5")
            .arg("-o")
            .arg("StrictHostKeyChecking=accept-new");

        if self.key_path.exists() {
            cmd.arg("-i").arg(&self.key_path);
        }

        cmd.arg(&destination).arg("nvidia-smi --query-gpu=name,memory.total,memory.free --format=csv,noheader");

        let output = cmd.output().await.map_err(|e| {
            anyhow!("Failed to execute ssh command: {}. Ensure OpenSSH client is on PATH.", e)
        })?;

        if output.status.success() {
            let res = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(res)
        } else {
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(anyhow!("SSH connection to {} failed: {}", self.host, err))
        }
    }

    /// Uploads lightweight payload (~5 MB: avatar, audio, captions, python script) via SCP
    pub async fn upload_input_payload(
        &self,
        avatar_path: &Path,
        audio_path: &Path,
        subtitles_path: Option<&Path>,
        script_path: &Path,
    ) -> Result<()> {
        println!("📤 [SSH Bridge] Uploading ~5 MB payload to Lightning AI cloud node...");

        // Ensure remote input and output directories exist
        self.run_remote_command(&format!("mkdir -p {}/input {}/output", self.remote_dir, self.remote_dir)).await?;

        let destination = format!("{}@{}:{}/input/", self.user, self.host, self.remote_dir);
        let port_str = self.port.to_string();

        let mut scp_cmd = Command::new("scp");
        scp_cmd.arg("-P").arg(&port_str).arg("-o").arg("StrictHostKeyChecking=accept-new");

        if self.key_path.exists() {
            scp_cmd.arg("-i").arg(&self.key_path);
        }

        scp_cmd.arg(avatar_path).arg(audio_path).arg(script_path);

        if let Some(sub) = subtitles_path {
            if sub.exists() {
                scp_cmd.arg(sub);
            }
        }

        scp_cmd.arg(&destination);

        let output = scp_cmd.output().await.map_err(|e| anyhow!("Failed to execute scp: {}", e))?;
        if output.status.success() {
            println!("  ✅ Payload uploaded successfully to {}/input/", self.remote_dir);
            Ok(())
        } else {
            let err = String::from_utf8_lossy(&output.stderr).to_string();
            Err(anyhow!("SCP upload failed: {}", err))
        }
    }

    /// Triggers remote pipeline execution (FasterLivePortrait + CodeFormer 4K + Cloud NVENC)
    pub async fn trigger_remote_pipeline(&self) -> Result<()> {
        println!("⚡ [SSH Bridge] Triggering cloud execution: run_cloud_pipeline.py");
        let remote_exec = format!("python3 {}/input/run_cloud_pipeline.py", self.remote_dir);
        let output = self.run_remote_command(&remote_exec).await?;
        println!("  ✅ Cloud Pipeline Output:\n{}", output);
        Ok(())
    }

    /// Downloads only the tiny 360p preview video (~5 MB) to local PC for fast inspection
    pub async fn download_preview_proxy(&self, local_dest: &Path) -> Result<()> {
        println!("📥 [SSH Bridge] Downloading lightweight 360p preview (~5 MB)...");
        let remote_src = format!("{}@{}:{}/output/preview_360p.mp4", self.user, self.host, self.remote_dir);
        let port_str = self.port.to_string();

        let mut scp_cmd = Command::new("scp");
        scp_cmd.arg("-P").arg(&port_str).arg("-o").arg("StrictHostKeyChecking=accept-new");

        if self.key_path.exists() {
            scp_cmd.arg("-i").arg(&self.key_path);
        }

        scp_cmd.arg(&remote_src).arg(local_dest);

        let output = scp_cmd.output().await.map_err(|e| anyhow!("Failed to download preview: {}", e))?;
        if output.status.success() {
            println!("  ✅ 360p Preview downloaded to {:?}", local_dest);
            Ok(())
        } else {
            let err = String::from_utf8_lossy(&output.stderr).to_string();
            Err(anyhow!("Preview download failed: {}", err))
        }
    }

    /// Triggers direct cloud publishing from Lightning AI node using safe JSON config upload via SCP
    pub async fn trigger_cloud_publish(&self, title: &str, tags: &str) -> Result<String> {
        println!("🚀 [SSH Bridge] Triggering Direct Cloud Publishing via safe JSON payload...");
        
        // Write payload to a safe local temporary JSON file
        let temp_dir = std::env::temp_dir();
        let payload_path = temp_dir.join(format!("publish_payload_{}_{}.json", std::process::id(), chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)));
        let payload_json = serde_json::json!({
            "title": title,
            "tags": tags,
            "video_path": format!("{}/output/master_4k.mp4", self.remote_dir),
            "timestamp": chrono::Utc::now().timestamp()
        });
        tokio::fs::write(&payload_path, serde_json::to_string_pretty(&payload_json)?).await?;

        // Upload payload JSON via SCP
        let destination = format!("{}@{}:{}/input/publish_payload.json", self.user, self.host, self.remote_dir);
        let port_str = self.port.to_string();

        let mut scp_cmd = Command::new("scp");
        scp_cmd.arg("-P").arg(&port_str).arg("-o").arg("StrictHostKeyChecking=accept-new");

        if self.key_path.exists() {
            scp_cmd.arg("-i").arg(&self.key_path);
        }

        scp_cmd.arg(&payload_path).arg(&destination);
        let output = scp_cmd.output().await.map_err(|e| anyhow!("Failed to upload publish config: {}", e))?;
        let _ = tokio::fs::remove_file(&payload_path).await; // Clean up local temp file

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(anyhow!("Publish config SCP upload failed: {}", err));
        }

        // Execute remote publisher with purely static file path reference (zero shell interpolation)
        let remote_exec = format!(
            "python3 {}/input/cloud_publisher.py --config {}/input/publish_payload.json",
            self.remote_dir, self.remote_dir
        );
        let output = self.run_remote_command(&remote_exec).await?;
        Ok(output)
    }

    async fn run_remote_command(&self, remote_cmd: &str) -> Result<String> {
        let destination = format!("{}@{}", self.user, self.host);
        let port_str = self.port.to_string();

        let mut cmd = Command::new("ssh");
        cmd.arg("-p")
            .arg(&port_str)
            .arg("-o")
            .arg("ConnectTimeout=10")
            .arg("-o")
            .arg("StrictHostKeyChecking=accept-new");

        if self.key_path.exists() {
            cmd.arg("-i").arg(&self.key_path);
        }

        cmd.arg(&destination).arg(remote_cmd);

        let output = cmd.output().await.map_err(|e| anyhow!("SSH execution failed: {}", e))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(anyhow!("Remote command failed: {}", err))
        }
    }
}
