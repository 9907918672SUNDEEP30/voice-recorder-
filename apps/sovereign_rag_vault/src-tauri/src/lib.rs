//! # Sovereign RAG Vault — Tauri v2 Application Library
//!
//! Powered by Sandeep Ji's 3-Tier Reborn Super-RAG Architecture:
//! - Tier-0: Rust Fast File Finder & In-Memory Page Locator (<1ms)
//! - Tier-1: BGE-Micro 384-d Semantic Vector Matcher (C:\AI_Models\bge_small_embedding)
//! - Tier-2: Qwen2-VL Visual Whole-Page Synthesizer (C:\AI_Models\qwen2-vl-2b)
//! - Hardware-Bound Anti-Tamper & Zeroize DRM Guard

pub mod anti_tamper;
pub mod bge_micro;
pub mod fast_finder;
pub mod ipc;
pub mod security;
pub mod smart_alias;

use bge_micro::BgeMicroEngine;
use fast_finder::FastFileFinder;
use ipc::{
    auto_rename_file, execute_hybrid_search, get_hardware_status, list_connected_drives,
    open_file_folder, AppEngineState,
};
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 1. Initial Anti-Tamper & Hardware Integrity Verification
    if let Err(e) = anti_tamper::verify_hardware_and_tamper() {
        eprintln!("🛑 [CRITICAL HARDWARE TAMPER]: {}", e);
        anti_tamper::neutralize_and_terminate();
    }

    // 2. Initialize Fast File Finder & BGE-Micro Semantic Engines
    let fast_finder = Arc::new(FastFileFinder::new());
    let bge_micro = Arc::new(BgeMicroEngine::new());

    let engine_state = AppEngineState {
        fast_finder,
        bge_micro,
    };

    // 3. Build & Run Tauri Application
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(engine_state)
        .invoke_handler(tauri::generate_handler![
            get_hardware_status,
            list_connected_drives,
            execute_hybrid_search,
            open_file_folder,
            auto_rename_file
        ])
        .run(tauri::generate_context!())
        .expect("Failed to launch Shambhavi Sovereign RAG Vault");
}
