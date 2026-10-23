//! # CDM Engine Main Binary Entrypoint
//!
//! Sandeep Ji's Deterministic Context-Data Matrix (CDM) Engine
//! 20-25 Year Lifespan • Zero Cloud • Single Model Qwen2-VL-2B • Anti-Tamper Secured

use anyhow::Result;
use cdm_engine::{CdmQueryEngine, ClientWorkspace, SecurityManager, SourceDocsWatcher};
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(
    name = "cdm_engine",
    author = "Sandeep Ji & AI Core Framework Engineering Team",
    version = "1.0.0",
    about = "Deterministic Context-Data Matrix (CDM) Engine - Zero RAG, Single Model Qwen2-VL-2B, 25-Year Lifespan"
)]
struct Args {
    /// Client workspace directory path containing source_docs/
    #[arg(short, long, default_value = "D:/Client_Sandeep_Kumar")]
    workspace_dir: PathBuf,

    /// Query string to run in deterministic microsecond mode
    #[arg(short, long)]
    query: Option<String>,

    /// Autonomous background watcher poll interval in milliseconds
    #[arg(long, default_value_t = 1500)]
    poll_ms: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Mandatory Pre-Flight Anti-Debugging Enforcement
    SecurityManager::enforce_anti_debugging();

    let args = Args::parse();

    println!("==================================================================");
    println!("     SHAMBHAVI SOVEREIGN CDM MATRIX ENGINE (ZERO-RAG CORE)        ");
    println!("==================================================================");
    println!(" Zero Cloud | Zero RAG | Single Model Qwen2-VL-2B | 25-Year Lifespan");
    println!(" Hardware-Locked: Motherboard + CPU ID + Disk GUID Bound          ");
    println!(" Storage: System-Hidden Encrypted SQLite + Immutable Audit Log    ");
    println!("------------------------------------------------------------------");

    // 2. Open / Mount Client Workspace
    let workspace = Arc::new(ClientWorkspace::open(&args.workspace_dir)?);

    // 3. If Query mode requested, execute deterministic 0.001 ms lookup
    if let Some(q) = args.query {
        println!("🔍 [DETERMINISTIC SQL QUERY]: \"{}\"", q);
        let conn = workspace.db_conn.lock().unwrap();
        let results = CdmQueryEngine::query(&conn, &q)?;

        if results.is_empty() {
            println!("⚠️ No matching relational records found in CDM matrix.");
        } else {
            println!("✅ Found {} exact ground-truth records:", results.len());
            for (idx, r) in results.iter().enumerate() {
                println!(
                    "  {}. [{}] Page {}: {} = {} (Ref: {}) [Latency: {} μs]",
                    idx + 1,
                    r.filename,
                    r.page_number,
                    r.particulars,
                    r.numeric_amount.map(|n| format!("₹{:.2}", n)).unwrap_or_else(|| r.verbatim_text.clone()),
                    r.section_code.as_deref().unwrap_or("General"),
                    r.latency_us
                );
            }
        }
        return Ok(());
    }

    // 4. Autonomous Background Watcher Daemon
    let extractor = Arc::new(cdm_engine::VisionExtractor::new());
    let watcher = SourceDocsWatcher::new(workspace.clone(), extractor);
    println!("🚀 [DAEMON STARTED] Watching for incoming PDFs in: {:?}", workspace.source_docs_dir);
    println!("   Drop files into 'source_docs/' to automatically ingest into Context-Data Matrix.");

    watcher.start_polling_loop().await?;

    Ok(())
}
