use anyhow::Result;
use clap::{Parser, Subcommand};
use content_engine::{
    ContentEngineConfig, ContentEngineOrchestrator, SubtitleEngine, TrendReconEngine,
    TtsVoiceEngine,
};

#[derive(Parser, Debug)]
#[command(
    name = "content_engine",
    version = "0.1.0",
    about = "Shambhavi AI Labs — Universal Autonomous Content & Media Engine"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Execute the complete autonomous content creation campaign
    Pipeline {
        /// Dynamic topic or breaking trend (e.g. "80s Retro AI Photo Trend")
        #[arg(short, long)]
        topic: Option<String>,
    },
    /// Perform deep trend reconnaissance and competitor analysis
    Recon {
        /// Dynamic topic to analyze
        #[arg(short, long)]
        topic: String,
    },
    /// Synthesize voiceover speech from text
    Speech {
        /// Text to synthesize
        #[arg(short, long)]
        text: String,
        /// Output WAV path
        #[arg(short, long, default_value = "output/omnichannel_content/audio.wav")]
        output: String,
    },
    /// Display current environment configuration and cloud connection status
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = ContentEngineConfig::load()?;

    match cli.command {
        Some(Commands::Pipeline { topic }) => {
            let orchestrator = ContentEngineOrchestrator::new(config);
            orchestrator.execute_campaign(topic.as_deref()).await?;
        }
        Some(Commands::Recon { topic }) => {
            let recon = TrendReconEngine::new(&config.cdp.host, config.cdp.port);
            let dossier = recon.analyze_topic(&topic).await?;
            let json = serde_json::to_string_pretty(&dossier)?;
            println!("\n📊 TREND DOSSIER RESULT:\n{}", json);
        }
        Some(Commands::Speech { text, output }) => {
            let tts = TtsVoiceEngine::new();
            let path = std::path::PathBuf::from(output);
            tts.synthesize_speech_wav(&text, &path)?;
            
            // Also generate ASS karaoke subtitles
            let duration = (text.split_whitespace().count() as f32 / 2.5).max(3.0);
            let timestamps = SubtitleEngine::align_word_timestamps(&text, duration);
            let ass_path = path.with_extension("ass");
            SubtitleEngine::export_ass_karaoke(&timestamps, &ass_path)?;
        }
        Some(Commands::Status) | None => {
            println!("===========================================================");
            println!("⚡ SHAMBHAVI AI LABS - CONTENT ENGINE ENVIRONMENT STATUS");
            println!("===========================================================");
            println!("  Execution Mode:         {:?}", config.execution_env);
            println!("  Local Models Dir:       {:?}", config.local.models_dir);
            println!("  Chrome CDP Endpoint:    http://{}:{}", config.cdp.host, config.cdp.port);
            println!("  Lightning Cloud Host:   {}", config.cloud.ssh_host);
            println!("  Cloud GPU Active:       {}", config.cloud.gpu_active);
            println!("  Direct Cloud Publish:   {}", config.cloud.direct_cloud_publish);
            println!("  Default Topic:          '{}'", config.default_topic);
            println!("  Content Output Dir:     {:?}", config.local.content_dir);
            println!("===========================================================");
            println!("Tip: Run 'cargo run -p content_engine -- pipeline' to launch.");
        }
    }

    Ok(())
}
