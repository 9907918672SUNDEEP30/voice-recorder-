use anyhow::Result;
use std::fs;
use std::path::Path;

// 1️⃣ Config module ko import karein
mod config;
use config::AppConfig;

// 2️⃣ Hamare apane local modules aur MLBridge ko import karein
mod ml_bridge;
mod models;

use ml_bridge::MLBridge;
use models::llm_vision::LlmVisionModel;
use models::embedding::EmbeddingModel;
use models::stt::SttModel;
use models::tts::TtsModel;
use models::reranker::RerankerModel;

#[tokio::main]
async fn main() -> Result<()> {
    println!("--- Initializing Local AI Agent Framework (DirectML / GPU) ---");

    // 3️⃣ Sabse pehle configuration load karein aur error check karein
    let settings = match AppConfig::load("config.json") {
        Ok(cfg) => {
            println!("✅ 'config.json' safaltapoorvak load ho gayi hai!");
            cfg
        }
        Err(e) => {
            println!("❌ 'config.json' load karne ya parse karne me error aayi: {}", e);
            println!("⚠️ Kripya check karein ki JSON file me koi syntax mistake toh nahi hai.");
            println!("⚠️ Abhi default settings ka use kiya ja raha hai...");
            AppConfig::default()
        }
    };

    // 🔍 Debug print taaki pata chale ki asli me kaunsa path load hua hai
    println!("🔍 [DEBUG] Asli loaded LLM path: {}", settings.llm_model);
    println!("📁 Models Directory: {:?}", settings.models_dir);
    println!("🤖 LLM Model File: {}", settings.llm_model);

    // 4️⃣ Check karein ki config me diya gaya models folder maujood hai ya nahi
    if !settings.models_dir.exists() {
        println!("❌ Mukhya folder nahi mila: '{:?}'", settings.models_dir);
        fs::create_dir_all(&settings.models_dir)?;
        println!("📁 Naya folder bana diya gaya hai.");
    } else {
        println!("✅ Models directory milti hai.");
    }

    // 5️⃣ Models ki directory ko scan karne ka test
    println!("\n--------------------------------------------------");
    println!("🔍 Local models ki scanning ki ja rahi hai...");
    println!("--------------------------------------------------");
    
    if let Ok(entries) = fs::read_dir(&settings.models_dir) {
        let count = entries.count();
        println!("✨ Models directory me kul {} files/items maujood hain.", count);
    }

    // 6️⃣ GPU (DirectML) par LLM model load karne ka live test
    let model_path = &settings.llm_model;
    if Path::new(model_path).exists() {
        println!("\n--------------------------------------------------");
        println!("🚀 GPU (DirectML) par model load karne ka test ho raha hai...");
        println!("--------------------------------------------------");
        
        match MLBridge::create_session(model_path) {
            Ok(_session) => {
                println!("🎉 Shandaar! Model safaltapoorvak DirectML (GPU) par load ho gaya hai.");
            }
            Err(e) => {
                println!("❌ GPU load test safal nahi ho saka: {}", e);
            }
        }
    } else {
        println!("⚠️ Dhyan dein: Config me diya gaya model path ('{}') abhi maujood nahi hai.", model_path);
        println!("(Jab aap ONNX model file yahan rakh denge, tab ye direct GPU par chalne lagega.)");
    }

    println!("\n✨ Framework puri tarah set hai!");

    Ok(())
}