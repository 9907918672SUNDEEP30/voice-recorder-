use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionState {
    Idle,
    Listening,
    Thinking,
    Speaking,
}

#[derive(Debug, Clone)]
pub enum PipelineEvent {
    UserSpeechStart,
    UserTranscript { text: String, is_final: bool },
    AgentThinking,
    AgentTranscriptChunk(String),
    AgentSpeaking,
    BargeInTriggered,
    TurnCompleted {
        vad_latency_ms: u64,
        stt_latency_ms: u64,
        ttft_ms: u64,
        tts_first_chunk_ms: u64,
        total_latency_ms: u64,
    },
}

#[async_trait]
pub trait VADAdapter: Send + Sync {
    /// Process raw PCM chunk and return a tuple: (is_speech, speech_prob)
    async fn process_chunk(&mut self, pcm: &[u8]) -> Result<(bool, f32)>;
}

#[async_trait]
pub trait SttAdapter: Send + Sync {
    /// Transcribe PCM bytes (Whisper-tiny ONNX)
    async fn transcribe(&self, pcm_data: &[u8]) -> Result<String>;
}

#[async_trait]
pub trait LlmAdapter: Send + Sync {
    /// Streams tokens from Qwen ONNX
    async fn generate_stream(
        &self,
        system_prompt: &str,
        messages: Vec<(String, String)>,
        cancel_token: CancellationToken,
    ) -> Result<mpsc::Receiver<String>>;
}

#[async_trait]
pub trait TtsAdapter: Send + Sync {
    /// Synthesizes text chunk to PCM using Kokoro ONNX
    async fn synthesize_stream(
        &self,
        text: &str,
        audio_tx: mpsc::Sender<Bytes>,
        cancel_token: CancellationToken,
    ) -> Result<()>;
}
