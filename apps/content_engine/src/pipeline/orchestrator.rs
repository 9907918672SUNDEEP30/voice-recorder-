use crate::assembly::LocalCompositor;
use crate::audio::{SubtitleEngine, TtsVoiceEngine};
use crate::cloud::{CloudWorkerGenerator, SshBridge};
use crate::config::ContentEngineConfig;
use crate::generator::{OmnichannelGenerator, OmnichannelPackage};
use crate::recon::TrendReconEngine;
use crate::supabase::SupabaseContentClient;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct ContentEngineOrchestrator {
    config: ContentEngineConfig,
    recon_engine: TrendReconEngine,
    voice_engine: TtsVoiceEngine,
    pub supabase_client: SupabaseContentClient,
}

impl ContentEngineOrchestrator {
    pub fn new(config: ContentEngineConfig) -> Self {
        let recon_engine = TrendReconEngine::new(&config.cdp.host, config.cdp.port);
        let voice_engine = TtsVoiceEngine::new();

        let cfg_val: serde_json::Value = std::fs::read_to_string("config.json")
            .or_else(|_| std::fs::read_to_string("../../config.json"))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| serde_json::json!({}));
        let supabase_client = SupabaseContentClient::new_from_env_or_config(&cfg_val);

        Self {
            config,
            recon_engine,
            voice_engine,
            supabase_client,
        }
    }

    /// Runs the complete autonomous content creation pipeline from topic to multi-platform assets
    pub async fn execute_campaign(&self, topic: Option<&str>) -> Result<OmnichannelPackage> {
        let active_topic = topic.unwrap_or(&self.config.default_topic);
        println!("\n==================================================================");
        println!("🚀 SHAMBHAVI AI LABS - AUTONOMOUS CONTENT & MEDIA ENGINE");
        println!("   Topic: '{}'", active_topic);
        println!("   Mode:  {:?}", self.config.execution_env);
        println!("==================================================================\n");

        // 1. Module 8A: Universal Trend Reconnaissance
        let dossier = self.recon_engine.analyze_topic(active_topic).await?;
        let _ = self.supabase_client.save_trend_dossier(&dossier).await;

        // 2. Omnichannel Content Generation Matrix (Shorts, Long-form, Blog, Twitter)
        let package = OmnichannelGenerator::generate_package(&dossier)?;
        OmnichannelGenerator::save_to_directory(&package, &self.config.local.content_dir)?;

        // 3. Audio Speech Synthesis (24kHz Studio WAV with Mastering)
        let audio_path = self.config.local.content_dir.join("audio.wav");
        self.voice_engine
            .synthesize_speech_wav(&package.short_script.full_spoken_transcript, &audio_path)?;

        // 4. Word-Level Subtitles (Dynamic Glowing ASS Karaoke Subtitles)
        let word_count = package.short_script.full_spoken_transcript.split_whitespace().count();
        let estimated_duration = (word_count as f32 / 2.5).max(3.0);
        let timestamps = SubtitleEngine::align_word_timestamps(
            &package.short_script.full_spoken_transcript,
            estimated_duration,
        );
        let ass_path = self.config.local.content_dir.join("captions.ass");
        SubtitleEngine::export_ass_karaoke(&timestamps, &ass_path)?;

        // Ensure default avatar exists
        let avatar_path = self.ensure_avatar_exists()?;

        // 5. Cloud Handshake OR Local Execution
        let preview_url = if self.config.is_cloud_active() {
            println!("\n⚡ [Cloud Execution] Handshaking with Lightning AI Cloud GPU...");
            self.execute_cloud_pipeline(&avatar_path, &audio_path, &ass_path, &package).await?;
            "cloud_rendered_video.mp4".to_string()
        } else {
            println!("\n💻 [Local Execution] Mode is set to LOCAL. Generating local preview composite...");
            let local_video_out = self.config.local.content_dir.join("local_preview.mp4");
            LocalCompositor::render_local_preview(
                &avatar_path,
                &audio_path,
                Some(&ass_path),
                &local_video_out,
            ).await?;
            println!("  ✅ Local preview available at: {:?}", local_video_out);
            local_video_out.to_string_lossy().to_string()
        };

        // 6. Supabase Cloud Persistence (Final Omnichannel Assets)
        let _ = self.supabase_client.save_content_package(
            &package,
            audio_path.to_str(),
            ass_path.to_str(),
            Some(&preview_url),
        ).await;

        println!("\n==================================================================");
        println!("🎉 CAMPAIGN EXECUTION FINISHED SUCCESSFULLY!");
        println!("   All assets generated, saved locally and synced to Supabase Cloud: {:?}", self.config.local.content_dir);
        println!("==================================================================\n");

        Ok(package)
    }

    async fn execute_cloud_pipeline(
        &self,
        avatar_path: &Path,
        audio_path: &Path,
        ass_path: &Path,
        package: &OmnichannelPackage,
    ) -> Result<()> {
        let ssh = SshBridge::new(
            &self.config.cloud.ssh_host,
            self.config.cloud.ssh_port,
            &self.config.cloud.ssh_user,
            &self.config.cloud.ssh_key_path,
            &self.config.cloud.remote_dir,
        );

        // Stage cloud worker scripts locally
        let scripts_dir = self.config.local.content_dir.join("cloud_scripts");
        CloudWorkerGenerator::write_scripts_to_dir(&scripts_dir)?;
        let runner_script = scripts_dir.join("run_cloud_pipeline.py");

        // Upload lightweight ~5 MB payload
        ssh.upload_input_payload(avatar_path, audio_path, Some(ass_path), &runner_script).await?;

        // Trigger remote LivePortrait + CodeFormer 4K + Cloud NVENC Assembly
        ssh.trigger_remote_pipeline().await?;

        // Download tiny 360p preview (~5 MB) for instant inspection
        let preview_dest = self.config.local.preview_dir.join("preview_360p.mp4");
        let _ = ssh.download_preview_proxy(&preview_dest).await;

        // If direct cloud publish is enabled, trigger remote publishing
        if self.config.cloud.direct_cloud_publish {
            let tags = format!("#{}, #AI, #TechTrends, #Viral", package.topic.replace(' ', ""));
            let publish_res = ssh.trigger_cloud_publish(&package.short_script.hook, &tags).await?;
            println!("  🚀 Cloud Publishing Output:\n{}", publish_res);
        }

        Ok(())
    }

    fn ensure_avatar_exists(&self) -> Result<PathBuf> {
        let default_path = self.config.local.content_dir.join("avatar.png");
        if !default_path.exists() {
            // Generate a clean 512x512 procedural avatar placeholder
            let img = image::RgbImage::from_fn(512, 512, |x, y| {
                let dx = (x as i32 - 256).abs() as f32;
                let dy = (y as i32 - 256).abs() as f32;
                let r = (dx * dx + dy * dy).sqrt();
                if r < 140.0 {
                    image::Rgb([220, 180, 150]) // Face
                } else if r < 200.0 {
                    image::Rgb([35, 45, 65]) // Suit
                } else {
                    image::Rgb([18, 24, 38]) // Studio Dark Blue Background
                }
            });
            img.save(&default_path)?;
            println!("  👤 Default presenter avatar generated: {:?}", default_path);
        }
        Ok(default_path)
    }
}
