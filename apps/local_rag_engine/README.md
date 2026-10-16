# 🧠 Native Rust Local RAG CRM & 24-Hour Cloudflare DRM Engine
### High-Performance, Zero-Python Local Document Intelligence & SME Customer Relationship Management Appliance

---

## 📌 Executive Overview

The **Local RAG CRM & DRM Appliance** (`apps/local_rag_engine`) is an institutional-grade, zero-cloud-leakage, 100% native Rust intelligence system engineered specifically for local businesses, clinics, legal firms, accounting practices, and retail SMEs.

It bridges fast local semantic retrieval with strict commercial subscription locking:
1. **0% Python, 100% Core-Engine First:** Operates entirely in pure Rust with DirectML GPU acceleration and ONNX Runtime (`ort`).
2. **Local Privacy Guarantee:** Business documents (invoices, client contracts, GST records, medical files) never leave the local machine.
3. **AES-256-GCM Encrypted Vector Database:** Document chunks and high-dimensional vector embeddings are encrypted at rest with AES-256-GCM using an ephemeral 24-hour session key.
4. **24-Hour Cloudflare DRM & Hardware Binding:** The app pings a Cloudflare Worker every 24 hours with a cryptographically bound `machine_uuid` (Motherboard + CPU ID). If the monthly subscription is valid, an Ed25519-signed session key is issued. If expired, memory is zeroized immediately and vector search is locked.

---

## 🏗️ Architecture & Crate Hierarchy

```text
ai_core_framework/
├── crates/
│   ├── text_parser/       -> Fast PDF, CSV, and text document sliding-window chunker
│   ├── vector_store/      -> AES-256-GCM encrypted vector database with SIMD cosine search
│   └── core_engine/       -> DirectML GPU acceleration and ONNX Runtime inference
└── apps/
    └── local_rag_engine/  -> Axum REST API, embedded web UI, and 24-hour Cloudflare DRM
```

| Module / Crate | Role & Technical Capabilities |
| :--- | :--- |
| **`crates/text_parser`** | Sliding-window text chunker (300–500 words with 50-word overlap); native PDF text extraction (`lopdf`); streaming CSV parser; metadata tagger. |
| **`crates/vector_store`** | High-performance in-memory cosine similarity indexing; persistent SQLite document vault; AES-256-GCM encryption with volatile RAM session keys. |
| **`crates/core_engine`** | DirectML GPU-accelerated ONNX Runtime adapter driving `bge-small` embeddings and local Qwen LLM contextual reasoning. |
| **`apps/local_rag_engine`** | Multi-threaded Axum web server (`localhost:3000`); SME CRM interaction log; 24-hour background Cloudflare DRM heartbeat daemon. |

---

## 🚀 Quickstart & Verification

### 1. Verification of Workspace Compilation
```powershell
cargo check -p local_rag_engine
cargo check --workspace
```

### 2. Run Comprehensive Test Suite
```powershell
cargo test -p text_parser
cargo test -p vector_store
cargo test -p local_rag_engine
```

### 3. Launch Local Server & Diagnostic Portal
```powershell
cargo run -p local_rag_engine -- --port 3000 --models-dir "C:\AI_Models"
```
The embedded diagnostic portal and REST API will be accessible at `http://localhost:3000`.
