//! # REST API Routes & Web Controller
//!
//! Provides Axum handlers for Document Ingestion, Semantic Querying, CRM Operations,
//! DRM Status & Renewal, and serves the embedded Single-Page Application (SPA).

use super::super::crm::CustomerRecord;
use super::super::engine::LocalRagEngine;
use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub fn build_router(engine: Arc<LocalRagEngine>) -> Router {
    Router::new()
        .route("/", get(serve_dashboard))
        .route("/api/health", get(health_handler))
        .route("/api/stats", get(stats_handler))
        .route("/api/license/status", get(license_status_handler))
        .route("/api/license/renew", post(license_renew_handler))
        .route("/api/documents/upload", post(upload_document_handler))
        .route("/api/documents/text", post(ingest_text_handler))
        .route("/api/documents/list", get(list_documents_handler))
        .route("/api/rag/query", post(rag_query_handler))
        .route("/api/crm/customers", get(list_customers_handler).post(upsert_customer_handler))
        .route("/api/crm/customers/:id/notes", post(add_customer_note_handler))
        .layer(CorsLayer::permissive())
        .with_state(engine)
}

// ----------------- Handlers -----------------

async fn serve_dashboard() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
}

async fn health_handler(State(engine): State<Arc<LocalRagEngine>>) -> impl IntoResponse {
    let stats = engine.get_stats().await;
    Json(json!({
        "status": "healthy",
        "service": "local_rag_engine",
        "drm_active": stats.drm_info.is_valid,
        "indexed_chunks": stats.indexed_vectors_in_ram,
    }))
}

async fn stats_handler(State(engine): State<Arc<LocalRagEngine>>) -> impl IntoResponse {
    let stats = engine.get_stats().await;
    Json(stats)
}

async fn license_status_handler(State(engine): State<Arc<LocalRagEngine>>) -> impl IntoResponse {
    let info = engine.drm().get_status_info().await;
    Json(info)
}

