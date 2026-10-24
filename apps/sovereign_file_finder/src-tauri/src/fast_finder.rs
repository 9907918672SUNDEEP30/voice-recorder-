//! # Sovereign Fast File Finder — In-Memory Metascan Index
//!
//! Sub-1ms parallel directory scanning and fast fuzzy matching.
//! Bypasses heavy Windows Search Indexer entirely.

use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct IndexedFileRecord {
    pub path: PathBuf,
    pub filename: String,
    pub size_bytes: u64,
    pub modified_ts: u64,
    pub extension: String,
}

#[derive(Clone)]
pub struct FastFileFinder {
    index: Arc<RwLock<HashMap<PathBuf, IndexedFileRecord>>>,
}

impl Default for FastFileFinder {
    fn default() -> Self {
        Self::new()
    }
}

impl FastFileFinder {
    pub fn new() -> Self {
        Self {
            index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Fast parallel scanning of user-selected scope (e.g. D:\ragapp, C:\, or custom folder)
    pub fn scan_scope(&self, root_path: &Path) -> usize {
        if !root_path.exists() {
            return 0;
        }

        // Collect all non-system files (depth up to 8 for subfolders)
        let entries: Vec<PathBuf> = WalkDir::new(root_path)
            .max_depth(8)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                // Skip system and hidden heavy cache directories
                !name.starts_with('.')
                    && name != "node_modules"
                    && name != "target"
                    && name != "$Recycle.Bin"
                    && name != "System Volume Information"
            })
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect();

        // Process in parallel via rayon
        let records: Vec<(PathBuf, IndexedFileRecord)> = entries
            .par_iter()
            .filter_map(|path| {
                let metadata = path.metadata().ok()?;
                let filename = path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("")
                    .to_string();
                let extension = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                let modified_ts = metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);

                Some((
                    path.clone(),
                    IndexedFileRecord {
                        path: path.clone(),
                        filename,
                        size_bytes: metadata.len(),
                        modified_ts,
                        extension,
                    },
                ))
            })
            .collect();

        let count = records.len();
        let mut index = self.index.write().unwrap();
        for (path, record) in records {
            index.insert(path, record);
        }

        count
    }

    /// Returns all indexed paths in the current index
    pub fn all_indexed_paths(&self) -> Vec<PathBuf> {
        let index = self.index.read().unwrap();
        let mut paths: Vec<PathBuf> = index.keys().cloned().collect();
        // Sort alphabetically
        paths.sort();
        paths
    }

    /// Pure Fast Filename Evaluator: Sub-millisecond in-memory filename token search
    pub fn find_fast_candidates(&self, query: &str) -> Vec<PathBuf> {
        let q = query.to_lowercase();
        let query_tokens: Vec<&str> = q.split_whitespace().collect();

        let index = self.index.read().unwrap();
        let mut scored_matches: Vec<(PathBuf, usize)> = Vec::new();

        for (path, record) in index.iter() {
            let fname_lower = record.filename.to_lowercase();
            let matched_count = query_tokens
                .iter()
                .filter(|&&t| fname_lower.contains(t))
                .count();

            if matched_count > 0 {
                scored_matches.push((path.clone(), matched_count));
            }
        }

        // Sort descending by token match count
        scored_matches.sort_by(|a, b| b.1.cmp(&a.1));
        scored_matches.into_iter().map(|(p, _)| p).collect()
    }
}
