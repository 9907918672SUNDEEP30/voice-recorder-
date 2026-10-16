# 📋 Product Requirements Document (PRD): Voice AI Calling Agent (`voice_agent`)

## 1. Executive Summary & Problem Statement
Customer support centers, lead qualification desks, and debt-collection operations face immense staffing overhead, human burnout, and inconsistent compliance. Existing voice AI bots rely on fragile chains of cloud APIs (OpenAI, Deepgram, ElevenLabs) wrapped in Python, incurring high latency (>1.5–3.0 seconds), unpredictable cloud API bills ($0.15–$0.30 per minute), and privacy leaks.

**`voice_agent`** is a standalone, 100% native Rust, full-duplex conversational telephony AI engine:
1. **Full-Duplex Sub-300ms Conversational Turnaround:** Streamlined native pipeline with Silero VAD, Whisper STT, Qwen LLM, and Piper/OpenVoice TTS executing with zero heavy Python wrappers.
2. **Telephony Ingestion & Audio Resampling (`codec.rs`):** Standard ITU-T G.711 $\mu$-law codec table decoding 8kHz telephony streams into 16kHz PCM with linear interpolation and zero-copy buffers.
3. **Smart Barge-In (Interruption Handling):** Automatically halts ongoing TTS speech synthesis the instant the caller speaks, preserving natural conversational flow.
4. **Cloudflare & Twilio / Exotel WebSocket Protocol:** Standardized bidirectional WebSocket interface (`/media-stream`) compatible with global telecom providers.
5. **Zero Cloud Per-Minute Fee:** 100% self-hosted on edge servers or local appliances with minimal operating costs.

---

## 2. Target Market & User Personas

| Target Segment | Typical Deployments | Key Pain Points Solved |
|---|---|---|
| **SME Inbound Receptionist** | Clinic appointments, restaurant bookings, repair services. | Missed calls after business hours; immediate confirmation via SMS/WhatsApp. |
| **Outbound Lead Qualification** | Real estate, insurance, finance outreach. | High agent turnover; deterministic script compliance; sub-300ms natural conversational speed. |
| **Payment Reminder & Collection** | Telecom, micro-lending, utility bills. | Automated polite payment follow-ups with instant localized Hindi/English voice. |
| **Air-Gapped & Secure Enterprises** | Government, defense, healthcare helplines. | 100% on-premises data processing; zero caller audio sent to public LLM cloud vendors. |

---

## 3. Core Functional Requirements

### 3.1 Telephony Protocol & Audio Codec
- **Protocol:** Bidirectional Axum WebSocket server listening on port 3000 (`/media-stream`).
- **Codec:** Native G.711 $\mu$-law 8kHz $\leftrightarrow$ 16kHz linear resampling with precomputed lookup tables.
- **Payload Handling:** Base64 chunk serialization compliant with Twilio Media Streams and Exotel Voice WebSockets.

### 3.2 Voice Activity Detection (VAD) & Barge-In
- **Silero VAD ONNX Engine:** 512-sample (32ms) frame analysis running continuously.
- **Dynamic Silence Detection:** Triggers end-of-speech turnaround after 600–800ms of caller silence.
- **Instant TTS Abort:** Immediate cancellation of active audio stream playback when user speaks.

### 3.3 Speech-to-Text (STT) & LLM Synthesis
- **Whisper STT:** On-device Whisper-Tiny/Small ONNX inference yielding $<120\text{ms}$ transcription.
- **Qwen Conversational Brain:** Grounded persona prompts (e.g. Maya Receptionist) streaming tokens to TTS.
- **Hindi & Multilingual Support:** Native phonemizer and multilingual vocabulary integration.

### 3.4 Text-to-Speech (TTS) Streaming
- **Synthesizer:** Piper / OpenVoice ONNX local neural speech generator.
- **Chunked Playback:** Sentence-boundary streaming delivering first audio response in $<300\text{ms}$.
- **Call Persistence:** Local SQLite call logs and optional Supabase call transcript sync.

---

## 4. Platform & Target Environments (Cloud / Desktop / Server App)

- **Application Nature:** Native Telephony Voice Service & Client App (Runs on standard cloud VPS, office desktop, laptop, or mobile telephony clients — no dedicated proprietary hardware appliance required).
- **Supported Platforms:**
  - **Servers & Cloud:** Linux (Ubuntu, Debian, Alpine Docker container) & Windows Server.
  - **Desktop & Mobile:** Windows 10/11, macOS, and Mobile via WebSocket browser client (`test_client.html`).
- **Resource Footprint:** Ultra-lean native Rust memory baseline (< 150 MB RAM). Runs effortlessly alongside existing office software or on any cheap $3/month cloud VPS without GPU.
- **Delivery Form Factor:** Single self-contained static executable (`voice_agent.exe` / `voice_agent`) or lightweight Docker image.

