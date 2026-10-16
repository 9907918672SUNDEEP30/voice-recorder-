# 📋 Product Requirements Document (PRD)
## Native Rust Local RAG CRM & 24-Hour Cloudflare DRM Appliance (`apps/local_rag_engine`)

---

## 1. Product Vision & Commercial Positioning

### Problem Statement:
Small and Medium Enterprises (SMEs) — such as law firms, CA accounting offices, diagnostic clinics, and retail distributors — want AI to search through their thousands of confidential documents (contracts, invoices, prescriptions, ledgers). However:
1. **Data Leakage Paranoia:** They cannot upload sensitive customer files to public cloud LLMs (OpenAI, Anthropic, Google Cloud) due to GDPR, HIPAA, and client confidentiality contracts.
2. **Heavy Python Fragility:** Existing local solutions (LangChain, Ollama, Python ChromaDB) require complex Python environments, crash easily, and consume excessive RAM (> 4 GB).
3. **Software Piracy for Vendors:** Independent software vendors (ISVs) cannot sell local software safely because buyers can easily copy the software or cancel their monthly subscriptions while continuing to use the local tool.

### Our Solution:
A **Pure Rust Native Desktop/Server Appliance** that runs 100% locally on standard office computers with GPU acceleration (DirectML), provides lightning-fast search (< 150ms), encrypts all data with AES-256-GCM, and strictly enforces a **24-hour Cloudflare DRM subscription lock**. If the client doesn't pay the monthly SaaS fee, the encryption key is zeroized and access is revoked.

---

## 2. Commercial Pricing & Target Personas

### Commercial Subscription Model:
- **Starter (Single PC):** ₹1,999 / month (Up to 25,000 document chunks, 1 hardware machine binding).
- **Professional (Small Office LAN):** ₹4,999 / month (Up to 150,000 document chunks, 5 concurrent staff users).
- **Enterprise Turnkey Box:** ₹29,999 one-time hardware kit + ₹2,499 / month recurring software license.

### Target Personas:
1. **Chartered Accountants (CA) & Tax Consultants:** Instantly searches across 5 years of GST notices, income tax rulings, and client audit trails.
2. **Advocates & Legal Practices:** Searches across thousands of case files, FIRs, bail orders, and precedents with exact paragraph citation.
3. **Diagnostic Centers & Polyclinics:** Query patient history and medical reports without violating medical privacy regulations.
4. **Wholesale Traders & Distributors:** Instant invoice lookup by item name, party name, or invoice number.

---

## 3. Detailed Functional Requirements

### FR-1: Document Ingestion & Sliding Window Chunking
- Ingest PDFs using native `lopdf` (zero external C-binaries).
- Parse streaming CSVs (accounting ledgers, customer records).
- Chunk text using a sliding window of **300 to 500 words** with **50 words overlap** to preserve semantic continuity across paragraph boundaries.
- Tag each chunk with document ID, file path, page number, and category metadata.

### FR-2: DirectML GPU Embeddings Pipeline
- Generate dense 384-dimensional vector embeddings using local `bge_small_embedding` ONNX model.
- Leverage **DirectML DirectX 12 Execution Provider** (`ort`) to utilize any Windows GPU (NVIDIA, AMD Radeon, Intel Arc/Iris Xe).
- Fallback gracefully to multi-threaded CPU SIMD if no GPU is detected.

### FR-3: In-Memory Cosine Similarity Vector Index
- In-memory vector index capable of indexing 100,000 document chunks.
- Sub-2ms Top-K cosine similarity retrieval using SIMD vector instructions.
- Returns top-5 relevant chunks with exact match score (0.0 to 1.0) and source citations.

### FR-4: Context-Injected Local LLM Synthesis
- Injects retrieved chunks into local quantized LLM prompt (`Qwen2-VL` / `Qwen 2.5`).
- Strictly enforces groundedness ("Answer ONLY from the provided business context. If not found, reply that it is not present in local records").
- Zero hallucination guarantee.

### FR-5: AES-256-GCM Encrypted Persistence
- All chunks and raw text stored in SQLite on disk MUST be encrypted with AES-256-GCM with a unique 96-bit nonce per record.
- The 256-bit encryption key MUST NOT be stored on disk. It exists only in volatile RAM as a temporary 24-hour session key.

### FR-6: 24-Hour Cloudflare DRM, OTA Auto-Updater & Metered Telemetry
- Background Tokio task pings Cloudflare Worker API every 24 hours.
- Machine fingerprinting: `machine_uuid = SHA256(Motherboard UUID + CPU Processor ID + MAC)`.
- Validates active payment status (`paid_until >= now`).
- In case of non-payment or expired lease:
  - RAM session key is explicitly overwritten with zeroes using `zeroize`.
  - In-memory vector database is purged.
  - UI locks and displays renewal payment link/QR code.
- **Over-The-Air (OTA) Updates:** Transparent background binary updates and atomic swapping.
- **Metered Usage Telemetry:** Reports `RagEngineMetrics` (`query_count`, `chunk_count`) during each 24-hour lease renewal.


### FR-7: Anti-Tamper & Anti-Clock-Rollback
- Tracks monotonic hardware clock (`std::time::Instant`).
- Rejects system clock rollbacks or manual date manipulation to extend expired leases.
- Binds encryption keys strictly to the client machine UUID (keys cannot be copied to another computer).

---

## 4. Non-Functional Requirements & Performance SLAs

| Parameter | Specification | Compliance Reality |
| :--- | :--- | :--- |
| **Language & Runtime** | 100% Pure Rust (0% Python) | Rust 2021 Edition |
| **Vector Search Latency** | $< 5\text{ ms}$ | **$< 2\text{ ms}$ (In-Memory SIMD)** |
| **End-to-End Query Latency** | $< 250\text{ ms}$ | **$< 150\text{ ms}$ (DirectML GPU)** |
| **RAM Ceiling** | $< 500\text{ MB}$ | **$< 350\text{ MB}$ (50,000 document chunks)** |
| **Encryption Standard** | AES-256-GCM | NIST SP 800-38D Compliant |
| **Tamper Resistance** | Hardware Bound + Monotonic Clock | Zero Shared License Keys |
