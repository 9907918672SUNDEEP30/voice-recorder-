pub mod agent_trait;
pub mod audio;
pub mod ml_bridge;
pub mod config;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn init_engine() -> Result<()> {
    // 🛠️ DLL का स्थायी पाथ सेट करना ताकि रनटाइम फेल न हो
    std::env::set_var(
        "ORT_DYLIB_PATH", 
        r"C:\Program Files (x86)\Microsoft\Edge\Application\152.0.4191.53\onnxruntime.dll"
    );

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
                    print!("   └─ ⚡ DirectML GPU लोड टेस्ट: ");

                  // C++ boundary crashes are avoided via a safe panic guard while registering the DirectML EP.
let session_result = std::panic::catch_unwind(|| {
    ort::session::Session::builder()?
        .with_execution_providers([
            ort::ep::DirectML::default().into(),
        ])?
        .commit_from_file(&path)
});

                    match session_result {
                        Ok(Ok(_)) => println!("✅ सफल! (Model running on GPU via DirectML)"),
                        Ok(Err(e)) => println!("❌ लोड फेल: {}", e),
                        Err(_) => println!("❌ क्रैश प्रिवेंटेड: C++ बाउंड्री पर एरर आया!"),
                    }
                }
            }
        }
    }

    Ok(count)
}