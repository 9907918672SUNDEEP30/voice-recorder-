# 📋 Product Requirements Document (PRD)
## Shambhavi Sankalpa — Manifestation Super App Engine (`apps/manifestation_app`)

---

## 1. Product Vision & Mission

### Problem Statement
Personal transformation, manifestation, and mental wellness applications in the market face severe systemic flaws:
1. **Cloud Subscriptions & Privacy Violations:** Modern mental wellness apps (Headspace, Calm, Mindvalley) store deeply vulnerable personal thoughts, dreams, and subconscious affirmations in third-party cloud databases, exposing users to data breaches and targeted advertising.
2. **Heavy, Sluggish Runtimes:** Existing AI-powered vision board and journaling apps rely on Python backends, heavy frameworks (PyTorch, Streamlit, Electron), requiring > 2.5 GB of RAM, experiencing high startup latencies (> 15 seconds), and suffering from broken dependency environments.
3. **Isolated Silos & Duplicate Engines:** Image generation and voice synthesis are frequently re-implemented from scratch in ad-hoc scripts rather than reusing a battle-tested, high-performance central AI core engine.
4. **Superficial Feature Sets:** Most apps offer static text logs or generic generic affirmations without true brainwave entrainment (Theta wave 4–7 Hz), Solfeggio acoustic resonance (528 Hz / 432 Hz), or single-step visual anchor synthesis.

### Our Solution
A **100% Pure Native Rust Binary Application** (`apps/manifestation_app`) that:
- Adheres strictly to the **"Core-Engine First" Architectural Standard**: All generative AI capabilities (text-to-image synthesis, neural speech, audio DSP) are housed centrally in `crates/core` (`ai_core`) and consumed through clean Rust traits (`ImageGenAdapter`).
- Operates at **institutional-grade sub-50ms REST latency** and **< 45 MB RAM usage**, completely free of Python, node.js, or cloud API fees.
- Delivers **Living 3D Vision Boards**, a **3-6-9 Nikola Tesla Manifestation Journal**, an interactive **Mirror Affirmation Chamber**, and a **Theta Bedtime Meditation Scheduler** through an embedded, luxury dark gold and emerald glassmorphism web portal on `http://localhost:9090`.
- Protects commercial value via **24-Hour Cryptographic Leased DRM** and **Metered Analytics** (`crates/drm_guard`).

---

## 2. Architectural Paradigm: "Core-Engine First"

A critical mandate of the AI Core Framework is that **individual applications MUST NEVER isolate proprietary AI model pipelines inside application-level folders**:
```
                           ┌────────────────────────────────────────┐
                           │              crates/core               │
                           │               (ai_core)                │
                           ├────────────────────────────────────────┤
                           │  • ImageGenAdapter Trait               │
                           │  • SdTurboImageEngine                  │
                           │  • Silero VAD / Whisper STT / Qwen LLM │
                           │  • OpenVoice / Piper TTS / YoloV8      │
                           └──────────────────┬─────────────────────┘
                                              │
                      ┌───────────────────────┼───────────────────────┐
                      ▼                       ▼                       ▼
           ┌──────────────────────┐┌──────────────────────┐┌──────────────────────┐
           │  manifestation_app   ││     video_studio     ││     voice_agent      │
           │ (Subconscious Super) ││ (AI Video Production)││  (Real-Time Agent)   │
           └──────────────────────┘└──────────────────────┘└──────────────────────┘
```
1. **Centralized Engine (`crates/core`):** Concrete inference for SD-Turbo (pure Candle safetensors format in `C:\AI_Models\sd_turbo`), along with VAD, STT, LLM, TTS, and Computer Vision, lives inside `ai_core`.
2. **Universal Reusability:** Any future app created in the ecosystem (e.g. `video_studio`, `content_automation`, `social_avatar`) simply inherits `ai_core = { path = "../../crates/core" }` and gains immediate access to the exact same high-speed image synthesis pipeline.
3. **Application Decoupling:** `apps/manifestation_app` focuses strictly on domain logic, subconscious psychology, state orchestration, metered telemetry, and luxury UI rendering.

---

## 3. Target Persona & User Stories

### Target Personas:
1. **High-Performance Achievers & Executives:** Practice daily visualization and subconscious habit reprogramming; require private, air-gapped journaling with zero chance of corporate or personal leaks.
2. **Mindfulness & Manifestation Practitioners:** Follow the Nikola Tesla 3-6-9 method and Neville Goddard's "Living in the End" feeling-state methodology; need vivid visual anchors to induce emotional conviction.
3. **Nighttime Meditators:** Use the hypnagogic window (the transition between wakefulness and sleep where Theta brainwaves 4–7 Hz dominate) to rewrite limiting subconscious beliefs before sleep.

