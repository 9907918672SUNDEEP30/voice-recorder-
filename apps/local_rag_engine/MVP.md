# 🚀 Minimum Viable Product (MVP) Roadmap
## Native Rust Local RAG CRM & DRM Appliance (`apps/local_rag_engine`)

---

## 📌 Executive MVP Scope

The **Minimum Viable Product (MVP)** delivers a production-grade, self-hosted document search and customer interaction intelligence engine operating under 150ms on standard Windows office PCs, backed by a 24-hour Cloudflare cryptographic lease.

---

## 🗺️ Multi-Phase Implementation Roadmap

```mermaid
gantt
    title Local RAG CRM & DRM MVP Implementation Timeline
    dateFormat  YYYY-MM-DD
    section Phase 1: Chunker & Vector Vault
    crates/text_parser (PDF/CSV sliding window chunker) :active, p1_1, 2026-10-17, 1d
    crates/vector_store (AES-256-GCM + SQLite + SIMD)   :p1_2, after p1_1, 1d
    section Phase 2: DirectML GPU AI
    crates/core_engine (DirectML BGE Embedder & LLM)    :p2_1, after p1_2, 1d
    Top-K Cosine Retrieval Pipeline                     :p2_2, after p2_1, 1d
    section Phase 3: Cloudflare DRM
    Hardware Machine Binding (Motherboard+CPU SHA256)   :p3_1, after p2_2, 1d
    24-Hour Heartbeat Tokio Worker & Memory Zeroize     :p3_2, after p3_1, 1d
    Cloudflare Worker Licensing Script                  :p3_3, after p3_2, 1d
    section Phase 4: Server & Web UI
    apps/local_rag_engine (Axum REST API & Embedded UI) :p4_1, after p3_3, 1d
    End-to-End Simulation & Verification Audit         :p4_2, after p4_1, 1d
```

---

## 🧪 MVP Verification & Test Matrix

| Component | Target Verification | Expected Pass Criteria |
| :--- | :--- | :--- |
| **Document Chunker** | Parse 10-page business PDF and 1,000-row CSV | Chunks created in 300–500 word windows with 50-word overlap; zero data loss. |
| **AES-256-GCM Store** | Encrypt 10,000 text chunks with 32-byte key | Plaintext unreadable in SQLite; authenticates and decrypts only with valid session key. |
| **DirectML GPU Pipeline** | Generate 384-dim embeddings on GPU | DirectX 12 adapter initialized; $< 3\text{ ms}$ per chunk embedding latency. |
| **In-Memory Cosine Index** | Query across 50,000 vectors | Top-5 relevant matches returned in $< 2\text{ ms}$ with exact source citations. |
| **24-Hour DRM Lockout** | Expired lease / tampered clock simulation | Session key zeroized from RAM; vector search locked; renewal QR code displayed. |
| **Axum Web Server** | Concurrent REST API query testing | 100 concurrent requests served in $< 150\text{ ms}$ average response time. |

---

## 🏁 MVP Launch Acceptance Criteria
1. `cargo check --workspace` must exit with **0 errors and 0 warnings**.
2. All unit tests across `text_parser`, `vector_store`, and `local_rag_engine` must pass with **100% green status**.
3. Memory consumption must strictly stay below **350 MB RAM** during 10,000-document ingestion.
4. The system must operate with **0% Python** and **zero cloud API dependencies** for local document intelligence.
