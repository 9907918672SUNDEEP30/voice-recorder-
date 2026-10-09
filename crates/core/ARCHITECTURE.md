# 🧠 AI Core Framework Architecture (`crates/core`)

## 1. High-Level Overview
The **AI Core Framework** (`crates/core`, package name: `ai_core`) is the foundational, application-agnostic ML/AI runtime of the system. Built with pure Rust and ONNX Runtime (`ort 2.0.0-rc.6` / `rc.13`), it delivers sub-millisecond tensor mathematics, neural inference, and digital signal processing (DSP) without relying on Python runtimes or external cloud APIs.

All higher-level applications (such as `apps/voice_agent`) consume the core through unified asynchronous Rust traits defined in `crate::models::adapters`.

```
+-----------------------------------------------------------------------------------+
|                           Application Layer (e.g. apps/voice_agent)               |
+-----------------------------------------------------------------------------------+
                                          |
                                          v  (Async Traits: VADAdapter, SttAdapter, etc.)
+-----------------------------------------------------------------------------------+
|                                  AI CORE FRAMEWORK                                |
|                                                                                   |
|  +--------------------+  +----------------------+  +---------------------------+  |
|  |   Listening Core   |  |  Understanding Core  |  |       Reasoning Core      |  |
|  |   (SileroVadLocal) |  |  (WhisperTinyLocal)  |  |        (QwenLocal)        |  |
|  |   512 f32 Samples  |  |  80-Bin Log-Mel      |  |  ChatML + 3D Tensor Embed |  |
|  |   LSTM State       |  |  Encoder + Decoder   |  |  Token Streaming          |  |
|  +--------------------+  +----------------------+  +---------------------------+  |
|                                                                                   |
|  +--------------------+  +----------------------+  +---------------------------+  |
|  |     Voice Core     |  |       DSP Core       |  |         RAG Core          |  |
|  |  (OpenVoiceLocal)  |  |   (AudioResampler)   |  |   (BgeEmbeddingLocal      |  |
|  |  Piper ONNX Int8   |  |  Rubato Sinc Interp  |  |    + BgeRerankerLocal)    |  |
|  |  Tone Color Clone  |  |  8k <-> 16k <-> 22k  |  |  L2 Vectors + Sigmoid     |  |
|  +--------------------+  +----------------------+  +---------------------------+  |
|                                                                                   |
|  +-----------------------------------------------------------------------------+  |
|  |               State Machine Engine (Orchestrator, Step, State)              |  |
|  +-----------------------------------------------------------------------------+  |
+-----------------------------------------------------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
|               Local ONNX Runtime (`ort` @ GraphOptimizationLevel::Level3)         |
|                        Local Models Stored in `C:\AI_Models`                      |
+-----------------------------------------------------------------------------------+
```

---

## 2. Component Blueprint & Tensor Specifications (`onnx_adapters.rs`)

### A. Listening Core (Voice Activity Detection)
* **Struct:** `SileroVadLocal`
* **Trait:** `VADAdapter`
* **Model File:** `C:\AI_Models\silero_vad.onnx`
* **Optimization:** `GraphOptimizationLevel::Level3`, `intra_threads = 1`
* **Execution Flow:**
  1. Accepts 16-bit linear PCM audio chunk (`&[u8]`).
  2. Converts to normalized Float32 values: `sample as f32 / 32768.0`.
  3. Resizes to 512 samples (standard chunk size for 16kHz audio).
  4. Prepares ONNX input tensors:
     - `input`: Shape `[1, 512]` (Float32 audio tensor)
     - `state`: Shape `[2, 1, 128]` (Float32 recurrent LSTM state)
     - `sr`: Shape `[1]` (`16000i64` sample rate tensor)
  5. Executes `session.lock().run(inputs)`.
  6. Extracts speech probability from output `output[[0, 0]]`.
  7. Updates internal recurrent state tensor from output `stateN`.
* **Output:** `(is_speaking: bool, speech_probability: f32)`.

---

### B. Understanding Core (Speech to Text)
* **Struct:** `WhisperTinyLocal`
* **Trait:** `SttAdapter`
* **Model Files:**
  - Encoder: `C:\AI_Models\whisper_tiny_onnx\encoder_model_q4.onnx`
  - Decoder: `C:\AI_Models\whisper_tiny_onnx\decoder_model_q4.onnx`
  - Tokenizer: `C:\AI_Models\whisper_tiny_onnx\tokenizer.json`
* **Optimization:** `GraphOptimizationLevel::Level3`, `intra_threads = 4` per session
* **Execution Flow:**
  1. **Log-Mel Spectrogram Calculation:**
     - Computes 80-channel Log-Mel spectrogram tensor `[1, 80, 3000]`.
     - Frame size: 400 samples (25ms @ 16kHz), Hop size: 160 samples (10ms @ 16kHz).
     - Log-energy calculated per frame across 80 mel-frequency filter banks.
  2. **Encoder Pass:**
     - Passes `input_features` `[1, 80, 3000]` into `encoder_model_q4.onnx`.
     - Extracts `last_hidden_state` tensor with shape `[1, 1500, 384]`.
  3. **Greedy Autoregressive Decoder Loop:**
     - Initial prompt tokens: `[50258, 50259, 50359, 50363]` (`<|startoftranscript|><|hi|><|transcribe|><|notimestamps|>`).
     - Loop iteratively runs `decoder_model_q4.onnx` with inputs `input_ids` and `encoder_hidden_states`.
     - Computes `argmax` across vocabulary logits (`vocab_size = 51865`) to select next token.
     - Stops when `<|endoftranscript|>` (`50257`) is emitted or max tokens reached.
  4. **Text Decoding:** HuggingFace `tokenizers` crate converts token IDs to UTF-8 text string.