async fn license_renew_handler(State(engine): State<Arc<LocalRagEngine>>) -> impl IntoResponse {
    match engine.drm().perform_heartbeat().await {
        Ok(_) => {
            let info = engine.drm().get_status_info().await;
            (StatusCode::OK, Json(json!({ "success": true, "drm_info": info })))
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct TextIngestRequest {
    pub doc_id: Option<String>,
    pub text: String,
    pub category: Option<String>,
    pub customer_id: Option<String>,
}

async fn ingest_text_handler(
    State(engine): State<Arc<LocalRagEngine>>,
    Json(payload): Json<TextIngestRequest>,
) -> impl IntoResponse {
    let doc_id = payload
        .doc_id
        .unwrap_or_else(|| format!("TXT-{}", chrono::Utc::now().timestamp_millis()));
    let category = payload.category.as_deref().unwrap_or("General");

    match engine
        .ingest_text(
            &doc_id,
            &payload.text,
            category,
            payload.customer_id.as_deref(),
        )
        .await
    {
        Ok(res) => (StatusCode::OK, Json(json!({ "success": true, "result": res }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

async fn upload_document_handler(
    State(engine): State<Arc<LocalRagEngine>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut filename = "uploaded_file.bin".to_string();
    let mut file_bytes = Vec::new();
    let mut category = "General".to_string();
    let mut doc_id = format!("DOC-{}", chrono::Utc::now().timestamp_millis());
    let mut customer_id: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or_default().to_string();
        if name == "file" {
            if let Some(fname) = field.file_name() {
                filename = fname.to_string();
            }
            if let Ok(bytes) = field.bytes().await {
                file_bytes = bytes.to_vec();
            }
        } else if name == "category" {
            if let Ok(text) = field.text().await {
                category = text;
            }
        } else if name == "doc_id" {
            if let Ok(text) = field.text().await {
                if !text.trim().is_empty() {
                    doc_id = text.trim().to_string();
                }
            }
        } else if name == "customer_id" {
            if let Ok(text) = field.text().await {
                if !text.trim().is_empty() {
                    customer_id = Some(text.trim().to_string());
                }
            }
        }
    }

    if file_bytes.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": "No file uploaded or file empty." })),
        );
    }

    match engine
        .ingest_file(
            &doc_id,
            &filename,
            &file_bytes,
            &category,
            customer_id.as_deref(),
        )
        .await
    {
        Ok(res) => (StatusCode::OK, Json(json!({ "success": true, "result": res }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

async fn list_documents_handler(State(engine): State<Arc<LocalRagEngine>>) -> impl IntoResponse {
    match engine.list_documents() {
        Ok(docs) => (StatusCode::OK, Json(json!({ "success": true, "documents": docs }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub query: String,
    pub top_k: Option<usize>,
}

async fn rag_query_handler(
    State(engine): State<Arc<LocalRagEngine>>,
    Json(payload): Json<QueryRequest>,
) -> impl IntoResponse {
    let top_k = payload.top_k.unwrap_or(5);
    match engine.query(&payload.query, top_k).await {
        Ok(res) => (StatusCode::OK, Json(json!({ "success": true, "result": res }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

// ----------------- CRM Handlers -----------------

async fn list_customers_handler(State(engine): State<Arc<LocalRagEngine>>) -> impl IntoResponse {
    let customers = engine.crm().list_customers().await;
    Json(json!({ "success": true, "customers": customers }))
}

async fn upsert_customer_handler(
    State(engine): State<Arc<LocalRagEngine>>,
    Json(record): Json<CustomerRecord>,
) -> impl IntoResponse {
    engine.crm().upsert_customer(record).await;
    Json(json!({ "success": true, "message": "Customer record saved." }))
}

#[derive(Debug, Deserialize)]
pub struct NotePayload {
    pub author: String,
    pub note: String,
}

async fn add_customer_note_handler(
    State(engine): State<Arc<LocalRagEngine>>,
    Path(id): Path<String>,
    Json(payload): Json<NotePayload>,
) -> impl IntoResponse {
    let ok = engine.crm().add_note(&id, &payload.author, &payload.note).await;
    if ok {
        (StatusCode::OK, Json(json!({ "success": true })))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Customer not found." })))
    }
}

// ----------------- Embedded Institutional Dashboard -----------------

const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Local RAG Engine & SME CRM Vault</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;600;700&family=Plus+Jakarta+Sans:wght@400;500;600;700;800&display=swap" rel="stylesheet">
  <style>
    :root {
      --bg: #090d16;
      --card-bg: rgba(18, 24, 38, 0.85);
      --card-border: rgba(255, 255, 255, 0.08);
      --primary: #3b82f6;
      --primary-hover: #2563eb;
      --accent: #10b981;
      --danger: #ef4444;
      --warning: #f59e0b;
      --text: #f3f4f6;
      --text-muted: #9ca3af;
      --font-main: 'Plus Jakarta Sans', -apple-system, sans-serif;
      --font-mono: 'JetBrains Mono', monospace;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      background: radial-gradient(circle at 10% 20%, #0d1527 0%, var(--bg) 90%);
      color: var(--text);
      font-family: var(--font-main);
      min-height: 100vh;
      display: flex;
      flex-direction: column;
    }
    header {
      background: rgba(10, 15, 26, 0.9);
      backdrop-filter: blur(12px);
      border-bottom: 1px solid var(--card-border);
      padding: 16px 32px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      position: sticky;
      top: 0;
      z-index: 50;
    }
    .logo-area { display: flex; align-items: center; gap: 12px; }
    .logo-icon {
      width: 36px; height: 36px; border-radius: 8px;
      background: linear-gradient(135deg, var(--primary), #8b5cf6);
      display: flex; align-items: center; justify-content: center;
      font-weight: 800; font-size: 18px; color: #fff;
    }
    .logo-title h1 { font-size: 18px; font-weight: 700; }
    .logo-title p { font-size: 12px; color: var(--text-muted); }
    .drm-status-badge {
      display: flex; align-items: center; gap: 8px;
      padding: 6px 14px; border-radius: 9999px; font-size: 12px; font-weight: 600;
      background: rgba(16, 185, 129, 0.15); color: var(--accent); border: 1px solid rgba(16, 185, 129, 0.3);
    }
    .drm-pulse { width: 8px; height: 8px; border-radius: 50%; background: var(--accent); box-shadow: 0 0 8px var(--accent); animation: pulse 2s infinite; }
    @keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.4; } }
    
    .nav-tabs {
      display: flex; gap: 8px; padding: 12px 32px; background: rgba(15, 23, 42, 0.5);
      border-bottom: 1px solid var(--card-border);
    }
    .tab-btn {
      padding: 8px 16px; border-radius: 6px; border: none; background: transparent;
      color: var(--text-muted); font-weight: 600; font-size: 14px; cursor: pointer; transition: all 0.2s;
    }
    .tab-btn:hover { color: #fff; background: rgba(255, 255, 255, 0.05); }
    .tab-btn.active { color: #fff; background: var(--primary); }

    main { flex: 1; padding: 24px 32px; max-width: 1400px; margin: 0 auto; width: 100%; }
    .tab-pane { display: none; }
    .tab-pane.active { display: block; }

    .grid-2 { display: grid; grid-template-columns: 1fr 1fr; gap: 24px; }
    .grid-3 { display: grid; grid-template-columns: repeat(3, 1fr); gap: 20px; margin-bottom: 24px; }
    
    .card {
      background: var(--card-bg); border: 1px solid var(--card-border);
      border-radius: 12px; padding: 20px; backdrop-filter: blur(8px);
    }
    .card h2 { font-size: 16px; font-weight: 700; margin-bottom: 16px; display: flex; align-items: center; justify-content: space-between; }
    .stat-label { font-size: 12px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em; }
    .stat-val { font-size: 28px; font-weight: 800; font-family: var(--font-mono); color: #fff; margin-top: 4px; }

    .input-group { margin-bottom: 14px; }
    .input-group label { display: block; font-size: 12px; font-weight: 600; color: var(--text-muted); margin-bottom: 6px; }
    input[type="text"], input[type="number"], textarea, select {
      width: 100%; padding: 10px 14px; border-radius: 8px; border: 1px solid var(--card-border);
      background: rgba(10, 15, 26, 0.8); color: #fff; font-family: var(--font-main); font-size: 14px;
    }
    textarea { resize: vertical; min-height: 90px; }
    input:focus, textarea:focus { outline: none; border-color: var(--primary); }

    .btn {
      display: inline-flex; align-items: center; justify-content: center; gap: 8px;
      padding: 10px 20px; border-radius: 8px; font-weight: 600; font-size: 14px;
      cursor: pointer; border: none; transition: all 0.2s; background: var(--primary); color: #fff;
    }
    .btn:hover { background: var(--primary-hover); transform: translateY(-1px); }
    .btn-secondary { background: rgba(255, 255, 255, 0.1); color: #fff; }
    .btn-secondary:hover { background: rgba(255, 255, 255, 0.18); }
    .btn-outline { background: transparent; border: 1px solid var(--card-border); color: var(--text-muted); }
    .btn-outline:hover { color: #fff; border-color: rgba(255, 255, 255, 0.3); }

    .citation-box {
      background: rgba(10, 15, 26, 0.6); border: 1px solid var(--card-border);
      border-radius: 8px; padding: 12px; margin-top: 10px; font-size: 13px;
    }
    .citation-header { display: flex; justify-content: space-between; font-weight: 600; color: var(--primary); font-size: 12px; margin-bottom: 4px; }
    .tag { display: inline-block; padding: 2px 8px; border-radius: 4px; font-size: 11px; font-weight: 600; background: rgba(59, 130, 246, 0.15); color: var(--primary); }

    .customer-card {
      background: rgba(10, 15, 26, 0.7); border: 1px solid var(--card-border);
      border-radius: 8px; padding: 14px; margin-bottom: 12px;
    }
    .customer-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 6px; }
    .customer-name { font-weight: 700; font-size: 15px; }
    .customer-company { font-size: 13px; color: var(--text-muted); }

    .mono { font-family: var(--font-mono); font-size: 12px; }
  </style>
</head>
<body>

  <header>
    <div class="logo-area">
      <div class="logo-icon">R</div>
      <div class="logo-title">
        <h1>Local RAG Engine & SME Vault</h1>
        <p>100% Pure Native Rust &bull; DirectML ONNX &bull; AES-256 Vault</p>
      </div>
    </div>
    <div id="drmBadge" class="drm-status-badge">
      <div class="drm-pulse"></div>
      <span id="drmText">DRM Active (24h Lease)</span>
    </div>
  </header>

  <nav class="nav-tabs">
    <button class="tab-btn active" onclick="switchTab('search')">AI Search & Synthesis</button>
    <button class="tab-btn" onclick="switchTab('vault')">Document Ingestion Vault</button>
    <button class="tab-btn" onclick="switchTab('crm')">CRM Customer Records</button>
    <button class="tab-btn" onclick="switchTab('drm')">Hardware Node & DRM</button>
  </nav>

  <main>
    <!-- STATS BAR -->
    <div class="grid-3">
      <div class="card">
        <div class="stat-label">Encrypted Chunks in Vault</div>
        <div id="statChunks" class="stat-val">0</div>
      </div>
      <div class="card">
        <div class="stat-label">RAM Vectors Indexed</div>
        <div id="statVectors" class="stat-val">0</div>
      </div>
      <div class="card">
        <div class="stat-label">Lease Time Remaining</div>
        <div id="statLease" class="stat-val">24h 00m</div>
      </div>
    </div>

    <!-- TAB 1: AI SEARCH & SYNTHESIS -->
    <div id="pane-search" class="tab-pane active">
      <div class="grid-2">
        <div class="card">
          <h2>Semantic AI Search & Synthesis</h2>
          <div class="input-group">
            <label for="queryInput">Ask anything from local invoices, contracts, or tax notices:</label>
            <textarea id="queryInput" placeholder="e.g., What are the total invoice amounts and GST compliance requirements for Sharma Engineering?"></textarea>
          </div>
          <div style="display: flex; gap: 12px; align-items: center;">
            <button class="btn" onclick="executeSearch()">
              <span>Query Local AI</span>
            </button>
            <div id="queryLatency" class="mono" style="color: var(--text-muted); font-size: 13px;"></div>
          </div>
        </div>

        <div class="card">
          <h2>Grounded Synthesis Output</h2>
          <div id="answerBox" style="min-height: 120px; line-height: 1.6; font-size: 14px; color: #e5e7eb;">
            Ask a query on the left to synthesize factual answers using local business files.
          </div>

          <h3 style="font-size: 14px; font-weight: 700; margin-top: 20px; color: var(--text-muted);">Verified Citations</h3>
          <div id="citationsList" style="max-height: 280px; overflow-y: auto;">
            <p style="font-size: 13px; color: var(--text-muted); margin-top: 8px;">No search results yet.</p>
          </div>
        </div>
      </div>
    </div>

    <!-- TAB 2: INGESTION VAULT -->
    <div id="pane-vault" class="tab-pane">
      <div class="grid-2">
        <div class="card">
          <h2>Upload Business Document (PDF / CSV / TXT)</h2>
          <form id="uploadForm" onsubmit="handleUpload(event)">
            <div class="input-group">
              <label>Select File (Native PDF, CSV, or Plain Text):</label>
              <input type="file" id="fileInput" required>
            </div>
            <div class="input-group">
              <label>Document Category:</label>
              <select id="catSelect">
                <option value="Invoice">Invoice / Tax Bill</option>
                <option value="Contract">Legal Contract / SLA</option>
                <option value="Compliance">GST / Audit Notice</option>
                <option value="Customer_Record">Customer Intelligence</option>
                <option value="General">General Business Records</option>
              </select>
            </div>
            <div class="input-group">
              <label>Link to Customer ID (Optional):</label>
              <input type="text" id="custLinkInput" placeholder="e.g. CUST-001">
            </div>
            <button type="submit" class="btn">Encrypt & Ingest to Vault</button>
          </form>
          <div id="uploadStatus" style="margin-top: 12px; font-size: 13px;"></div>
        </div>

        <div class="card">
          <h2>Direct Plain Text Ingestion</h2>
          <div class="input-group">
            <label>Document ID:</label>
            <input type="text" id="textDocId" placeholder="e.g. DOC-MEMO-2026">
          </div>
          <div class="input-group">
            <label>Text Content:</label>
            <textarea id="textContent" placeholder="Paste notes, transcripts, or policies here..."></textarea>
          </div>
          <button class="btn btn-secondary" onclick="handleTextIngest()">Encrypt & Ingest Text</button>
          <div id="textStatus" style="margin-top: 12px; font-size: 13px;"></div>
        </div>
      </div>
    </div>

    <!-- TAB 3: CRM CUSTOMER RECORDS -->
    <div id="pane-crm" class="tab-pane">
      <div class="grid-2">
        <div class="card">
          <h2>Customer Account Records</h2>
          <div id="customerList" style="max-height: 520px; overflow-y: auto;">
            Loading customers...
          </div>
        </div>

        <div class="card">
          <h2>Create or Update Customer</h2>
          <div class="input-group">
            <label>Customer ID:</label>
            <input type="text" id="crmId" placeholder="e.g. CUST-003">
          </div>
          <div class="input-group">
            <label>Full Name:</label>
            <input type="text" id="crmName" placeholder="e.g. Ananya Patel">
          </div>
          <div class="input-group">
            <label>Company Name:</label>
            <input type="text" id="crmCompany" placeholder="e.g. Patel Logistics LLP">
          </div>
          <div class="input-group">
            <label>Email & Phone:</label>
            <div style="display: flex; gap: 10px;">
              <input type="text" id="crmEmail" placeholder="email@company.com">
              <input type="text" id="crmPhone" placeholder="+91 99999 88888">
            </div>
          </div>
          <div class="input-group">
            <label>Deal Value (INR):</label>
            <input type="number" id="crmDeal" placeholder="1500000">
          </div>
          <div class="input-group">
            <label>Tags (comma separated):</label>
            <input type="text" id="crmTags" placeholder="SME, Logistics, Premium">
          </div>
          <button class="btn" onclick="saveCustomer()">Save Customer Record</button>
        </div>
      </div>
    </div>

    <!-- TAB 4: HARDWARE NODE & DRM -->
    <div id="pane-drm" class="tab-pane">
      <div class="card" style="max-width: 800px; margin: 0 auto;">
        <h2>Node Hardware Binding & Cloudflare DRM</h2>
        <div style="margin: 20px 0;">
          <div class="input-group">
            <label>Hardware Machine UUID (Deterministic SHA-256):</label>
            <input type="text" id="nodeUuid" class="mono" readonly value="Calculating...">
          </div>
          <div class="input-group">
            <label>Client ID:</label>
            <input type="text" id="nodeClientId" class="mono" readonly value="...">
          </div>
          <div class="input-group">
            <label>License Status:</label>
            <div id="nodeStatus" style="font-weight: 700; color: var(--accent); margin-top: 4px;">ACTIVE</div>
          </div>
          <div class="input-group">
            <label>Anti-Clock-Tampering Guard:</label>
            <div style="color: var(--accent); font-size: 13px;">Monotonic clock verification active. Clock-rollback triggers instant RAM zeroization.</div>
          </div>
        </div>
        <button class="btn btn-secondary" onclick="renewDrm()">Trigger Cloudflare Heartbeat Renewal</button>
      </div>
    </div>
  </main>

  <script>
    function switchTab(name) {
      document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
      document.querySelectorAll('.tab-pane').forEach(p => p.classList.remove('active'));
      event.target.classList.add('active');
      document.getElementById('pane-' + name).classList.add('active');
    }

    async function refreshStats() {
      try {
        const res = await fetch('/api/stats');
        const data = await res.json();
        document.getElementById('statChunks').innerText = data.total_encrypted_chunks;
        document.getElementById('statVectors').innerText = data.indexed_vectors_in_ram;
        
        const remaining = data.drm_info.seconds_remaining;
        const hours = Math.floor(remaining / 3600);
        const mins = Math.floor((remaining % 3600) / 60);
        document.getElementById('statLease').innerText = `${hours}h ${mins}m`;

        document.getElementById('nodeUuid').value = data.drm_info.machine_uuid;
        document.getElementById('nodeClientId').value = data.drm_info.client_id;
        document.getElementById('nodeStatus').innerText = data.drm_info.status;
      } catch (e) {
        console.error(e);
      }
    }

    async function executeSearch() {
      const q = document.getElementById('queryInput').value.trim();
      if (!q) return;

      document.getElementById('answerBox').innerText = "Analyzing local encrypted records & synthesizing answer...";
      document.getElementById('citationsList').innerHTML = "";
      const start = performance.now();

      try {
        const res = await fetch('/api/rag/query', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ query: q, top_k: 5 })
        });
        const data = await res.json();
        if (data.success) {
          const r = data.result;
          document.getElementById('answerBox').innerText = r.answer;
          document.getElementById('queryLatency').innerText = `Latency: ${r.latency_ms} ms`;

          if (r.citations.length > 0) {
            let html = "";
            r.citations.forEach((c, idx) => {
              html += `
                <div class="citation-box">
                  <div class="citation-header">
                    <span>Source #${idx+1}: [${c.category}] ${c.doc_id}</span>
                    <span>Similarity: ${(c.score * 100).toFixed(1)}%</span>
                  </div>
                  <div>${c.content_preview}</div>
                </div>
              `;
            });
            document.getElementById('citationsList').innerHTML = html;
          } else {
            document.getElementById('citationsList').innerHTML = "<p style='font-size:13px; color:var(--text-muted);'>No matching chunks found.</p>";
          }
        } else {
          document.getElementById('answerBox').innerText = "Error: " + data.error;
        }
      } catch (e) {
        document.getElementById('answerBox').innerText = "Request failed: " + e;
      }
    }

    async function handleUpload(e) {
      e.preventDefault();
      const status = document.getElementById('uploadStatus');
      const file = document.getElementById('fileInput').files[0];
      if (!file) return;

      const fd = new FormData();
      fd.append('file', file);
      fd.append('category', document.getElementById('catSelect').value);
      const cust = document.getElementById('custLinkInput').value.trim();
      if (cust) fd.append('customer_id', cust);

      status.innerText = "Encrypting and indexing to vault...";
      try {
        const res = await fetch('/api/documents/upload', { method: 'POST', body: fd });
        const data = await res.json();
        if (data.success) {
          status.innerText = `Successfully ingested ${data.result.filename} (${data.result.chunks_created} chunks in ${data.result.duration_ms}ms)`;
          refreshStats();
          loadCustomers();
        } else {
          status.innerText = "Error: " + data.error;
        }
      } catch (err) {
        status.innerText = "Upload failed: " + err;
      }
    }

    async function handleTextIngest() {
      const status = document.getElementById('textStatus');
      const docId = document.getElementById('textDocId').value.trim();
      const text = document.getElementById('textContent').value.trim();
      if (!text) return;

      status.innerText = "Ingesting...";
      try {
        const res = await fetch('/api/documents/text', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ doc_id: docId || null, text: text, category: "Notes" })
        });
        const data = await res.json();
        if (data.success) {
          status.innerText = `Ingested direct text (${data.result.chunks_created} chunks in ${data.result.duration_ms}ms)`;
          document.getElementById('textContent').value = "";
          refreshStats();
        } else {
          status.innerText = "Error: " + data.error;
        }
      } catch (err) {
        status.innerText = "Failed: " + err;
      }
    }

    async function loadCustomers() {
      try {
        const res = await fetch('/api/crm/customers');
        const data = await res.json();
        const list = document.getElementById('customerList');
        if (data.success && data.customers.length > 0) {
          let html = "";
          data.customers.forEach(c => {
            const tags = c.tags.map(t => `<span class="tag">${t}</span>`).join(' ');
            const docs = c.associated_docs.length > 0 ? c.associated_docs.join(', ') : 'None';
            html += `
              <div class="customer-card">
                <div class="customer-header">
                  <div>
                    <div class="customer-name">${c.name} <span class="mono" style="color:var(--text-muted); font-size:12px;">(${c.id})</span></div>
                    <div class="customer-company">${c.company} &bull; ${c.email} &bull; ${c.phone}</div>
                  </div>
                  <div style="font-weight:700; color:var(--accent);">₹${c.deal_value.toLocaleString()}</div>
                </div>
                <div style="margin: 8px 0;">${tags}</div>
                <div style="font-size:12px; color:var(--text-muted);">
                  <strong>Linked Vault Docs:</strong> ${docs}
                </div>
              </div>
            `;
          });
          list.innerHTML = html;
        } else {
          list.innerHTML = "<p style='font-size:13px; color:var(--text-muted);'>No customers recorded.</p>";
        }
      } catch (e) {
        console.error(e);
      }
    }

    async function saveCustomer() {
      const record = {
        id: document.getElementById('crmId').value.trim(),
        name: document.getElementById('crmName').value.trim(),
        company: document.getElementById('crmCompany').value.trim(),
        email: document.getElementById('crmEmail').value.trim(),
        phone: document.getElementById('crmPhone').value.trim(),
        deal_value: parseFloat(document.getElementById('crmDeal').value) || 0,
        tags: document.getElementById('crmTags').value.split(',').map(s => s.trim()).filter(Boolean),
        associated_docs: [],
        notes: [],
        created_at: "",
        updated_at: ""
      };
      if (!record.id || !record.name) {
        alert("Please specify Customer ID and Name.");
        return;
      }

      await fetch('/api/crm/customers', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(record)
      });
      loadCustomers();
    }

    async function renewDrm() {
      const res = await fetch('/api/license/renew', { method: 'POST' });
      const data = await res.json();
      if (data.success) {
        alert("DRM Heartbeat successfully refreshed!");
        refreshStats();
      } else {
        alert("DRM renewal error: " + data.error);
      }
    }

    // Init
    refreshStats();
    loadCustomers();
    setInterval(refreshStats, 30000);
  </script>
</body>
</html>
"#;
