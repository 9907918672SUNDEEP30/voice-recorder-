# 🏛️ AI Core Framework (Universal Multi-App Autonomous AI Stack)
## Pure Native Rust Architecture · Shambhavi AI Labs
**Architect:** Sandeep Udasi  
**Standard:** Enterprise Production Tier (Zero Dummy, Zero Shortcuts, Fail-Closed Security)

---

## 📦 Workspace Overview

The `ai_core_framework` is a unified, high-performance monorepo hosting 6 core foundation crates, 10 autonomous commercial applications, and 3 sovereign cross-platform desktop clients.

```
shambhavi_workspace/
├── ai_core_framework/
│   ├── crates/
│   │   ├── core/               # Shared ONNX Runtime, Whisper STT, Qwen LLM/VLM, Silero VAD, SD-Turbo
│   │   ├── core_engine/        # Pipelines, concurrency utilities, task execution pools
│   │   ├── drm_guard/          # 24h rotating Ed25519 lease, 10m heartbeat, Jio-TV OTA updater
│   │   ├── runtime_cli/        # CLI logging and argument parsers
│   │   ├── text_parser/        # PDF extraction, CSV/TXT parsers, recursive chunker
│   │   └── vector_store/       # Pure Rust HNSW graph index (<3ms top-k) + AES-256 SQLite vault
│   │
│   ├── apps/
│   │   ├── voice_agent/        # Native RFC 3261 SIP/RTP Voice AI + Supabase Cloud CRM (5 tables)
│   │   ├── cctv_guard/         # Turnkey edge security appliance + Qwen2-VL calibrated threats
│   │   ├── nvr_core/           # Black-box NVR appliance + TP-Link Web Portal (Port 8080)
│   │   ├── dsp_recorder/       # 40-band ISO 266 vocal mastering studio + sub-5.3ms DAW engine
│   │   ├── video_studio/       # OBS-style creator studio + teleprompter eye gaze correction
│   │   ├── financial_engine/   # 60s autonomous super-analyst, in-memory stream, S-curve TAM
│   │   ├── local_rag_engine/   # Localhost document search vault (<3ms HNSW over 500k chunks)
│   │   ├── manifestation_app/  # Luxury Glassmorphism SPA (Port 9090) + vocal conviction diagnosis
│   │   ├── content_engine/     # Autonomous media studio + Supabase Media Matrix (3 tables)
│   │   ├── cdm_engine/         # 25-Year Zero-RAG 2-stage automata engine + single-file SQLite
│   │   └── rust_file_finder/   # Sub-millisecond parallel drive file search (<1ms)
│   │
│   └── config.json             # Central configuration (AI models, Cloudflare, Supabase)
│
├── native_apps/
│   └── desktop_clients/
│       ├── sovereign_file_finder/  # App 1: Desktop Client (Tauri v2 + React 19 + Rayon)
│       ├── sovereign_rag_vault/    # App 2: Desktop Client (Tauri v2 + React 19 + Qwen2-VL)
│       └── sovereign_cdm_vault/    # App 3: Desktop Client (Tauri v2 + React 19 + 25-Year Matrix)
│
└── .github/workflows/
    ├── build-file-finder.yml   # Multi-OS Rolling Latest CI/CD (Windows/macOS/Linux)
    ├── build-rag-vault.yml     # Multi-OS Rolling Latest CI/CD (Windows/macOS/Linux)
    └── build-cdm-vault.yml     # Multi-OS Rolling Latest CI/CD (Windows/macOS/Linux)
```

---

## 🔒 Enterprise DRM & Cloud Persistence Infrastructure

1. **Cloudflare Edge DRM Gateway:**
   - Worker: `https://shambhavi-drm-guard.shambhaviailabs.workers.dev`
   - D1 Database: `shambhavi-drm-db` (APAC Region)
   - KV Cache: `DRM_KV`
   - 24-Hour Rotating Cryptographic Lease (`.shambhavi_lease_<app>.json`) + 10-Minute Health Heartbeat.

2. **Supabase Cloud CRM (Voice AI Agent):**
   - Project: `kbrfcrhioqdnhznkpseb` (`https://kbrfcrhioqdnhznkpseb.supabase.co`)
   - Tables: `leads`, `call_logs`, `scheduled_callbacks`, `meeting_bookings`, `action_tickets`.

3. **Supabase Cloud Media Matrix (Content Engine):**
   - Project: `hbnwmjrrhvpibtqsucqz` (`https://hbnwmjrrhvpibtqsucqz.supabase.co`)
   - Tables: `content_pieces`, `publishing_logs`, `trend_dossiers`.

---

## 🚀 Multi-OS CI/CD & Silent In-App Auto-Update (Option A: Rolling Latest)

- **Target Platforms:**
  - **Windows:** `.exe` (NSIS Easy Installer) + `.msi` (WiX Enterprise Installer)
  - **macOS:** Universal `.dmg` (Apple Silicon M1/M2/M3 + Intel x86_64)
  - **Linux:** `.AppImage` (Self-contained executable) + `.deb` (Debian/Ubuntu)
- **Zero Local Machine Build Overhead:**
  - All builds run on GitHub Actions virtual machine matrix (`windows-latest`, `macos-latest`, `ubuntu-22.04`).
  - Developers push code (`git push`), and GitHub automatically compiles, signs with Ed25519 keys, and updates the release.
- **In-App Auto-Update:**
  - Desktop clients read `latest.json` on startup.
  - New builds are downloaded in the background, installed in-place, and restarted automatically with zero manual customer effort.

---

## ⚡ Instant Verification & Testing

```powershell
# 1. Compile all crates and applications
cargo check --workspace

# 2. Run universal unit test suite
cargo test --workspace

# 3. Run Live Cloudflare Edge DRM Test Suite (3/3 Pass)
cargo test --test live_edge_tests -p drm_guard -- --nocapture

# 4. Run Live Supabase Voice CRM Integration Test (1/1 Pass)
cargo test --test supabase_crm_integration -p voice_agent -- --nocapture

# 5. Run Live Supabase Content Engine Media Matrix Test (1/1 Pass)
cargo test --test supabase_content_integration -p content_engine -- --nocapture
```
