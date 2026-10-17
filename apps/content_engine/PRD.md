# 📜 Product Requirements Document (PRD)
## Universal Autonomous Content & Media Engine (`apps/content_engine`)
**8th Autonomous App & Cross-App Integration — Shambhavi AI Labs**  
**Architect:** Sandeep Udasi  

---

## 1. Executive Summary & Vision

The **Universal Autonomous Content & Media Engine** (`apps/content_engine`) is a production-grade, 100% niche-neutral, zero-paid-API autonomous content production and publishing engine. 

### Core Value Proposition:
Given any topic, trend, or niche (e.g. AI News, 80s Retro Yearbook Trend, Stock Market Fluctuations, Fitness, Crypto, Philosophy), the engine:
1. Conducts cross-platform deep trend reconnaissance across YouTube, Instagram Reels, TikTok, Twitter/X, and Web RSS feeds.
2. Identifies high-performing competitor videos, view-count velocity, winning title formulas, and retention hooks.
3. Reverse-engineers the exact tools, prompts, and workflows driving the viral trend.
4. Synthesizes an **Omnichannel Content Matrix**:
   - 60-Second Short / Reel / TikTok Script with psychological retention hooks.
   - 8–10 Minute Long-Form YouTube Explainer Script.
   - Comprehensive SEO-optimized Blog Article (Markdown/HTML).
   - 5–7 Tweet Viral Twitter (X) Thread.
5. Synthesizes voiceover locally using Piper/Kokoro TTS and extracts word-level ASS glowing karaoke subtitles via Whisper-tiny ONNX.
6. Seamlessly integrates with **Lightning AI Cloud GPU** to drive LivePortrait (facial animation) and CodeFormer (4K face restoration), assembling a master 4K 60fps video directly on the cloud.
7. Publishes content directly from the cloud node to YouTube / Instagram at 1–10 Gbps speeds, saving local disk space and internet bandwidth.

---

## 2. Target Persona & User Stories

### Target User:
Content creators, digital media agencies, solopreneurs, educators, and quantitative trend analysts who want to dominate multiple social platforms without manual video editing, expensive studio setups, or recurring SaaS subscriptions (Synthesia, HeyGen, InVideo, VidIQ).

### User Stories:
- **US-1 (Trend Hijacking):** *"As a creator, when an 80s photo trend or AI model breakthrough occurs, I want my engine to immediately scrape the top-performing YouTube videos and viral tweets, reverse-engineer the tools used, and write scripts before the trend dies."*
- **US-2 (Omnichannel Assets):** *"As an agency owner, for every single topic I input, I want an immediate 60s Reel script, a long-form YouTube script, an SEO blog post, and a viral Twitter thread generated in one go."*
- **US-3 (Zero-Download Cloud Publishing):** *"As a user with a modest local PC (Intel i3, AMD RX 550), I want the heavy 4K video rendering and publishing to happen directly on the cloud GPU, sending only a 5 MB preview to my local PC."*
- **US-4 (Gaze Correction):** *"As a viewer, I want the AI clone to maintain 100% natural eye contact with the camera lens without shifting pupils or robotic dead-stare."*

---

## 3. Functional Requirements

### FR-1: Dynamic Topic Ingestion & Trend Reconnaissance (Module 8A)
- Niche-neutral topic query input (CLI `--topic "..."` or config).
- Zero-paid-API public scraping:
  * YouTube Search & Competitor Hook Scraper.
  * Instagram / TikTok Trend Radar.
  * Twitter / X Viral Pulse & Debate Miner.
  * Google News RSS, Reddit JSON, arXiv API, Yahoo Finance.
- Trend Dossier generation (Summary, Top 3 Hooks, View Velocity, Extracted Tools/Prompts).

### FR-2: Omnichannel Content Generation
- Local Qwen-2.1 / Qwen2-VL ONNX model execution (`C:\AI_Models\qwen2-vl-2b`).
- Generation of 4 synchronized assets:
  1. `short_script.json` (60s with visual cues and timestamps).
  2. `long_script.md` (8-10m YouTube chaptered script).
  3. `article.md` (SEO blog article with H1-H3 and FAQ).
  4. `twitter_thread.json` (5-7 viral tweets).

### FR-3: Speech Synthesis & Audio Mastering
- Local Piper TTS / Kokoro synthesis from `C:\AI_Models\piper_hi`.
- Pure lossless 24kHz / 22.05kHz PCM generation.
- DSP mastering via `apps/dsp_recorder` RBJ Biquad EQ (vocal presence boost at 2.8kHz, 80Hz rumble cut).

### FR-4: Word-Level Auto-Captions
- Local Whisper-tiny ONNX inference (`C:\AI_Models\whisper_tiny_onnx`).
- Word-by-word timestamp extraction.
- Dynamic ASS (Advanced SubStation Alpha) glowing karaoke subtitles.

### FR-5: Cloud GPU Bridge & Direct Publishing (Lightning AI)
- SSH/SFTP lightweight asset upload (`avatar.png`, `audio.wav`, `subtitles.ass` - ~5 MB total).
- Remote trigger of LivePortrait + CodeFormer 4K + Cloud FFmpeg NVENC assembly.
- Generation and local download of optional 360p lightweight preview (~5 MB).
- Remote command trigger for direct cloud upload to YouTube Shorts and Instagram Reels.
- Local fallback mode when running with `EXECUTION_ENV=LOCAL`.

### FR-6: Video Studio Gaze & 3D LUT Integration
- Seamless import of `video_studio::gaze::GazeCorrector` and `IrisTracker`.
- 3D LUT cinematic color grading (`video_studio::grading::Lut3D`).

### FR-7: DRM Guard Licensing & Metered Usage
- Integration with `crates/drm_guard`.
- Reporting `ContentEngineMetrics` (scripts generated, articles written, videos rendered, cloud GPU seconds).

---

## 4. Non-Functional Requirements
- **Local PC Safety:** RAM usage strictly under 4–6 GB on Intel i3 10th Gen.
- **Zero Local Heavy Downloads:** 4K master files reside on cloud storage; zero disk bloat.
- **Pure Rust Scaffolding:** CLI and orchestrator compiled into a single binary with zero external runtime dependencies.
- **Zero Paid APIs:** All public trend intelligence operates without paid API keys.
