//! # 1-Click Disk Auto-Rename Engine
//!
//! Renames files safely on the file system with collision avoidance.

use std::path::{Path, PathBuf};

pub fn rename_file_on_disk(original_path: &Path, new_filename: &str) -> Result<PathBuf, String> {
    if !original_path.exists() {
        return Err("Original file does not exist".to_string());
    }

    let parent = original_path
        .parent()
        .ok_or_else(|| "Target file has no parent directory".to_string())?;

    let destination = parent.join(new_filename);
    if destination.exists() {
        return Err(format!(
            "Destination file '{}' already exists in folder",
            new_filename
        ));
    }

    std::fs::rename(original_path, &destination)
        .map_err(|e| format!("Filesystem rename error: {}", e))?;

    Ok(destination)
}
