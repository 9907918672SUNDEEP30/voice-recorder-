//! # Sovereign Fast File Finder — Tauri v2 Application Library
//!
//! Open Source & 100% Free for Everyone.
//! Powered by Sandeep Ji's Tier-0 Rust Fast Finder & Smart In-Memory Alias.

pub mod fast_finder;
pub mod ipc;
pub mod smart_alias;

use fast_finder::FastFileFinder;
use ipc::{
    auto_rename_file, list_connected_drives, open_file, open_file_folder, scan_directory,
    search_files, AppEngineState,
};
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize Fast File Finder Engine in RAM
    let fast_finder = Arc::new(FastFileFinder::new());

    let engine_state = AppEngineState { fast_finder };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(engine_state)
        .invoke_handler(tauri::generate_handler![
            list_connected_drives,
            scan_directory,
            search_files,
            open_file,
            open_file_folder,
            auto_rename_file
        ])
        .run(tauri::generate_context!())
        .expect("Failed to launch Sovereign Fast File Finder");
}
