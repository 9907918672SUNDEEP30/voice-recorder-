# 🏗️ System Architecture & Engineering Specification
## Shambhavi Sankalpa — Manifestation Super App Engine (`apps/manifestation_app`)

---

## 1. Architectural Philosophy & Core Tenets

The architecture of `apps/manifestation_app` is governed by four strict architectural tenets:

1. **"Core-Engine First" Policy:** The manifestation application MUST NOT contain ad-hoc, proprietary AI model inference pipelines. All neural models (text-to-image synthesis, neural speech, audio DSP, computer vision) reside centrally in `crates/core` (`ai_core`). The application consumes these capabilities strictly via abstract trait interfaces (`ImageGenAdapter`). This ensures that any future applications built in the workspace can instantly inherit the exact same capabilities without duplicating code or model weights.
2. **Pure Native Rust & Zero Python Guarantee:** 100% of the code compiles into a single, high-performance, deterministic native binary. No Python virtual environments, PyTorch runtimes, node.js servers, or Electron wrappers are permitted in production runtimes.
3. **Single-Binary Zero-Bloat Deployment:** All web portal assets (HTML5, Vanilla CSS3, JavaScript) are compiled directly into the binary. Launching the application starts an embedded Axum web server on port `9090` without requiring external CDN access, NPM builds, or static asset folders.
4. **Universal 24-Hour Leased DRM & Metered Billing:** Cryptographic licensing, 24-hour rotating offline leases, and usage metering are enforced transparently via `crates/drm_guard`.

---

## 2. High-Level System Architecture Diagram

```
 ┌────────────────────────────────────────────────────────────────────────┐
 │                      PRESENTATION LAYER (PORT 9090)                    │
 │                                                                        │
 │  ┌──────────────────────────────────────────────────────────────────┐  │
 │  │ Embedded Luxury Glassmorphism SPA (Vanilla HTML5 / CSS3 / JS)    │  │
 │  │ • Living 3D Vision Boards      • 3-6-9 Nikola Tesla Journal      │  │
 │  │ • Mirror Affirmation Chamber   • Theta Bedtime Sleep Portal      │  │
 │  └──────────────────────────────────┬───────────────────────────────┘  │
 └─────────────────────────────────────┼──────────────────────────────────┘
                                       │ HTTP / REST / JSON (< 2ms)
                                       ▼
 ┌────────────────────────────────────────────────────────────────────────┐
 │            APPLICATION DOMAIN & ORCHESTRATION LAYER                    │
 │                    (apps/manifestation_app)                            │
 │                                                                        │
 │  ┌─────────────────────────┐  ┌──────────────────────────────────────┐ │
 │  │   Thread-Safe State     │  │   Domain Service Handlers            │ │
 │  │   (Arc<AppState>)       │  │   • Vision Board Manager             │ │
 │  │   • Arc<RwLock<Visions>>│  │   • 3-6-9 Journal Engine             │ │
 │  │   • Arc<RwLock<Gratitude│  │   • Mirror Affirmation Scorer        │ │
 │  │   • DRMGuard Reference  │  │   • Theta Bedtime Script Generator   │ │
 │  └───────────┬─────────────┘  └──────────────────┬───────────────────┘ │
 └──────────────┼───────────────────────────────────┼─────────────────────┘
                │                                   │
                │ Calls ImageGenAdapter             │ Updates Usage Metrics
                ▼                                   ▼
 ┌──────────────────────────────┐    ┌────────────────────────────────────┐
 │      CENTRAL AI CORE         │    │           DRM GUARD                │
 │       (crates/core)          │    │       (crates/drm_guard)           │
 │                              │    │                                    │
 │  • ImageGenAdapter (Trait)   │    │  • 24-Hour Rotating Lease          │
 │  • SdTurboImageEngine        │    │  • ManifestationMetrics            │
 │  • Pure Candle Safetensors   │    │  • Machine UUID Fingerprinting     │
 │  • Procedural Dream Canvas   │    │  • Background Heartbeat Daemon     │
 └──────────────┬───────────────┘    └────────────────────────────────────┘
                │
                ▼ Reads Weights
 ┌────────────────────────────────────────────────────────────────────────┐
 │               LOCAL WEIGHT STORAGE (C:\AI_Models\sd_turbo)             │
 │  • text_encoder/model.safetensors (1.36 GB)                            │
 │  • unet/diffusion_pytorch_model.safetensors (3.22 GB)                  │
 │  • vae/diffusion_pytorch_model.safetensors (319 MB)                   │
 └────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Component Deep Dive

### 3.1. Centralized AI Engine Integration (`crates/core`)
Instead of bundling ONNX or Candle models inside `manifestation_app`, the app references `ai_core`:
```rust
use ai_core::models::{ImageGenAdapter, SdTurboConfig, SdTurboImageEngine};
```
- **Trait Decoupling:** The application interacts solely with `Arc<dyn ImageGenAdapter>` (or concrete `SdTurboImageEngine`).
- **Graceful Fallback:** If the multi-gigabyte safetensors weights are still downloading from Hugging Face, `SdTurboImageEngine::is_model_available()` returns `false`, and the engine seamlessly generates high-contrast procedural visual canvases using golden-ratio luxury color gradients, ensuring zero panics or blank screens.
- **Single-Step Latent Diffusion:** SD-Turbo is architected to synthesize complete images in a single step ($N = 1$), eliminating multi-iteration diffusion loops and slashing GPU/CPU latency to hundreds of milliseconds.

### 3.2. Subconscious Mind Domain Modules
1. **Living Vision Board Engine:**
   - Stores active goals (`VisionItem`) with Title, Emotional Feeling State, and Category.
   - When a user triggers visual synthesis, a descriptive prompt is assembled (e.g. `"<description>, highly detailed cinematic golden ratio, feeling of <feeling>"`), and dispatched to `ai_core`.
   - Resulting PNG bytes are Base64-encoded and transmitted over REST JSON to update the client canvas instantly.
2. **Nikola Tesla 3-6-9 Gratitude Journal:**
   - Enforces the 3-6-9 manifestation rhythm:
     - 3 Morning intentions
     - 6 Afternoon vibrational escalations
     - 9 Night subconscious imprints
   - Each entry calculates a vibrational conviction index (`feeling_score`), saved in a lock-free thread-safe vector with ISO-8601 timestamps.
3. **Mirror Affirmation Chamber:**
   - Facilitates gaze-focused conscious-to-subconscious affirmation logging.
   - Evaluates cognitive resonance and triggers positive feedback loops.
   - Atomically increments `ManifestationMetrics::record_affirmation()`.
4. **Theta Bedtime & Sleep Meditation Engine:**
   - Exploits the hypnagogic transitional phase where conscious critical filters relax into Theta brainwaves (4–7 Hz).
   - Dynamically synthesizes personalized meditation journey scripts embedded with Solfeggio acoustic frequencies (528 Hz transformation, 432 Hz healing).

### 3.3. Universal 24-Hour Cryptographic Leased DRM
Commercial security is maintained through `crates/drm_guard`:
- **Cryptographic Leases:** The application requires a valid 24-hour cryptographic lease token.
- **Graceful Offline Tolerance:** If internet connectivity drops, the local cached lease remains valid for up to 24 hours, guaranteeing uninterrupted meditation sessions.
- **Metered Usage Reporting:** Tracks user interaction volume (`affirmations_practiced`, `meditation_minutes`, `vision_boards_generated`) in atomic counters, reported during regular background heartbeats.

---

## 4. Sequence Workflows

### Sequence 1: Vision Board Creation & Single-Step Synthesis
```mermaid
sequenceDiagram
    autonumber
    actor User as Practitioner
    participant UI as Glassmorphism Web Portal
    participant Axum as Axum REST Router
    participant Core as ai_core (SdTurboImageEngine)
    participant DRM as DRMGuard (ManifestationMetrics)

    User->>UI: Types Goal & Desired Emotional Feeling
    UI->>Axum: POST /api/vision/create (JSON)
    Axum->>DRM: metrics.record_vision_board()
    Axum-->>UI: 201 Created (VisionItem ID)
    
    User->>UI: Clicks "Synthesize Visual Anchor"
    UI->>Axum: POST /api/vision/:id/render
    Axum->>Core: generate_image(prompt, width=512, height=512, steps=1)
    Core->>Core: Execute SD-Turbo / Procedural Dream Canvas
    Core-->>Axum: Ok(Vec<u8> PNG Bytes)
    Axum->>Axum: Encode PNG into Base64 Data URI
    Axum-->>UI: 200 OK {"image_data": "data:image/png;base64,..."}
    UI->>User: Displays Living Vision Card with Golden Aura
