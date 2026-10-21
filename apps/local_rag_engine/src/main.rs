//! # Local RAG Engine Main Binary Entrypoint
//!
//! Pure Rust Local RAG Document Search, CRM, & 24-Hour Cloudflare DRM Engine.

use anyhow::Result;
use clap::Parser;
use local_rag_engine::{build_router, CrmManager, DrmGuard, LocalRagEngine};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(
    name = "local_rag_engine",
    author = "AI Core Framework Engineering Team",
    version = "0.1.0",
    about = "Pure Native Rust Local RAG Document Search, CRM, & 24-Hour Cloudflare DRM Engine"
)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Bind IP address (default 127.0.0.1 loopback)
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Explicit authorization flag required to bind to non-loopback network addresses
    #[arg(long, default_value_t = false)]
    allow_external: bool,

    /// Master API Key for REST endpoints (or auto-read from RAG_API_KEY env var)
    #[arg(long)]
    api_key: Option<String>,

    /// Directory containing local ONNX models (bge_small_embedding, qwen2-vl-2b)
    #[arg(short, long, default_value = "C:/AI_Models")]
    models_dir: PathBuf,

    /// Path to encrypted SQLite document vault database file
    #[arg(short, long, default_value = "rag_vault.db")]
    db_path: PathBuf,

    /// DRM SME License Key
    #[arg(short, long, default_value = "DEMO-SME-2026-KEY")]
    license_key: String,

    /// DRM Client ID
    #[arg(short, long, default_value = "sme_client_001")]
    client_id: String,

    /// Cloudflare Worker Licensing Endpoint URL
    #[arg(short, long)]
    worker_url: Option<String>,

    /// Local document storage directory to auto-scan & index (Configurable via RAG_STORAGE_DIR in .env or --storage-dir)
    #[arg(short = 's', long, env = "RAG_STORAGE_DIR", default_value = "D:/ragapp")]
    storage_dir: PathBuf,

    /// Watcher scan interval in seconds (Configurable via RAG_WATCHER_INTERVAL_SECS in .env or --interval)
    #[arg(long, env = "RAG_WATCHER_INTERVAL_SECS", default_value_t = 3)]
    watcher_interval_secs: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 0. Load .env file (Environment-Friendly configuration)
    let _ = dotenvy::dotenv();

    // 1. Initialize Tracing Subscriber
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "local_rag_engine=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    // H-10 Loopback Binding Guard: strictly refuse non-loopback bind without --allow-external
    let is_loopback = args.host == "127.0.0.1" || args.host == "localhost" || args.host == "::1";
    if !is_loopback && !args.allow_external {
        eprintln!("🛑 [SECURITY ERROR] Refusing to bind Local RAG Vault to external interface '{}'.", args.host);
        eprintln!("   Local RAG Engine contains confidential enterprise documents.");
        eprintln!("   To intentionally expose on LAN/WAN, you MUST pass '--allow-external'.");
        return Err(anyhow::anyhow!("Non-loopback bind prohibited without '--allow-external' flag"));
    }

    if let Some(key) = &args.api_key {
        std::env::set_var("RAG_API_KEY", key);
    }

    println!("=================================================================");
    println!("     PURE RUST LOCAL RAG DOCUMENT SEARCH & SME CRM VAULT        ");
    println!("=================================================================");
    println!(" 0% Python | Zero External SaaS | AES-256 Vault | DirectML ONNX ");
    println!(" 2026 Contextual Hybrid RAG: Dense HNSW + Okapi BM25 + RRF Fusion");
    println!("-----------------------------------------------------------------");

    // 2. Initialize DRM Guard & Hardware Binding
    let drm = Arc::new(DrmGuard::new(
        args.client_id.clone(),
        args.license_key.clone(),
        args.worker_url.clone(),
    ));

    println!(" Hardware Machine UUID: {}", drm.machine_uuid());
    println!(" Authenticating 24-Hour Lease via DRM Guard...");

    if let Err(e) = drm.perform_heartbeat().await {
        eprintln!(" DRM Initialization Warning: {}. Running in restricted mode.", e);
    } else {
        println!(" [SUCCESS] 24-Hour DRM Lease Granted. RAM Session Key Mounted.");
    }

    // Spawn 10-minute heartbeat background task
    drm.clone().spawn_heartbeat_loop();

    // 3. Initialize CRM Manager
    let crm = CrmManager::new();

    // 4. Initialize Local RAG Engine
    let models_path = if args.models_dir.exists() {
        Some(args.models_dir.as_path())
    } else {
        println!(" Notice: Models path '{:?}' not found, using deterministic embedding fallback.", args.models_dir);
        None
    };

    let engine = Arc::new(LocalRagEngine::new(
        &args.db_path,
        models_path,
        drm.clone(),
        crm,
    )?);

    // 5. Spawn Autonomous Storage Directory Watcher (Environment-friendly)
    println!(" 📂 Monitored Storage Directory: {:?}", args.storage_dir);
    println!(" ⏱️ Auto-Scan Polling Interval:  Every {} seconds", args.watcher_interval_secs);
    engine.clone().spawn_directory_watcher(args.storage_dir.clone(), args.watcher_interval_secs);

    // 6. Build Axum Web Application
    let app = build_router(engine);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    println!("-----------------------------------------------------------------");
    println!(" Web Dashboard & REST API live at: http://{}", addr);
    println!(" Encrypted Database Vault Path:    {:?}", args.db_path);
    println!(" Press CTRL+C to stop.");
    println!("=================================================================");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