### Key User Stories:
- *As a practitioner*, I want to type my heartfelt goal ("Driving a sapphire blue BMW along Mumbai Marine Drive at sunset") and have the engine synthesize a high-contrast visual anchor in seconds so I can imprint it during meditation.
- *As a daily journaler*, I want a structured 3-6-9 entry workflow that prompts me 3 times in the morning, 6 times in the afternoon, and 9 times at night, calculating my emotional conviction score.
- *As a privacy-focused user*, I want all my visions, gratitude logs, and affirmations stored 100% locally on my machine without any telemetry escaping to Google, Meta, or third-party cloud servers.

---

## 4. Detailed Functional Requirements

### FR-1: Core-Engine First AI Architecture
- The application MUST import `ai_core::models::{ImageGenAdapter, SdTurboConfig, SdTurboImageEngine}`.
- Model weights MUST reside in `C:\AI_Models\sd_turbo` containing `text_encoder`, `unet`, `vae`, `tokenizer`, and `scheduler`.
- The application MUST verify model weight availability gracefully without panicking if weights are in the process of downloading.

### FR-2: Living 3D Vision Board Generation
- Users can create custom Vision items with: `title`, `description`, `feeling`, and `category` ("Wealth", "Health", "Relationship", "Lifestyle", "Career").
- Clicking "Synthesize Visual Anchor" calls `/api/vision/:id/render` which invokes `ai_core`'s `ImageGenAdapter::generate_image`.
- Returns single-step synthesized PNG image encoded directly in Base64 for instant zero-latency browser display.
- Procedural dream canvas fallback ensures visual output even if offline or low on VRAM.

### FR-3: Nikola Tesla 3-6-9 Manifestation Journal
- Implements the 3-6-9 manifestation framework:
  - **Morning (3 repetitions):** Focus on pure intention and establishing gratitude upon waking.
  - **Afternoon (6 repetitions):** Amplify the vibrational intensity and emotional alignment.
  - **Night (9 repetitions):** Seal the manifestation into the subconscious mind before Theta sleep.
- Entries record `feeling_score` (0.0 to 1.0) and ISO-8601 timestamps.

### FR-4: Mirror Affirmation Chamber
- Real-time spoken and textual affirmation capture.
- Analyzes affirmation statement and delivers vibrational conviction feedback (e.g. "High conviction detected! Subconscious mind is imprinting.").
- Metered increments recorded in `ManifestationMetrics::record_affirmation()`.

### FR-5: Theta Bedtime & Sleep Meditation Engine
- Generates tailored subconscious meditation scripts based on the user's specific manifestation goals.
- Incorporates Solfeggio acoustic frequencies:
  - **528 Hz:** Transformation, DNA repair, and miracles.
  - **432 Hz:** Natural harmonic cosmic healing.
- Embeds 6.0 Hz Theta wave auditory pulsing instructions to bridge the conscious-subconscious barrier during bedtime.

### FR-6: Embedded Luxury Glassmorphism Web Portal
- High-aesthetic web application served natively from port `9090` (`http://localhost:9090`).
- Strictly Vanilla HTML5, CSS3, and JavaScript — zero external CDNs, zero NPM dependencies, zero React runtime bloat.
- Dark obsidian and royal gold palette with glassmorphism blur filters (`backdrop-filter: blur(16px)`).
- 100% self-contained inside the compiled Rust binary via `include_str!` or static strings.

### FR-7: 24-Hour Cryptographic Leased DRM & Metered Usage
- Integrated with `crates/drm_guard`.
- Enforces a 24-hour rotating cryptographic lease verified against the license server.
- Seamlessly falls back to local cached lease during network outages.
- Tracks metered metrics: `affirmations_practiced`, `meditation_minutes`, and `vision_boards_generated`.

### FR-8: Silent Background OTA Updater & Lease Heartbeat Daemon
- Runs non-blocking background Tokio task checking for upstream engine updates and renewing leases every hour.
- Machine hardware fingerprint generated using SHA-256 of hardware UUIDs for zero-spoofing node identification.

---

## 5. Non-Functional Requirements & Performance SLAs

| Metric | Target SLA | Production Reality |
| :--- | :--- | :--- |
| **Language & Runtime** | 100% Pure Rust (0% Python) | Pure Rust 2021 Edition |
| **Model Hosting** | `crates/core` (`ai_core`) | Universal adapter inherited by all apps |
| **Web Server Latency** | $< 10\text{ ms}$ response | **$< 2\text{ ms}$ (Axum + Tokio)** |
| **RAM Footprint (Base)** | $< 100\text{ MB}$ | **$< 45\text{ MB}$** |
| **Vision Board Latency** | $< 2\text{ seconds}$ | Sub-second single-step SD-Turbo execution |
| **External Dependencies**| Zero NPM / Zero CDN | 100% embedded assets |
| **Lease Verification** | 24-Hour Offline Resilience | Grace period supported with zero crash |
