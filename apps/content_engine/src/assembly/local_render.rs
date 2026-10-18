use anyhow::Result;
use std::path::Path;
use tokio::process::Command;

use crate::render::FfmpegProbe;

pub struct LocalCompositor;

impl LocalCompositor {
    /// Renders a local preview video using dynamically probed FFmpeg & hardware encoders
    pub async fn render_local_preview(
        avatar_path: &Path,
        audio_path: &Path,
        subtitles_path: Option<&Path>,
        output_mp4_path: &Path,
    ) -> Result<()> {
        println!("🎬 [Assembly] Probing local/cloud FFmpeg installation...");

        let ffmpeg_info = match FfmpegProbe::probe().await {
            Ok(info) => {
                println!("  ✅ Probed FFmpeg: {} (Best encoder: {:?})", info.version, info.best_encoder);
                info
            }
            Err(e) => {
                println!("  ℹ️ {}", e);
                println!("     All assets (avatar, audio.wav, captions.ass, scripts) are ready in output folder for Lightning AI Cloud GPU.");
                return Ok(());
            }
        };

        println!("🎬 [Assembly] Rendering local video via {} (Codec: {})...", 
            ffmpeg_info.ffmpeg_path.display(),
            ffmpeg_info.best_encoder.codec_name()
        );

        let mut cmd = Command::new(&ffmpeg_info.ffmpeg_path);
        cmd.arg("-y")
            .arg("-loop")
            .arg("1")
            .arg("-i")
            .arg(avatar_path)
            .arg("-i")
            .arg(audio_path);

        if let Some(sub) = subtitles_path {
            if sub.exists() && ffmpeg_info.supports_ass_filter {
                // Escape Windows backslashes for FFmpeg ass filter if needed
                let sub_str = sub.to_string_lossy().replace('\\', "/").replace(':', "\\:");
                cmd.arg("-vf").arg(format!("ass='{}'", sub_str));
            }
        }

        cmd.arg("-c:v")
            .arg(ffmpeg_info.best_encoder.codec_name())
            .arg("-pix_fmt")
            .arg("yuv420p")
            .arg("-c:a")
            .arg("aac")
            .arg("-b:a")
            .arg("192k")
            .arg("-shortest")
            .arg(output_mp4_path);

        if !ffmpeg_info.best_encoder.is_hardware_accelerated() {
            cmd.arg("-preset")
                .arg("ultrafast")
                .arg("-tune")
                .arg("stillimage");
        }

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
                println!("  ⚠️ FFmpeg execution failed: {}", e);
            }
        }

        Ok(())
    }
}