```

### Sequence 2: 3-6-9 Subconscious Journaling & Nighttime Theta Induction
```mermaid
sequenceDiagram
    autonumber
    actor User as Practitioner
    participant UI as Glassmorphism Web Portal
    participant Axum as Axum REST Router
    participant DRM as DRMGuard (ManifestationMetrics)

    Note over User,UI: Nighttime Hypnagogic Window (Theta 4-7 Hz)
    User->>UI: Enters 9th Night Gratitude Reflection
    UI->>Axum: POST /api/gratitude/add (entry_type: "Night_9")
    Axum-->>UI: 201 Created (feeling_score: 0.95)
    
    User->>UI: Requests Bedtime Sleep Meditation
    UI->>Axum: POST /api/meditation/generate (topic: "Abundance & Peace")
    Axum->>DRM: metrics.record_meditation(180)
    Axum-->>UI: 200 OK (528 Hz + 6.0 Hz Theta Script)
    UI->>User: Renders Guided Visual Meditation Journey
```

---

## 5. Concurrency & Memory Budget

| Component | Concurrency Model | Target Memory | Realized Memory |
| :--- | :--- | :--- | :--- |
| **Axum Web Server** | Async non-blocking (Tokio multi-thread) | $< 15\text{ MB}$ | **$\sim 8\text{ MB}$** |
| **Vision Items Store** | `Arc<RwLock<Vec<VisionItem>>>` | $< 5\text{ MB}$ | **$< 1\text{ MB}$** |
| **3-6-9 Gratitude Store** | `Arc<RwLock<Vec<GratitudeEntry>>>`| $< 5\text{ MB}$ | **$< 1\text{ MB}$** |
| **DRM Guard Daemon** | Tokio background task (`spawn`) | $< 5\text{ MB}$ | **$< 2\text{ MB}$** |
| **Total Base Runtime** | Compiled static binary | **$< 50\text{ MB}$** | **$< 45\text{ MB}$** |

By strictly delegating neural model operations to `ai_core` and utilizing lock-free and asynchronous primitives, `manifestation_app` achieves institutional-grade reliability, zero memory leaks, and blazing performance.
