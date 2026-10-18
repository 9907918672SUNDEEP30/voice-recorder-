# 🧭 AI Core Framework — Master System State & Agent Index (SYSTEM_STATE.md)
> **Single Source of Truth (SSOT)** for AI Core Framework & All 8 Applications.  
> **Mandatory Agent Rule:** Read this file first. Do NOT perform repetitive codebase scans. Update this file immediately whenever changes occur.

---

## 🏛️ 1. Workspace Summary & Architecture

| Property | Specification |
|---|---|
| **Root Path** | `d:\ai_core_framework - Copy (2)` |
| **Language & Engine** | 100% Pure Native Rust (2021 Edition), Zero Python Wrappers |
| **GPU Acceleration** | DirectML / ONNX Runtime (DirectX 12) + Candle SD-Turbo Safetensors |
| **Security Layer** | AES-256-GCM Memory Encryption + `ZeroizeOnDrop` Volatile Scrubbing |
| **Licensing Layer** | `crates/drm_guard` (24h Rotating Lease + 10m Heartbeat Ping + OTA) |
| **Total Apps** | 8 Applications (Modular Workspace Members) |

---

## 📦 2. Core Crates Ledger (`crates/`)

| Crate | Path | Key Files | Core Responsibility |
|---|---|---|---|
| `ai_core` | `crates/core` | `src/models/adapters.rs`<br>`src/models/onnx_adapters.rs`<br>`src/models/candle_sd_turbo.rs` | **Central AI Inference:** Candle SD-Turbo safetensors, Silero VAD, Whisper STT, Qwen LLM/VLM, OpenVoice TTS. *Core-Engine First architecture.* |
| `drm_guard` | `crates/drm_guard` | `src/guard.rs`<br>`src/hardware.rs`<br>`src/heartbeat.rs`<br>`src/telemetry.rs`<br>`src/updater.rs` | **Security & Licensing:** SHA-256 Hardware Fingerprint (UUID+CPU+MAC), 24h rotating lease, 10m proactive heartbeat, memory zeroization, Jio-TV OTA binary updater. |
| `core_engine` | `crates/core_engine` | `src/lib.rs` | Shared pipelines, thread pools, async executor helpers. |
| `runtime_cli` | `crates/runtime_cli` | `src/lib.rs` | Shared CLI parsing, logging subscriber initializers. |
| `text_parser` | `crates/text_parser` | `src/lib.rs`<br>`src/pdf.rs`<br>`src/chunker.rs` | PDF, CSV, TXT document parsing and semantic chunking. |
| `vector_store` | `crates/vector_store` | `src/lib.rs`<br>`src/store.rs`<br>`src/session_key.rs` | In-memory cosine similarity vector index + AES-256 encrypted file vault. |

---

## 🏛️ 3. Master 8 Applications Ledger (`apps/`)

