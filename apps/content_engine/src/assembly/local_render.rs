use anyhow::Result;
use std::path::Path;
use tokio::process::Command;

pub struct LocalCompositor;

impl LocalCompositor {
    /// Renders a local preview video using local FFmpeg
    pub async fn render_local_preview(
        avatar_path: &Path,
        audio_path: &Path,
        subtitles_path: Option<&Path>,
        output_mp4_path: &Path,
    ) -> Result<()> {
        println!("🎬 [Assembly] Rendering local fallback video via FFmpeg...");

        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-y")
            .arg("-loop")
            .arg("1")
            .arg("-i")
            .arg(avatar_path)
            .arg("-i")
            .arg(audio_path);

        if let Some(sub) = subtitles_path {
            if sub.exists() {
                // Escape Windows backslashes for FFmpeg ass filter if needed
                let sub_str = sub.to_string_lossy().replace('\\', "/").replace(':', "\\:");
                cmd.arg("-vf").arg(format!("ass='{}'", sub_str));
            }
        }

        cmd.arg("-c:v")
            .arg("libx264")
            .arg("-preset")
            .arg("ultrafast")
            .arg("-tune")
            .arg("stillimage")
            .arg("-pix_fmt")
            .arg("yuv420p")
            .arg("-c:a")
            .arg("aac")
            .arg("-b:a")
            .arg("192k")
            .arg("-shortest")
            .arg(output_mp4_path);

        match cmd.output().await {
            Ok(output) => {
                if output.status.success() {
                    println!("  ✅ Local video composite generated: {:?}", output_mp4_path);
                } else {
                    let err = String::from_utf8_lossy(&output.stderr);
                    println!("  ℹ️ Local FFmpeg render note: {}", err.lines().next().unwrap_or(""));
                }
            }
            Err(e) => {
                println!("  ℹ️ Local FFmpeg binary not found on PATH ({}). Skipping local MP4 render.", e);
                println!("     All assets (avatar, audio.wav, captions.ass, scripts) are ready in output folder for Lightning AI Cloud GPU.");
            }
        }

        Ok(())
    }
}
