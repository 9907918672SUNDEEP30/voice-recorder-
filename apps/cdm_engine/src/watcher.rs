//! # Autonomous Source Docs Directory Watcher
//!
//! Continuously scans `source_docs/` for incoming PDFs dropped by CAs/Lawyers.
//! Triggers Qwen2-VL-2B whole-page extraction and updates `cdm_matrix.db` & `audit_log.bin`.

use crate::audit::AuditAction;
use crate::vision_extractor::VisionExtractor;
use crate::workspace::ClientWorkspace;
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub struct SourceDocsWatcher {
    workspace: Arc<ClientWorkspace>,
    extractor: Arc<VisionExtractor>,
}

impl SourceDocsWatcher {
    pub fn new(workspace: Arc<ClientWorkspace>) -> Self {
        Self {
            workspace,
            extractor: Arc::new(VisionExtractor::new()),
        }
    }

    /// Runs autonomous watcher polling loop
    pub async fn run_loop(&self, poll_interval_ms: u64) -> Result<()> {
        let mut processed_hashes: HashSet<String> = HashSet::new();

        println!(
            "📂 [CDM WATCHER ACTIVE] Monitoring client source directory: {:?}",
            self.workspace.source_docs_dir
        );

        loop {
            sleep(Duration::from_millis(poll_interval_ms)).await;

            let mut dir_entries = tokio::fs::read_dir(&self.workspace.source_docs_dir).await?;
            while let Some(entry) = dir_entries.next_entry().await? {
                let file_path = entry.path();
                if file_path.is_file() {
                    let ext = file_path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_lowercase();

                    if ext == "pdf" || ext == "csv" || ext == "txt" {
                        let bytes = match tokio::fs::read(&file_path).await {
                            Ok(b) => b,
                            Err(_) => continue,
                        };

                        let mut hasher = Sha256::new();
                        hasher.update(&bytes);
                        let file_hash = hex::encode(hasher.finalize());

                        if !processed_hashes.contains(&file_hash) {
                            println!(
                                "🔍 [NEW CLIENT DOC DETECTED] Processing '{}' via DirectML Vision Model...",
                                file_path.display()
                            );

                            let filename = file_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("document.pdf")
                                .to_string();

                            let text_preview = String::from_utf8_lossy(&bytes[0..bytes.len().min(4000)]);
                            let output = self
                                .extractor
                                .process_whole_page(&file_path, 1, &text_preview)
                                .await?;

                            // Atomically insert into SQLite cdm_matrix.db
                            let doc_id = format!("DOC-{}", file_hash[0..8].to_uppercase());
                            {
                                let conn = self.workspace.db_conn.lock().unwrap();
                                let now = chrono::Utc::now().timestamp();

                                conn.execute(
                                    r#"
                                    INSERT OR REPLACE INTO documents_manifest 
                                    (doc_id, filename, relative_path, file_sha256, file_size_bytes, doc_type, client_name, fiscal_year, assessment_year, total_pages, ingested_at)
                                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11);
                                    "#,
                                    (
                                        &doc_id,
                                        &filename,
                                        &filename,
                                        &file_hash,
                                        bytes.len() as i64,
                                        &output.doc_type,
                                        &output.client_name,
                                        &output.fiscal_year,
                                        &output.assessment_year,
                                        1,
                                        now,
                                    ),
                                )?;

                                for (idx, item) in output.financial_items.iter().enumerate() {
                                    let item_id = format!("{}-ITEM-{}", doc_id, idx + 1);
                                    let (bx, by, bw, bh) = match item.bbox {
                                        Some([x, y, w, h]) => (Some(x), Some(y), Some(w), Some(h)),
                                        None => (None, None, None, None),
                                    };

                                    conn.execute(
                                        r#"
                                        INSERT OR REPLACE INTO financial_line_items
                                        (item_id, doc_id, page_number, schedule_name, section_code, particulars_label, numeric_value, currency, verbatim_text, bbox_x, bbox_y, bbox_width, bbox_height)
                                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13);
                                        "#,
                                        (
                                            &item_id,
                                            &doc_id,
                                            output.page_number,
                                            &item.schedule_name,
                                            &item.section_code,
                                            &item.particulars_label,
                                            item.numeric_value,
                                            &item.currency,
                                            &item.verbatim_text,
                                            bx,
                                            by,
                                            bw,
                                            bh,
                                        ),
                                    )?;
                                }
                            }

                            // Append to Immutable Audit Trail
                            self.workspace.audit_logger.append_record(
                                AuditAction::DocIngest,
                                format!("INGEST:{}:{}", doc_id, filename).as_bytes(),
                            )?;

                            println!(
                                "✅ [CDM MATRIX INGESTION SUCCESS] Structured '{}' | Financial Items: {} | Written to cdm_matrix.db",
                                filename, output.financial_items.len()
                            );

                            processed_hashes.insert(file_hash);
                        }
                    }
                }
            }
        }
    }
}