| # | App Name | Folder Path | Deployment Guide | Port | Frontend Type | DRM Status & Metrics |
|:---:|---|---|---|:---:|---|---|
| **1** | **Voice AI Agent** | `apps/voice_agent` | [DEPLOYMENT.md](file:///d:/ai_core_framework%20-%20Copy%20(2)/apps/voice_agent/DEPLOYMENT.md) | `3000` / WS | **Next.js PWA Dashboard** (Web & Mobile) | ✅ `VoiceAgentMetrics` (24h Lease, 10m Ping) |
| **2** | **CCTV Guard & NVR** | `apps/cctv_guard`<br>`apps/nvr_core` | [CCTV Guide](file:///d:/ai_core_framework%20-%20Copy%20(2)/apps/cctv_guard/DEPLOYMENT.md)<br>[NVR Guide](file:///d:/ai_core_framework%20-%20Copy%20(2)/apps/nvr_core/DEPLOYMENT.md) | `8080` | **Turnkey Black-Box Appliance** + TP-Link Web Portal | ✅ `CctvGuardMetrics` (24h Lease, 10m Wire Ping, Jio-TV OTA) |
| **3** | **DSP Audio Recorder** | `apps/dsp_recorder` | [DEPLOYMENT.md](file:///d:/ai_core_framework%20-%20Copy%20(2)/apps/dsp_recorder/DEPLOYMENT.md) | CLI / Native | **Desktop DAW UI + Native Mobile App** (Android / iOS) | ✅ `DspRecorderMetrics` (24h Lease, 10m Daemon) |
| **4** | **Video Studio** | `apps/video_studio` | [DEPLOYMENT.md](file:///d:/ai_core_framework%20-%20Copy%20(2)/apps/video_studio/DEPLOYMENT.md) | CLI / Native | **OBS-Style Desktop UI + Native Mobile App** (Android / iOS) | ✅ `VideoStudioMetrics` (24h Lease, 10m Daemon) |
| **5** | **Financial Engine** | `apps/financial_engine` | [DEPLOYMENT.md](file:///d:/ai_core_framework%20-%20Copy%20(2)/apps/financial_engine/DEPLOYMENT.md) | CLI / Web API | **Shambhavi Proprietary Wealth Dashboard + Native Mobile App** | ✅ `FinancialMetrics` (24h Lease, 10m Daemon, Deep Scans) |
| **6** | **Local RAG Engine** | `apps/local_rag_engine` | [DEPLOYMENT.md](file:///d:/ai_core_framework%20-%20Copy%20(2)/apps/local_rag_engine/DEPLOYMENT.md) | `8080` (REST) | **Enterprise Document Vault Desktop/Web UI** *(Restricted CORS)* | ✅ `RagEngineMetrics` (24h Lease, 10m Loop, AES-256 Vault) |
| **7** | **Manifestation App** | `apps/manifestation_app` | [DEPLOYMENT.md](file:///d:/ai_core_framework%20-%20Copy%20(2)/apps/manifestation_app/DEPLOYMENT.md) | `9090` | **Luxury Glassmorphism SPA + Native Mobile App** (Android / iOS) | ✅ `ManifestationMetrics` (24h Lease, 10m Daemon, SD-Turbo) |
| **8** | **Content Engine** | `apps/content_engine` | [DEPLOYMENT.md](file:///d:/ai_core_framework%20-%20Copy%20(2)/apps/content_engine/DEPLOYMENT.md) | CLI / Pipeline | **Desktop/Chrome Creator Studio Workspace** *(Hardened SCP)* | ✅ `ContentEngineMetrics` (24h Lease, Omnichannel Generator) |

---

## 🎨 4. Frontend & Client Architecture Matrix (Phase 5 Roadmap Priority)

> **Customer Presentation Layer:** Built to give users direct steering-wheel control over the underlying high-performance Rust core engines.

1. **🎙️ Voice AI Calling Agent:**
   - **Frontend:** Next.js PWA Web & Mobile Dashboard.
   - **Features:** Real-time call monitor, audio waveform, sub-300ms live transcript stream, bot persona/prompt editor, recordings playback, call minutes analytics.
2. **🛡️ CCTV Guard & NVR Core:**
   - **Frontend:** On-Premises Turnkey Black-Box Appliance (like TP-Link / Jio STB).
   - **Features:** Embedded Axum Web Server (`http://localhost:8080`), 4-step setup wizard, 4/8/16 camera RTSP grid, 10-minute wire fault / camera offline WhatsApp alerts, Jio-TV style silent OTA updates. Redacted credentials and protected machine IDs.
3. **🎛️ DSP Audio Recorder:**
   - **Frontend:** Studio DAW Desktop UI + Native Mobile App (Android / iOS).
   - **Features:** Real-time FFT spectrum analyzer, 10-band Biquad EQ curves, dB peak meters, sub-5.3ms latency, 24-bit lossless WAV export. (Mobile-first for on-the-go studio vocal recording).
4. **🎬 Video Studio:**
   - **Frontend:** OBS-Style Creator Studio Desktop UI + Native Mobile App (Android / iOS).
   - **Features:** Live webcam preview, AI cameraman PTZ auto-framing, teleprompter eye gaze correction slider, 3D LUT cinematic grading gallery. (Mobile app for YouTube creators shooting on smartphones).
5. **📈 Financial Super Engine:**
   - **Frontend:** Shambhavi Proprietary Wealth Dashboard + Native Mobile App (Android / iOS).
   - **Features:** Instant stock search bar (<10ms), Top Header Quick Ratios box (P/E, ROCE, ROE, Debt/Equity, Market Cap), algorithmic Pros & Cons cards, 10-year historical tables (Quarterly, P&L, Balance Sheet, Cash Flows), con-call management commentary mining, and deterministic valuation algorithms (DCF, Graham Number, Peter Lynch Fair Value).
6. **🗄️ Local RAG Engine:**
   - **Frontend:** Enterprise Document Search Vault Desktop / Intranet Web UI.
   - **Note:** Dedicated strictly to desktop/workstations (for hospitals, legal firms, accounting offices); no mobile app required. Drag-and-drop PDF/Excel shelf, sub-2ms BGE cosine search, original page citations, strictly locked to localhost/127.0.0.1.
7. **🧘 Manifestation Super App:**
   - **Frontend:** Luxury Glassmorphism Web SPA (`http://localhost:9090`) + Native Mobile App (Android / iOS).
   - **Features:** SD-Turbo 1-step living 3D vision board generation, Nikola Tesla 3-6-9 journal, mirror affirmation chamber with conviction scoring, 528 Hz / 432 Hz Solfeggio theta sleep meditation player.
8. **✍️ Content Engine:**
   - **Frontend:** Desktop/Chrome Creator Studio Workspace.
   - **Note:** Focused on desktop/Chrome workstation for creators; Chrome CDP trend recon radar, automated video script/blog editor, ASS karaoke subtitle previewer. Remote publishing secured via SCP JSON payload transfer.

---

## 🔒 5. Universal DRM & Security Hardening Protocol (`crates/drm_guard`)

- **Ed25519 Root Public Key:** `82d9651fdb75e85ab7e88c35440e4c7629b53b97c9fbfdf2335a6cb458af1400`
- **Fail-Closed Offline Policy:** Unconditional fail-open completely eliminated. Offline grace strictly bounded to unexpired server-signed leases (`.shambhavi_lease_<app>.json`).
- **Cryptographic Signature Payload:** `{client_id}:{machine_uuid}:{app_name}:{expires_at}:{nonce}:{session_key_hex}` verified via Ed25519.
- **Hardware Fingerprint:** `SHA-256(Motherboard UUID + CPU ID + MAC Address)`.
- **24-Hour Rotating Lease:** Every 24 hours app requests renewal; bounded offline grace up to lease expiry.
- **10-Minute Proactive Heartbeat:** Every 600s (`tokio::time::interval(Duration::from_secs(600))`) telemetry is sent.
  - Telemetry endpoint `/api/telemetry/heartbeat-10m` protected with Bearer token authentication.
- **Volatile RAM Zeroization:** On license expiry or tampering, `ZeroizeOnDrop` immediately wipes volatile AES-256 session keys from memory (0x00).
- **Anti-Clock Rollback:** `MonotonicClockGuard` detects system clock rollback.
- **Hardened OTA Updater:**
  - Enforced HTTPS downloads (caps max download to 150 MB).
  - Version attempt tracking prevents 10-minute download loop.
  - Ed25519 cryptographic signature verification on update binaries.
  - Windows atomic `.old` swap and Linux inode replacement.

---

## 🧠 6. Local AI Models Matrix (`C:\AI_Models\`)

| Path | Format | Size | Consumer App(s) |
|---|---|---|---|
| `C:\AI_Models\sd_turbo\` | Candle Safetensors (Pure Rust) | **5.16 GB** | `crates/core` (`ai_core`) -> `manifestation_app` |
| `C:\AI_Models\yolov8m.onnx` | DirectML ONNX | **103.7 MB** | `cctv_guard`, `nvr_core` |
| `C:\AI_Models\silero_vad.onnx` | DirectML ONNX | **< 2 MB** | `voice_agent` |
| `C:\AI_Models\whisper_tiny_onnx\` | DirectML ONNX (Enc, Dec, Tok) | **~150 MB** | `voice_agent` |
| `C:\AI_Models\qwen2-vl-2b\` | DirectML ONNX | **~1.5 GB** | `cctv_guard`, `voice_agent` |
| `C:\AI_Models\bge_small_embedding\` | DirectML ONNX | **~130 MB** | `local_rag_engine`, `voice_agent` |
| `C:\AI_Models\bge_reranker\` | DirectML ONNX | **~130 MB** | `voice_agent` |
| `C:\AI_Models\piper_hi\` | ONNX / WAV | **~60 MB** | `voice_agent`, `content_engine` |

---

## ⚡ 7. Instant Execution Commands

```powershell
# Verification & Compilation
cargo check --workspace                                          # Verify all crates & apps
cargo test --workspace                                           # Run all unit tests
cargo clean                                                      # Strip build artifacts to reduce disk size

# Launch Specific Applications (Hands-on Interactive Drives)
cargo run -p nvr_core -- --simulate --cameras 4 --port 8080      # TP-Link NVR Web Portal (Port 8080)
cargo run -p manifestation_app -- --port 9090                   # Manifestation Glassmorphism SPA (Port 9090)
cargo run -p voice_agent                                         # Voice AI Calling Agent
cargo run -p financial_engine                                    # Financial Super Engine (5000+ stocks in-memory)
cargo run -p dsp_recorder -- --list-devices                      # Audio DSP Studio
cargo run -p video_studio -- --virtual --frames 100              # Video Studio (60 FPS virtual camera)
cargo run -p local_rag_engine -- --port 8080                     # Local RAG Vault (Sub-2ms search)
cargo run -p content_engine -- pipeline                          # Content Automation Pipeline
```

---

## 🗺️ 8. Master Execution Roadmap

1. **चरण 0: सुरक्षा लॉकडाउन व ऑडिट सुधार (Security Lockdown & Hardening):** ✅ **100% पूर्ण**
   - SSH कमांड इंजेक्शन फिक्स (SCP JSON अपलोड + `StrictHostKeyChecking=accept-new`)।
   - Ed25519 रूट पब्लिक की क्रिप्टोग्राफिक वेरिफिकेशन (`crates/drm_guard` व Cloudflare Worker)।
   - Fail-Open लूपहोल पूरी तरह समाप्त; अनवेरिफाइड स्टार्टअप पर स्ट्रिक्ट Fail-Closed।
   - NVR Core स्टार्टअप लॉकआउट: 8-कैमरा फ्री बाइपास हटाया गया, स्टार्टअप डिफ़ॉल्ट रूप से Locked।
   - वेब पोर्टल सुरक्षा: `nvr_core` व `local_rag_engine` पर प्रतिबंधित CORS (लोकलहोस्ट ओनली), RTSP पासवर्ड मास्किंग, हार्डवेयर आईडी सुरक्षा।
   - OTA अपडेटर हार्डनिंग: केवल HTTPS, 150 MB सीमा, 10-मिनट लूप रोकथाम, Ed25519 बाइनरी सत्यापन।
   - 10-मिनट टेलीमेट्री पर Bearer टोकन प्रमाणीकरण।
2. **चरण 1: वास्तविक AI मॉडल्स एवं शुद्ध गणितीय तर्क सुधार (Production AI & Math Hardening):** ✅ **100% पूर्ण**
   - **Qwen2-VL 2B VLM (`apps/cctv_guard`):** गनमेटल पिक्सेल गिनती व नकली फॉलबैक्स पूरी तरह समाप्त। वास्तविक Qwen2-VL 2D पैच एक्सट्रैक्शन `[N, 1176]` व `grid_thw [1, 3]` इनपुट, 1536-डाइमेंशनल विज़न टोकन्स का ONNX फॉरवर्ड पास और स्पेसियल अनोमली मैपिंग।
   - **Whisper STT (`crates/core`):** 80-बिन में फ्लैट स्केलर कॉपी करने का डमी कोड हटाया गया। 400-पॉइंट पीरियोडिक Hann विंडो + 201-बिन Real DFT + 80-चैनल ट्रायंगुलर Mel-स्केल फ़िल्टरबैंक मैट्रिक्स + स्टैंडर्ड व्हिस्पर डायनामिक रेंज कंप्रेशन। बहुभाषी टोकन सपोर्ट: हिंदी (`50276`) व अंग्रेज़ी (`50259`)।
   - **DSP स्टूडियो ऑडियो इंजन (`apps/dsp_recorder`):** 1.2 पर 0.166 डिस्कंटीन्यूअस जंप खत्म करके C1-कंटीन्यूअस हाइपरबोलिक टैंजेंट (`tanh`) सॉफ्ट-सैचुरेटर लागू। 5ms कैप्चर लूप में बार-बार टोकियो रनटाइम रीक्रिएशन बंद, सिंगल परसिस्टेंट वर्कर रनटाइम।
   - **फाइनेंशियल इंजन (`apps/financial_engine`):** '0.0%' प्रमोटर प्लेज को रिस्क मानना बंद; डेसिमल-सेंसिटिव Regex। MACD रिवर्सल क्रॉसओवर प्रेसिडेंस फिक्स। स्थिर 15x TAM हटाकर डायनामिक इंडियन सेक्टर टेबल्स (IT, BFSI, Auto, Pharma, FMCG, Infra) व वास्तविक मार्केट शेयर गणना। SEBI क्वांटिटेटिव रिसर्च स्टेट्यूटरी डिस्क्लेमर जोड़ा गया।
3. **चरण 2: ऐप-वार समर्पित DEPLOYMENT.md निर्माण:** ✅ **100% पूर्ण** (सभी 8 ऐप्स में समर्पित गाइड मौजूद)।
4. **चरण 3: `SYSTEM_STATE.md` मास्टर इंडेक्स व रूल्स:** ✅ **100% पूर्ण** (Single Source of Truth सक्रिय)।
5. **चरण 4: क्लाउडफ्लेयर वर्कर + Supabase लाइव सेटअप:** ⏳ **शेड्यूल्ड**।
6. **चरण 5: हैंड्स-ऑन लाइव टेस्टिंग व ऑपरेटर सिमुलेशन:** ⏳ **यूज़र द्वारा संचालित किया जाएगा**।
7. **चरण 6: कस्टमर फ्रंटएंड, डैशबोर्ड्स व नेटिव मोबाइल ऐप्स:** ⏳ **उच्चतम प्राथमिकता (Shambhavi Wealth UI, Voice PWA, DAW UI, आदि)**।

