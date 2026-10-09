# 🧠 AI Core Framework (`crates/core`)

**Version:** 0.1.0 (Enterprise SaaS Ready)  
**Architecture:** 100% Application-Agnostic, Zero-Python, Pure Rust + ONNX Runtime  
**Target Latency:** Sub-10ms Tensor & Math Execution

---

## 🚀 1. Framework Overview
The **AI Core Framework** (`crates/core`, package name: `ai_core`) is a high-performance, real-time ML/AI and DSP engine designed for Enterprise SaaS backends. It functions as a universal core that powers multiple applications (e.g., `voice_agent`, `customer_support_bot`, `omnichannel_assistant`) simultaneously without leaking any application-specific logic into the ML layer.

### 🌟 Key Philosophy: "Single Binary, Zero Python, Zero Cloud API"
- **Zero Dummy Code:** The framework executes real multi-threading (`tokio`), real-time state machines (`engine`), and live ONNX tensor computations.
- **Application Agnostic:** It exposes clean, asynchronous Rust traits (`VADAdapter`, `SttAdapter`, `LlmAdapter`, `TtsAdapter`, `VisionAdapter`, `OcrAdapter`) via `crate::models::adapters`.
- **Extreme Speed:** Direct in-memory `.onnx` model execution via `ort 2.0.0-rc.6` / `rc.13` with `GraphOptimizationLevel::Level3` bypassing all Python interpretative overhead.

---

## ⚙️ 2. The Real Math & Tensor Pipeline

### A. VAD (Silero) — Listening Math
- **Normalization:** Raw 16-bit signed PCM audio bytes are converted into normalized `Float32` values in `[-1.0, 1.0]`.
- **Tensor Input:** Resized to 512 samples per frame (standard for 16kHz audio).
  - `input`: `[1, 512]` (Float32)
  - `state`: `[2, 1, 128]` (Float32 recurrent LSTM state)
  - `sr`: `[1]` (`16000i64` sample rate)
- **Execution:** Runs `session.run()` to yield speech probability (`output`) and updates recurrent tensor state (`stateN`).

### B. STT (Whisper Tiny) — Speech-to-Text Math
- **Log-Mel Spectrogram:** Audio passes through a 400-sample window FFT and a 80-bin Mel filterbank to create an 80-channel spectrogram tensor `[1, 80, 3000]`.
- **Encoder Pass:** `encoder_model_q4.onnx` generates `last_hidden_state` with shape `[1, 1500, 384]`.
- **Autoregressive Greedy Decoding:** `decoder_model_q4.onnx` runs an iterative loop starting with prompt tokens `[50258, 50259, 50359, 50363]`, computing the `argmax` from vocabulary logits until `<|endoftranscript|>` (`50257`). Decoded back to text using HuggingFace `tokenizers`.

### C. Multimodal Brain (Qwen2) — Reasoning Math
- **ChatML Prompting:** Encodes prompts formatted with `<|im_start|>` and `<|im_end|>` tags.
- **ONNX Forward Pass:** Constructs 3D `inputs_embeds` `[1, seq_len, 1536]`, `attention_mask` `[1, seq_len]`, and `position_ids` `[1, seq_len]`.
- **Streaming Tokens:** Streams generated text tokens asynchronously over `tokio::sync::mpsc::Sender<String>` with full `CancellationToken` support for immediate cancellation.

### D. TTS (Piper + OpenVoice) — Speech Synthesis Math
- **Phonemization:** `NativePhonemizer` converts Hindi/English text into Piper phoneme IDs (`[1, seq_len]`).
- **Piper Neural Pass:** Runs `hi_IN-priyamvada-medium-int8.onnx` with phonemes, `input_lengths`, and `scales` (`[0.667, 1.0, 0.8]`), generating 22.05kHz raw Float32 audio samples.
- **Tone Cloning:** Extracts reference tone vector `[1, 256]` cached in RAM via `RwLock`, and applies tone color via `tone_color.onnx`.
- **PCM Streaming:** Samples are quantized to 16-bit linear PCM and pushed in 1024-byte chunks to `audio_tx: mpsc::Sender<Bytes>`.

### E. DSP Audio Resampling (`dsp.rs`)
- **Rubato Sinc Interpolation:** Band-limited Sinc interpolation converts audio seamlessly between 8kHz (Telephony $\mu$-law), 16kHz (AI models), and 22.05kHz (TTS) without acoustic distortion.

