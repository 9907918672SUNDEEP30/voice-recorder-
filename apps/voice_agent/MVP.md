# 🚀 Minimum Viable Product (MVP) Roadmap
## Voice AI Calling Agent (`apps/voice_agent`)

---

## 📌 Executive MVP Scope

The **Minimum Viable Product (MVP)** delivers a fully functional, self-hosted telephony AI bot built in 100% pure Rust. It connects to Twilio or Exotel via bidirectional WebSockets, processes audio using local ONNX neural models (VAD, Whisper STT, Qwen LLM, TTS), and achieves sub-300ms conversational turnaround with barge-in support. It runs seamlessly on ultra-low-budget hardware (Minimal Dual-Core/Celeron CPU, 2 GB RAM, SSD) with zero per-minute cloud fees.

---

## 🗺️ Multi-Phase Implementation Roadmap

```mermaid
gantt
    title Voice Agent MVP Implementation Timeline
    dateFormat  YYYY-MM-DD
    section Phase 1: Telephony Ingestion & Codec
    Axum WebSocket Server (/media-stream) & Twilio Protocol     :active, p1_1, 2026-10-17, 1d
    G.711 mu-law 8kHz <-> 16kHz Linear Resampling Table        :p1_2, after p1_1, 1d
    section Phase 2: Neural Processing Loop
    Silero VAD Silence Gate & Smart Barge-In Interruption      :p2_1, after p1_2, 1d
    Whisper-Tiny STT ONNX Local Speech Recognition              :p2_2, after p2_1, 1d
    Qwen LLM Persona Dialogue Engine (Maya Hindi/English)       :p2_3, after p2_2, 1d
    section Phase 3: Speech Synthesis & Streaming
    Piper / OpenVoice Neural TTS Audio Generation              :p3_1, after p2_3, 1d
    Chunked Audio Packet Streamer to WebSocket                  :p3_2, after p3_1, 1d
    section Phase 4: Verification & Web Client
    Interactive HTML5 Browser Test Client (test_client.html)    :p4_1, after p3_2, 1d
    Production Service Packaging (Windows Service / Systemd)   :p4_2, after p4_1, 1d
```

---

## 🧪 MVP Verification & Test Matrix

| Component | Target Verification | Expected Pass Criteria |
| :--- | :--- | :--- |
| **G.711 Codec** | $\mu$-law 8kHz $\leftrightarrow$ 16kHz PCM roundtrip | Bit-accurate linear interpolation; zero audio clipping. |
| **Silero VAD** | 32ms audio frames speech/silence detection | Identifies caller pause in $<800\text{ms}$; ignores background hum. |
| **Barge-In Interrupt** | User speaks while bot is playing audio | Bot cuts off playback in $<50\text{ms}$; resets audio buffer cleanly. |
| **STT Accuracy** | 10 telephony spoken audio phrases | Word Error Rate (WER) $<8\%$; latency $<120\text{ms}$. |
| **End-to-End Latency** | Caller finishes speaking to first bot audio | Total turnaround latency strictly $<300\text{ms}$. |
| **Hardware Footprint** | Concurrent active call on 2 GB RAM box | Total process RAM $<150\text{ MB}$; CPU usage $<25\%$ during inference. |

---

## 🏁 MVP Launch Acceptance Criteria
1. Automated test suite in `apps/voice_agent` passes with **100% green status**.
2. Live interactive testing verified via `test_client.html` in browser.
3. Clean workspace check: `cargo check -p voice_agent` with **0 errors**.
4. 100% self-hosted, offline operation without third-party cloud API costs.
