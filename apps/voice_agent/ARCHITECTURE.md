# Voice AI Calling Agent Architecture

## 1. High-Level Overview
The **Voice Agent** (`apps/voice_agent`) is the application layer that sits on top of the AI Core Framework. It handles the networking, telephony protocols, state management, and orchestrates the AI pipeline to create a real-time, streaming AI conversational agent.

---

## 2. System Components

### A. Telephony Gateway (`telephony.rs`)
*   **Server:** Runs an Axum WebSocket server listening on `ws://0.0.0.0:3000/ws/telephony`.
*   **Ingestion:** Receives incoming WebSocket `Message::Binary` containing $\mu$-law audio.
*   **Decoding:** Decodes $\mu$-law payload to 16kHz PCM using `G711Codec`.
*   **Transmission:** Encodes outgoing PCM audio back to $\mu$-law and streams it to the telephony provider via base64 encoded JSON frames (`"event": "media"`).

### B. Pipeline Orchestrator (`pipeline.rs`)
*   **State Management:** Tracks real-time session state (`Idle`, `Listening`, `Thinking`, `Speaking`).
*   **Event Handling:** Listens to incoming decoded PCM and directs it through the ML adapters.

---

## 3. Data Flow & Workflow Blueprint

### The Main Loop (`start_pipeline`)
1. **Audio Ingestion**: WS -> $\mu$-law -> PCM16 -> `inbound_audio_rx`.
2. **VAD Check (Listening Phase)**:
    * PCM chunks are passed to `SileroVadLocal`.
    * If `is_speech` is true, the agent transitions to `Listening` and buffers audio.
    * If silence persists for ~1 second (20 chunks), it considers the user's turn over.
3. **STT (Understanding Phase)**:
    * The buffered audio is sent to `WhisperTinyLocal` for transcription.
4. **AMD (Robotic Voice / Voicemail Cut-off)**:
    * The STT text is analyzed. If words like "leave a message", "voicemail", or "beep" are detected, the Orchestrator sends a `"mark": "hangup"` frame to instantly cut the call.
5. **Brain (Thinking Phase)**:
    * Valid transcript is sent to `QwenLocal`.
    * The agent transitions to `Thinking` while waiting for the first tokens.
6. **TTS (Speaking Phase)**:
    * As the Brain streams text, it's sent to `OpenVoiceLocal`.
    * Audio chunks are generated, base64 encoded, and sent to the Telephony Gateway.

### Barge-In Flow (Interruption Handling)
* If the user speaks (VAD detects speech) while the State is `Speaking`:
    1. `handle_barge_in()` is invoked.
    2. The `CancellationToken` for the current TTs/LLM task is aborted.
    3. A new `CancellationToken` is spawned.
    4. A `"event": "clear"` frame is sent to the Telephony provider to flush the provider's audio buffer immediately.
    5. The agent switches back to `Listening`.

### Advanced Features & Memory
1. **Full-Duplex Streaming (Ultra-Low Latency)**:
    * **Streaming STT**: Implements a Sliding Window approach. As the user speaks, chunks are periodically sent to the STT model to generate partial transcripts in real-time, completely eliminating STT latency when the user stops speaking.
    * **Sentence-by-Sentence TTS**: The LLM streams tokens. As soon as a sentence boundary (`.`, `?`, `!`, `।`) is detected, that sentence is instantly handed off to the TTS engine and streamed to the user.
2. **Contextual Memory**:
    * **System Prompt**: A hardcoded persona (e.g., "Divya" from `divyamilan.in`) is injected at the start of the call.
    * **Conversation History**: The Orchestrator maintains an internal `history: Arc<Mutex<Vec<(String, String)>>>` to remember past turns within the active session, allowing for natural multi-turn conversations.
