//! # Smart In-Memory Virtual Alias Engine
//!
//! Maps cryptic scanned files (e.g. `SCAN_00492.pdf`) to structured human titles in RAM.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualAliasRecord {
    pub physical_path: PathBuf,
    pub original_filename: String,
    pub canonical_title: String,
    pub client_name: Option<String>,
    pub doc_type: String,
    pub year: Option<String>,
    pub suggested_filename: Option<String>,
    pub needs_renaming: bool,
}

#[derive(Clone, Default)]
pub struct SmartAliasEngine {
    cache: Arc<RwLock<HashMap<PathBuf, VirtualAliasRecord>>>,
}

impl SmartAliasEngine {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Evaluates filename or text snippet to generate virtual alias
    pub fn resolve_alias(&self, path: &Path, content_sample: Option<&str>) -> VirtualAliasRecord {
        let filename = path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("document.pdf")
            .to_string();

        let target_text = format!("{} {}", filename, content_sample.unwrap_or("")).to_lowercase();

        // 1. Client Detection
        let client_name = if target_text.contains("sandeep") {
            Some("Sandeep Kumar".to_string())
        } else if target_text.contains("rajesh") {
            Some("Rajesh Sharma".to_string())
        } else if target_text.contains("shambhavi") {
            Some("Shambhavi AI Labs".to_string())
        } else {
            None
        };

        // 2. Document Type Detection
        let doc_type = if target_text.contains("itr") || target_text.contains("income tax") {
            "Income Tax Return (ITR-V)".to_string()
        } else if target_text.contains("sale deed") || target_text.contains("plot") || target_text.contains("property") {
            "Plot Sale Deed & Capital Gains".to_string()
        } else if target_text.contains("audit") || target_text.contains("44ab") {
            "Tax Audit Report".to_string()
        } else if target_text.contains("gst") || target_text.contains("gstr") {
            "GST Compliance Statement".to_string()
        } else if target_text.contains("invoice") || target_text.contains("bill") {
            "Commercial Tax Invoice".to_string()
        } else {
            "General Document".to_string()
        };

        // 3. Year Detection
        let year = if target_text.contains("2019-20") || target_text.contains("2019") {
            Some("AY 2019-20".to_string())
        } else if target_text.contains("2024-25") || target_text.contains("2024") {
            Some("AY 2024-25".to_string())
        } else if target_text.contains("2025-26") || target_text.contains("2025") {
            Some("AY 2025-26".to_string())
        } else {
            None
        };

        let fname_lower = filename.to_lowercase();
        let is_cryptic = fname_lower.starts_with("scan")
            || fname_lower.starts_with("doc")
            || fname_lower.starts_with("img")
            || fname_lower.starts_with("untitled");

        let suggested_filename = if is_cryptic {
            let c = client_name.as_deref().unwrap_or("Client").replace(" ", "_");
            let d = doc_type.replace(" ", "_").replace("/", "_").replace("&", "and");
            let y = year.as_deref().unwrap_or("Current").replace(" ", "_");
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("pdf");
            Some(format!("{}_{}_{}_{}.{}", c, d, y, filename.trim_end_matches(&format!(".{}", ext)), ext))
        } else {
            None
        };

        let canonical_title = format!(
            "{} - {} ({})",
            client_name.as_deref().unwrap_or("Entity"),
            doc_type,
            year.as_deref().unwrap_or("General")
        );

        let record = VirtualAliasRecord {
            physical_path: path.to_path_buf(),
            original_filename: filename,
            canonical_title,
            client_name,
            doc_type,
            year,
            suggested_filename,
            needs_renaming: is_cryptic,
        };

        self.cache.write().unwrap().insert(path.to_path_buf(), record.clone());
        record
    }
}
