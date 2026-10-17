# 🎯 Minimum Viable Product (MVP) Scope
## Universal Autonomous Content & Media Engine (`apps/content_engine`)
**8th Autonomous App & Cross-App Integration — Shambhavi AI Labs**  
**Architect:** Sandeep Udasi  

---

## 1. MVP Objective

The MVP delivers a fully functional, end-to-end autonomous content creation pipeline capable of operating **100% locally on the developer machine** while remaining **architecturally pre-wired for seamless Lightning AI Cloud GPU remote execution**.

---

## 2. In-Scope for MVP

### Milestone 1: Workspace & CLI Harness
- Clean integration into root `Cargo.toml`.
- CLI commands:
  * `content_engine ingest --topic "..."` (Trend Reconnaissance)
  * `content_engine generate --topic "..."` (Omnichannel Script & Blog Generation)
  * `content_engine voice --text "..."` (Piper TTS WAV synthesis)
  * `content_engine caption --audio "..."` (Whisper-tiny ASS word captions)
  * `content_engine assemble` (Local video composite or Cloud Handshake)
  * `content_engine pipeline --topic "..."` (End-to-End Orchestrator)

### Milestone 2: Module 8A — Universal Trend Reconnaissance (Zero Paid API)
- Dynamic query expansion using local Qwen LLM.
- Public YouTube trending & competitor video title/hook extraction.
- Public Google News RSS and Reddit top-post scraper.
- Structured Trend Dossier generation (angles, hooks, tools identified).

### Milestone 3: Omnichannel Content Matrix
- Generation of:
  * 60s viral short script with visual cue markers.
  * 8-10m YouTube chaptered outline.
  * 1,200-word SEO blog article.
  * 5-7 tweet viral Twitter/X thread.

### Milestone 4: Speech & Subtitles
- Piper TTS integration for studio-grade `audio.wav`.
- Whisper-tiny ONNX word-by-word timestamps.
- ASS Karaoke-style highlighted subtitle formatting.

### Milestone 5: Remote SSH Bridge & Cloud Worker Scripts
- Native OpenSSH client wrapper for command execution and SFTP upload.
- Lightning AI remote worker script (`run_cloud_pipeline.py`) generator.
- Direct Cloud Publishing script (`cloud_publisher.py`) generator.
- Lightweight 360p preview download mechanism.
- Local fallback compositing engine for offline mode.

### Milestone 6: Video Studio Integration & DRM Guard
- `video_studio` GazeCorrector and 3D LUT integration.
- `drm_guard` integration with `ContentEngineMetrics`.

---

## 3. Out-of-Scope for Initial MVP (Post-MVP Enhancements)
- Multi-voice conversational podcasts (2 avatars debating).
- Custom 3D VR environment rendering.
- Paid social media ad manager integration.

---

## 4. MVP Verification Criteria
1. **Compilation:** `cargo check -p content_engine` and `cargo check --workspace` must exit with 0 errors.
2. **Local Execution:** A sample topic (e.g. "Artificial Intelligence") generates all 4 content formats, synthesized audio, and captions locally.
3. **Cloud Ready:** All remote scripts and SSH connection points are fully typed and tested in mock/dry-run mode.