* **Output:** Transcribed Hindi/English text string.

---

### C. Brain & Reasoning Core (Multimodal LLM)
* **Struct:** `QwenLocal`
* **Traits:** `LlmAdapter`, `VisionAdapter`, `OcrAdapter`
* **Model Files:**
  - LLM Brain: `C:\AI_Models\qwen2-vl-2b\llmmodel.onnx`
  - Vision Model: `C:\AI_Models\qwen2-vl-2b\visionmodel.onnx` (Optional/Multimodal)
  - Vision Encoder: `C:\AI_Models\qwen2-vl-2b\vision_encoder.onnx` (Optional/Multimodal)
  - Tokenizer: `C:\AI_Models\qwen2-vl-2b\tokenizer.json`
* **Optimization:** `GraphOptimizationLevel::Level3`
* **Execution Flow:**
  1. Formats input into standard ChatML prompt:
     ```text
     <|im_start|>system
     {system_prompt}<|im_end|>
     <|im_start|>user
     {user_message}<|im_end|>
     <|im_start|>assistant
     ```
  2. Encodes prompt into token sequence using BPE tokenizer.
  3. Constructs ONNX input tensors:
     - `inputs_embeds`: Shape `[1, seq_len, 1536]` (Float32 3D tensor)
     - `attention_mask`: Shape `[1, seq_len]` (`1i64` mask)
     - `position_ids`: Shape `[1, seq_len]` (`0..seq_len` indices)
  4. Executes forward pass on `llmmodel.onnx` to generate output logits.
  5. Spawns asynchronous streaming task over `tokio::sync::mpsc::Sender<String>`.
  6. Respects `CancellationToken` for sub-50ms conversational interruption (barge-in).
* **Output:** Asynchronous channel stream `mpsc::Receiver<String>` of response tokens.

---

### D. Voice Core (Text to Speech & Voice Cloning)
* **Struct:** `OpenVoiceLocal`
* **Trait:** `TtsAdapter`
* **Model Files:**
  - Base Piper TTS: `C:\AI_Models\piper_hi\hi_IN-priyamvada-medium-int8.onnx`
  - Tone Color Model: `C:\AI_Models\OpenVoice-ONNX-v2\tone_color.onnx`
  - Tone Extract Model: `C:\AI_Models\OpenVoice-ONNX-v2\tone_extract.onnx`
* **Optimization:** `GraphOptimizationLevel::Level3`, `intra_threads = 2`
* **Execution Flow:**
  1. **Phonemization:** `NativePhonemizer` converts input text into Piper Phoneme IDs (`Vec<i64>`).
  2. **Neural Forward Pass:**
     - `input`: Shape `[1, seq_len]` phoneme IDs tensor.
     - `input_lengths`: Shape `[1]` (`[seq_len as i64]`).
     - `scales`: Shape `[3]` (`[0.667f32, 1.0f32, 0.8f32]` noise_scale, length_scale, noise_w).
     - Generates 22.05kHz raw Float32 audio waveform tensor `output`.
  3. **Tone Extraction & Voice Cloning:**
     - Tone embedding vector `[1, 256]` cached in RAM via `RwLock`.
     - Injects tone color via `tone_color.onnx` forward pass.
     - Live voice changing supported via `update_voice(new_reference_path)`.
  4. **PCM Conversion & Live Streaming:**
     - Clamps samples `[-1.0, 1.0]` and quantizes to signed 16-bit integers (`i16`).
     - Splits into 1024-byte PCM chunks and streams continuously into `audio_tx: mpsc::Sender<Bytes>`.
     - Checks `CancellationToken` before every chunk to stop playback instantly on interruption.
* **Output:** Continuous real-time stream of PCM 16-bit bytes.

---

### E. Digital Signal Processing (DSP) Core (`dsp.rs`)
* **Struct:** `AudioResampler`
* **Library:** `rubato` (Band-limited Sinc Interpolation)
* **Functionality:**
  - Converts telephony audio (8kHz G.711 $\mu$-law) $\to$ 16kHz Float32 for VAD and STT.
  - Converts synthesized TTS audio (22.05kHz PCM) $\to$ 8kHz linear PCM $\to$ 8kHz $\mu$-law for outbound telephony streams.
  - Guarantees zero acoustic distortion and sub-millisecond conversion latency.

---

### F. RAG & Semantic Retrieval Core
* **Structs:** `BgeEmbeddingLocal`, `BgeRerankerLocal`
* **Model Files:**
  - Embedding: `C:\AI_Models\bge_small_embedding\model.onnx`
  - Reranker: `C:\AI_Models\bge_reranker\model.onnx`
* **Functionality:**
  - Generates 1024-dimensional dense vectors with L2 normalization.
  - Cross-encoder reranking produces high-precision semantic similarity probabilities via Sigmoid activation.

---

## 3. Concurrency, Memory & Build Architecture

1. **Thread-Safe Model Storage:** Sessions are wrapped in `Arc<Mutex<Session>>` or `Arc<Session>` allowing parallel asynchronous access across Tokio worker threads.
2. **Zero Memory Duplication:** Audio buffers and tokens utilize `bytes::Bytes` and zero-copy slice representations wherever possible.
3. **MSVC Runtime Linkage:**
   - Configured via `.cargo/config.toml`:
     ```toml
     [target.x86_64-pc-windows-msvc]
     rustflags = ["-C", "target-feature=-crt-static"]
     ```
   - Sets dynamic C-runtime linking (`/MD`) to match pre-compiled ONNX Runtime DLLs (`onnxruntime.dll`), eliminating CRT mismatch linker errors.
