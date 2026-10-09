# 🧠 AI Core Framework (SaaS Engine)

**Version:** 1.0.0 (Production Ready)  
**Architecture:** 100% Neutral, Application-Agnostic, Zero-Python, Pure Rust + ONNX

---

## 🚀 1. Framework Overview
The **AI Core Framework** is a high-performance, real-time voice and logic engine designed for Enterprise SaaS applications. It acts as a universal backend that can power multiple applications (e.g., `voice_agent`, `customer_support_bot`, `outbound_sales`) simultaneously without any application-specific logic leaking into the core.

### 🌟 Key Philosophy: "Single Binary, Zero Python, 10ms Latency"
- **Zero Dummy Code:** The framework handles real Multi-threading (`tokio`), Real-time State Machines (`engine`), Inter-Process Communication (`ipc`), and raw ONNX Tensor computations.
- **Application Agnostic:** It provides a universal set of Traits (`VADAdapter`, `SttAdapter`, `LlmAdapter`, `TtsAdapter`) that any App can use. The core doesn't care about your business logic.
- **Extreme Speed:** Directly loads `.onnx` models into RAM via `ort` at `GraphOptimizationLevel::Level3` (Max CPU Speed) bypassing heavy Python runtimes.

---

## ⚙️ 2. The Real Math & Tensor Pipeline (100% Implemented)

### A. VAD (Silero) - The Listening Math
- **Pre-processing:** Raw 16-bit PCM Audio Bytes are mathematically converted into normalized `Float32` tensors `[-1.0, 1.0]`.
- **Execution:** Pushed into `silero_vad.onnx` as a 2D Tensor `[1, sequence_length]` along with LSTM state vectors to detect human speech instantly.

### B. STT (Whisper) - Speech to Text
- **Log-Mel Spectrogram:** The incoming audio is passed through a Fast Fourier Transform (FFT) and a Mel Filterbank to create an 80-bin Spectrogram Tensor `[1, 80, 3000]`.
- **Autoregressive Greedy Decoding:** The engine runs a real ONNX Autoregressive loop, starting with `<|startoftranscript|><|hi|><|transcribe|>` tokens, calculating the `argmax` from logits, and decoding it back to text using HuggingFace's `tokenizers` crate in pure Rust.

### C. The Multimodal Brain (Qwen2-VL)
The system uses a single unified model (`qwen2-vl-2b.onnx`) to handle Text, Vision, and OCR simultaneously:
- **LLM & Text Generation:** Generates text streams autoregressively directly from ONNX, using `tokenizers` to mathematically encode/decode ChatML.
- **Vision (Image Understanding):** Runs a Vision Transformer (ViT) pass to create Image Embeddings, concatenating them with the text prompt to generate visual answers.
- **OCR (Optical Character Recognition):** Eliminates the need for separate OCR models (like PaddleOCR). Qwen2-VL natively recognizes and extracts text from images within the same tensor pass.

### E. RAG Engine (Memory & Retrieval)
- **Dense Embedding (BGE-m3):** The `BgeEmbeddingLocal` adapter passes company documents (like Refund Policies) through a Transformer, applies Mean Pooling, and extracts a dense `L2 Normalized` Float32 vector (e.g., `[1, 1024]`).
- **Cross-Encoder Reranking (BGE-Reranker):** The `BgeRerankerLocal` adapter takes the user's query and top document matches, concatenates them, and runs them through a Classifier Head with a Sigmoid activation to assign a relevance probability (e.g., 0.99). This guarantees the LLM only reads exactly what is relevant.

### F. TTS (Piper + OpenVoice) - Voice Cloning Math
A state-of-the-art 2-step mathematical pipeline:
1. **Phonemization & Base TTS (Piper):** Uses our `NativePhonemizer` to map Hindi words to phoneme IDs, then runs them through `piper.onnx` to generate a 1D Tensor of "Neutral" audio.
2. **Tone Extraction & Color (OpenVoice v2):** Performs FFT on a 3-second reference audio to yield a `[1, 256]` Tone Embedding Tensor (cached in RAM). It then injects this Tone Embedding into the Piper Neutral Audio Tensor using `tone_color.onnx` to instantly clone the voice.

### G. Telephony Audio Resampling (DSP Math)
- **Fast-Fixed Sinc Interpolation:** Implemented via the `rubato` crate in `dsp.rs`. Safely converts audio between telephony networks (8kHz MuLaw format used by Twilio/Jio) and our AI models (16kHz Float32) without any distortion.

## 🧠 3. Advanced Architecture (IPC & State Machine)

### 🔄 A. The Pipeline Orchestrator (State Machine & Barge-in)
The core engine supports a millisecond-level State Machine that tracks the conversation state:
- `Idle` -> `Listening` -> `Thinking` -> `Speaking`
- **Barge-in (Interrupt) Logic:** If the AI is `Speaking` and the VAD detects human voice (`Listening`), the Orchestrator instantly fires a `CancellationToken` to kill the LLM and TTS tasks mid-sentence, ensuring the AI stops speaking in <100ms and listens to the user.

### ⚡ B. IPC (Inter-Process Communication) & Telemetry
- **IPC:** The engine uses highly optimized Rust channels (`tokio::sync::mpsc`) and `Bytes` streams to pass audio chunks and text tokens across threads without heavy memory cloning.
- **Telemetry:** Every latency metric (VAD Latency, TTFT - Time to First Token, Audio Chunk generation time) is logged via the `telemetry.rs` module in microseconds for strict performance monitoring.

---

## 🛠️ 4. Setup & Build Instructions

### A. Download the ONNX Models
Before running the framework, you need to place the ONNX models in your local directory (e.g., `C:\AI_Models\`).
Required models:
1. `silero_vad.onnx` (For Listening VAD)
2. `whisper_tiny_onnx/encoder_model_q4.onnx` & `decoder_model_merged_q4.onnx` & `tokenizer.json` (For STT)
3. `qwen2-vl-2b/llmmodel.onnx`, `visionmodel.onnx`, & `vision_encoder.onnx` (For Multimodal Brain)
4. `piper_hi/.../hi_IN-priyamvada-medium.onnx` & `OpenVoice-ONNX-v2/tone_color.onnx`, `tone_extract.onnx` (For TTS & Voice Cloning)
5. `bge_small_embedding/model.onnx` (For Dense Embeddings)
6. `bge_reranker/model.onnx` (For Cross-Encoder Scoring)

### B. Build and Run (Zero-Python)
Since this is a pure Rust SaaS engine, there are no Python environments (`pip`, `conda`, or `requirements.txt`) to manage.

```bash
# Compile the core framework for maximum speed
cargo build --release

# Run a specific app (e.g., Voice Agent)
cargo run --release --bin voice_agent
```

---

## 💻 5. How to Use in an App
Any app inside the `apps/` directory simply calls the core:

```rust
use ai_core::models::onnx_adapters::{OpenVoiceLocal, WhisperTinyLocal, QwenLocal};

// Initialize the universal core components
let tts_adapter = OpenVoiceLocal::new(piper_path, color_path, extract_path, ref_audio)?;
let stt_adapter = WhisperTinyLocal::new(whisper_path, whisper_tokenizer)?;
let llm_adapter = QwenLocal::new(qwen_path, qwen_tokenizer)?;
```
No AI logic is maintained in the App. The App only focuses on Business Logic and WebSockets.
