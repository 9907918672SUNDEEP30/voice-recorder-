use anyhow::Result;
use ort::session::Session;
use std::path::Path;
use crate::config::AppConfig;

pub struct MLBridge {
    pub vad_session: Session,
    pub llm_session: Session,
    pub vision_session: Session,
    pub embedding_session: Session,
    pub stt_encoder_session: Session,
    pub tts_session: Session,
    pub classifier_session: Session,
}

impl MLBridge {
    /// `AppConfig` का उपयोग करके सभी एआई मॉडल्स को DirectML (GPU) पर लोड करने के लिए
    pub fn new(config: &AppConfig) -> Result<Self> {
        println!("🚀 ML Bridge: सभी AI मॉडल्स को DirectML (GPU) पर इनिशियलाइज किया जा रहा है...");

        let vad_session = Self::create_session(&config.silero_vad_model)?;
        let llm_session = Self::create_session(&config.llm_model)?;
        let vision_session = Self::create_session(&config.vision_model)?;
        let embedding_session = Self::create_session(&config.embedding_model)?;
        let stt_encoder_session = Self::create_session(&config.stt_encoder)?;
        let tts_session = Self::create_session(&config.tts_model)?;
        let classifier_session = Self::create_session(&config.classifier_model)?;

        println!("✅ सफलता: सभी मॉडल्स DirectML (GPU) पर सफलतापूर्वक बाइंड हो गए हैं!");

        Ok(Self {
            vad_session,
            llm_session,
            vision_session,
            embedding_session,
            stt_encoder_session,
            tts_session,
            classifier_session,
        })
    }

    /// किसी भी ONNX मॉडल पाथ से सेशन बनाने का कोर फंक्शन (.map_err के साथ)
    pub fn create_session<P: AsRef<Path>>(model_path: P) -> Result<Session> {
        let path = model_path.as_ref();
        println!("📂 मॉडल फाइल लोड हो रही है: {:?}", path);

        let mut builder = Session::builder()
            .map_err(|e| anyhow::anyhow!("Session builder error: {}", e))?
            .with_execution_providers([
                ort::ep::DirectML::default().into(),
            ])
            .map_err(|e| anyhow::anyhow!("Failed to set DirectML execution provider: {}", e))?;

        let session = builder
            .commit_from_file(path)
            .map_err(|e| anyhow::anyhow!("Failed to commit model from file: {}", e))?;

        Ok(session)
    }
}