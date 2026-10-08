pub mod agent_trait;
pub mod audio;
pub mod ml_bridge;
pub mod config;
pub mod models;
pub mod steps;

use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn init_engine() -> Result<()> {
    // ORT_DYLIB_PATH is no longer hardcoded here. 
    // We rely on ort's default binary management to avoid unsafe set_var 
    // and hardcoded browser version paths.
    Ok(())
}

pub fn search_and_verify_onnx(dir: &Path) -> Result<i32> {
    let mut count = 0;

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                count += search_and_verify_onnx(&path)?;
            } else if path.is_file() {
                if path.extension().and_then(|s| s.to_str()) == Some("onnx") {
                    count += 1;
                    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                    let parent_folder = path.parent()
                        .and_then(|p| p.file_name())
                        .unwrap_or_default()
                        .to_string_lossy();

                    println!("📦 मॉडल मिला [{}] -> '{}'", parent_folder, file_name);
                    print!("   └─ 🔄 मॉडल लोड टेस्ट (Auto EP Fallback): ");

                    let session_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        ml_bridge::MLBridge::create_session_with_target(&path, ml_bridge::ExecutionTarget::Auto)
                    }));

                    match session_result {
                        Ok(Ok((_session, active_hw))) => println!("✅ सफल! ({})", active_hw),
                        Ok(Err(e)) => println!("❌ लोड फेल: {}", e),
                        Err(_) => println!("❌ क्रैश प्रिवेंटेड: C++ बाउंड्री पर एरर आया!"),
                    }
                }
            }
        }
    }

    Ok(count)
}