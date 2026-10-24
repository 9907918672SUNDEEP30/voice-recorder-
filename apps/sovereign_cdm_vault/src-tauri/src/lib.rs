//! # Sovereign RAG Vault — Tauri v2 Application Library
//!
//! Powered by Sandeep Ji's Zero-RAG CDM Architecture & Pure Native Rust

pub mod anti_tamper;
pub mod cdm_matrix;
pub mod fast_finder;
pub mod ipc;
pub mod smart_alias;

use cdm_matrix::ContextDataMatrix;
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

    // 2. Initialize Fast File Finder & Context-Data Matrix Engines
    let fast_finder = Arc::new(FastFileFinder::new());
    let cdm_matrix = Arc::new(ContextDataMatrix::new("cdm_matrix.db"));

    let engine_state = AppEngineState {
        fast_finder,
        cdm_matrix,
    };

    // 3. Build & Run Tauri Application
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
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