### F. RAG Engine (Embeddings & Reranking)
- **Dense Embeddings:** `BgeEmbeddingLocal` extracts L2-normalized 1024-dimensional vectors from `bge_small_embedding/model.onnx`.
- **Cross-Encoder Reranking:** `BgeRerankerLocal` scores query-document relevance through `bge_reranker/model.onnx` with Sigmoid activation.

---

## 📂 3. Required Local Models Directory (`C:\AI_Models`)

The framework expects all models to reside locally in `C:\AI_Models`:

| Component | Relative Path from `C:\AI_Models\` | Format | Purpose |
|---|---|---|---|
| **VAD** | `silero_vad.onnx` | ONNX (FP32) | Voice Activity Detection |
| **STT Encoder** | `whisper_tiny_onnx\encoder_model_q4.onnx` | ONNX (Q4) | Whisper Audio Encoder |
| **STT Decoder** | `whisper_tiny_onnx\decoder_model_q4.onnx` | ONNX (Q4) | Whisper Text Decoder |
| **STT Tokenizer** | `whisper_tiny_onnx\tokenizer.json` | JSON | BPE Tokenizer for Whisper |
| **LLM Brain** | `qwen2-vl-2b\llmmodel.onnx` | ONNX (Q4/FP16) | Multimodal Reasoning Engine |
| **LLM Tokenizer** | `qwen2-vl-2b\tokenizer.json` | JSON | Qwen ChatML Tokenizer |
| **TTS Base** | `piper_hi\hi_IN-priyamvada-medium-int8.onnx` | ONNX (Int8) | Neural Voice Generator |
| **Tone Color** | `OpenVoice-ONNX-v2\tone_color.onnx` | ONNX (FP32) | Zero-Shot Tone Color Injector |
| **Tone Extract** | `OpenVoice-ONNX-v2\tone_extract.onnx` | ONNX (FP32) | Reference Tone Extractor |
| **RAG Embedding** | `bge_small_embedding\model.onnx` | ONNX (FP32) | Semantic Vector Embeddings |
| **RAG Reranker** | `bge_reranker\model.onnx` | ONNX (FP32) | Cross-Encoder Reranking |

---

## 🛠️ 4. Windows MSVC Build & Linker Configuration

The ONNX Runtime dynamic library (`onnxruntime.dll`) requires MSVC dynamic C-runtime linking (`/MD`). To prevent static/dynamic CRT mismatch linker errors (`LNK2038`), the repository includes `.cargo/config.toml`:

```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-feature=-crt-static"]

[env]
CFLAGS = "/MD"
CXXFLAGS = "/MD"
```

### Build Commands:
```bash
# Check compilation across all crates
cargo check --workspace

# Build the AI Core library
cargo build -p ai_core

# Build with release optimizations
cargo build -p ai_core --release
```

---

## 💻 5. How to Consume `ai_core` in an Application

Add `ai_core` to your application's `Cargo.toml`:
```toml
[dependencies]
ai_core = { path = "../../crates/core" }
```

Initialize models through clean, unified traits:
```rust
use ai_core::models::onnx_adapters::{
    SileroVadLocal, WhisperTinyLocal, QwenLocal, OpenVoiceLocal
};
use std::sync::Arc;
use tokio::sync::Mutex;

// 1. Initialize VAD
let vad = Arc::new(Mutex::new(SileroVadLocal::new("C:\\AI_Models\\silero_vad.onnx")?));

// 2. Initialize STT
let stt = Arc::new(WhisperTinyLocal::new(
    "C:\\AI_Models\\whisper_tiny_onnx\\encoder_model_q4.onnx",
    "C:\\AI_Models\\whisper_tiny_onnx\\decoder_model_q4.onnx",
    "C:\\AI_Models\\whisper_tiny_onnx\\tokenizer.json",
)?);

// 3. Initialize LLM Brain
let llm = Arc::new(QwenLocal::new(
    "C:\\AI_Models\\qwen2-vl-2b\\llmmodel.onnx",
    "", "",
    "C:\\AI_Models\\qwen2-vl-2b\\tokenizer.json",
)?);

// 4. Initialize TTS
let tts = Arc::new(OpenVoiceLocal::new(
    "C:\\AI_Models\\piper_hi\\hi_IN-priyamvada-medium-int8.onnx",
    "C:\\AI_Models\\OpenVoice-ONNX-v2\\tone_color.onnx",
    "C:\\AI_Models\\OpenVoice-ONNX-v2\\tone_extract.onnx",
    "assets/reference.wav",
)?);
```
No machine learning or ONNX logic needs to be implemented in the application layer. The application strictly focuses on networking, business logic, and protocols.
