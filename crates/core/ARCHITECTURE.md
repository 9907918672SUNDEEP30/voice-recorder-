# AI Core Framework Architecture

## 1. High-Level Overview
The **AI Core Framework** (`crates/core`) is the foundational ML/AI engine of the system. It handles all ONNX runtime executions, complex tensor mathematics, neural network integrations, and provides unified Rust traits (`adapters.rs`) for the application layer.

---

## 2. Component Blueprint (`onnx_adapters.rs`)

### A. Listening Core (VAD)
**Implementation:** `SileroVadLocal`
*   **Purpose:** Detects human voice vs. silence from raw audio chunks.
*   **Input:** 16-bit PCM chunks (Float32).
*   **Process:** Normalizes PCM to [-1.0, 1.0], passes through Silero VAD ONNX model, maintains internal LSTM state tensors across sequential chunks.
*   **Output:** Speaking probability & boolean flag (`is_speaking`).

### B. Understanding Core (STT)
**Implementation:** `WhisperTinyLocal`
*   **Purpose:** Converts human speech into text.
*   **Input:** Complete PCM Audio buffers.
*   **Process:** 
    1. Computes 80-bin Log-Mel Spectrogram via FFT math.
    2. Runs ONNX Encoder (encoder_model.onnx).
    3. Runs Autoregressive Greedy Decoding loop (decoder_model.onnx) to generate tokens.
    4. Decodes tokens to text using HuggingFace tokenizer.
*   **Output:** Transcribed Text.

### C. Brain / Reasoning Core (LLM)
**Implementation:** `QwenLocal`
*   **Purpose:** The central intelligence, generates conversational responses and analyzes images.
*   **Input:** ChatML formatted messages.
*   **Process:** Tokenizes text -> Runs Qwen ONNX autoregressive loop -> Streams decoded tokens. Supports Vision models for multi-modal analysis (OCR/Image Analysis).
*   **Output:** Stream of Text Tokens.

### D. Voice Core (TTS)
**Implementation:** `OpenVoiceLocal`
*   **Purpose:** Converts generated text back into cloned human-like speech.
*   **Input:** Text stream.
*   **Process:** 
    1. Native phonemization of text (to Piper Phoneme IDs).
    2. Base TTS (Piper ONNX) generates neutral audio tensor.
    3. Tone Extract ONNX model extracts a Tone Embedding vector from reference audio (cached in RAM).
    4. Tone Color ONNX injects the Tone Embedding into the neutral audio.
*   **Output:** PCM f32 Audio Tensor -> converted to bytes.

### E. RAG Core (Embeddings)
**Implementation:** `BgeEmbeddingLocal` & `BgeRerankerLocal`
*   **Purpose:** For dense vector embeddings and cross-encoder scoring.
