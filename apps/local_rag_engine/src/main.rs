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

    /// Bind IP address
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

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
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize Tracing Subscriber
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "local_rag_engine=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    println!("=================================================================");
    println!("     PURE RUST LOCAL RAG DOCUMENT SEARCH & SME CRM VAULT        ");
    println!("=================================================================");
    println!(" 0% Python | Zero External SaaS | AES-256 Vault | DirectML ONNX ");
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

    // 5. Build Axum Web Application
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
