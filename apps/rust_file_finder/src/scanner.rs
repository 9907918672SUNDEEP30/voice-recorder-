//! # Sub-Millisecond Directory Scanner
//!
//! Fast multithreaded directory crawling and in-memory indexing.
//! Bypasses heavy Windows Search Indexer entirely.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedFileRecord {
    pub path: PathBuf,
    pub filename: String,
    pub size_bytes: u64,
    pub modified_ts: u64,
    pub extension: String,
}

#[derive(Clone, Default)]
pub struct FastDirectoryScanner {
    index: Arc<RwLock<HashMap<PathBuf, IndexedFileRecord>>>,
}

impl FastDirectoryScanner {
    pub fn new() -> Self {
        Self {
            index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Parallel scan across user directory or drive letter
    pub fn scan_path(&self, root_path: &Path) -> usize {
        if !root_path.exists() {
            return 0;
        }

        let entries: Vec<PathBuf> = WalkDir::new(root_path)
            .max_depth(8)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
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

    /// Returns total indexed file count
    pub fn count(&self) -> usize {
        self.index.read().unwrap().len()
    }

    /// Sub-1ms tokenized search over indexed filenames
    pub fn search(&self, query: &str) -> Vec<IndexedFileRecord> {
        let q = query.trim().to_lowercase();
        let tokens: Vec<&str> = q.split_whitespace().collect();
        let index = self.index.read().unwrap();

        if tokens.is_empty() {
            return index.values().cloned().collect();
        }

        let mut matches: Vec<(IndexedFileRecord, usize)> = Vec::new();

        for record in index.values() {
            let fname_lower = record.filename.to_lowercase();
            let matched_count = tokens.iter().filter(|&&t| fname_lower.contains(t)).count();
            if matched_count > 0 {
                matches.push((record.clone(), matched_count));
            }
        }

        matches.sort_by(|a, b| b.1.cmp(&a.1));
        matches.into_iter().map(|(r, _)| r).collect()
    }
}
