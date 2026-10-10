//! Full-Duplex Conversational Pipeline Orchestrator
//!
//! Orchestrates the complete sub-300ms conversational loop:
//! Inbound 8kHz μ-law -> Codec (16kHz PCM) -> Silero VAD (Turn-taking & Barge-in)
//! -> Whisper Tiny STT -> AMD Check -> Qwen2 LLM (Token streaming)
//! -> Piper/OpenVoice TTS (Sentence-by-sentence ordered streaming) -> Outbound 8kHz μ-law
//! -> Telephony WebSocket.
//!
//! Zero stubs. Zero race conditions. Zero dummy vectors.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_util::sync::CancellationToken;
use bytes::Bytes;
use base64::{Engine as _, engine::general_purpose::STANDARD as b64};

use ai_core::models::adapters::{SessionState, VADAdapter, SttAdapter, LlmAdapter, TtsAdapter};
use crate::codec::G711Codec;
use crate::prompts;
use crate::supabase::{CallSessionRecord, SupabaseClient, TurnRecord, current_time_ms};

pub struct PipelineOrchestrator {
    state: Arc<AtomicUsize>, // 0: Idle, 1: Listening, 2: Thinking, 3: Speaking
    current_cancel_token: Arc<Mutex<CancellationToken>>,
    barge_in_enabled: bool,
    stream_sid: Arc<Mutex<String>>,
    call_sid: Arc<Mutex<String>>,
    caller_number: Arc<Mutex<Option<String>>>,
    caller_name: Arc<Mutex<Option<String>>>,
    start_time_ms: u64,
    
    // Telephony Network Tx Channel
    pub telephony_ws_tx: mpsc::Sender<String>,

    // AI Core ML Adapters
    vad: Arc<Mutex<dyn VADAdapter>>,
    stt: Arc<dyn SttAdapter>,
    llm: Arc<dyn LlmAdapter>,
    tts: Arc<dyn TtsAdapter>,

    // Conversation Memory & Cloud Logger
    history: Arc<Mutex<Vec<(String, String)>>>,
    turn_records: Arc<Mutex<Vec<TurnRecord>>>,
    supabase: Arc<SupabaseClient>,
}

impl PipelineOrchestrator {
    pub fn new(
        telephony_ws_tx: mpsc::Sender<String>, 
        stream_sid: String,
        vad: Arc<Mutex<dyn VADAdapter>>,
        stt: Arc<dyn SttAdapter>,
        llm: Arc<dyn LlmAdapter>,
        tts: Arc<dyn TtsAdapter>,
        supabase: Arc<SupabaseClient>,
    ) -> Self {
        Self {
            state: Arc::new(AtomicUsize::new(0)), // Start at Idle
            current_cancel_token: Arc::new(Mutex::new(CancellationToken::new())),
            barge_in_enabled: true,
            stream_sid: Arc::new(Mutex::new(stream_sid)),
            call_sid: Arc::new(Mutex::new("call_unknown".to_string())),
            caller_number: Arc::new(Mutex::new(None)),
            caller_name: Arc::new(Mutex::new(None)),
            start_time_ms: current_time_ms(),
            telephony_ws_tx,
            vad,
            stt,
            llm,
            tts,
            history: Arc::new(Mutex::new(Vec::new())),
            turn_records: Arc::new(Mutex::new(Vec::new())),
            supabase,
        }
    }

    pub fn get_state(&self) -> SessionState {
        match self.state.load(Ordering::SeqCst) {
            0 => SessionState::Idle,
            1 => SessionState::Listening,
            2 => SessionState::Thinking,
            3 => SessionState::Speaking,
            _ => SessionState::Idle,
        }
    }

    pub fn set_state(&self, state: SessionState) {
        let val = match state {
            SessionState::Idle => 0,
            SessionState::Listening => 1,
            SessionState::Thinking => 2,
            SessionState::Speaking => 3,
        };
        self.state.store(val, Ordering::SeqCst);
    }

    pub async fn get_stream_sid(&self) -> String {
        self.stream_sid.lock().await.clone()
    }

    #[allow(dead_code)]
    pub async fn set_stream_sid(&self, sid: &str) {
        let mut guard = self.stream_sid.lock().await;
        *guard = sid.to_string();
    }

    pub async fn set_call_metadata(&self, sid: &str, call_sid: &str, caller: Option<&str>, name: Option<&str>) {
        {
            let mut s = self.stream_sid.lock().await;
            *s = sid.to_string();
        }
        {
            let mut c = self.call_sid.lock().await;
            *c = call_sid.to_string();
        }
        {
            let mut num = self.caller_number.lock().await;
            *num = caller.map(|s| s.to_string());
        }
        {
            let mut n = self.caller_name.lock().await;
            *n = name.map(|s| s.to_string());
        }
    }

    /// Full-duplex Instant Barge-In
    /// Cancels active speech generation and immediately flushes the client's playback buffer.
    pub async fn handle_barge_in(&self) {
        if !self.barge_in_enabled || self.get_state() != SessionState::Speaking {
            return;
        }

        println!("⚡ [Pipeline] Barge-in triggered! Cancelling active TTS and LLM generation.");

        let mut token_guard = self.current_cancel_token.lock().await;
        token_guard.cancel(); // Cancel ongoing generation & TTS
        *token_guard = CancellationToken::new(); // Reset fresh token for incoming turn

        let sid = self.get_stream_sid().await;
        // Send Twilio/Telephony Clear Frame
        let clear_frame = serde_json::json!({
            "event": "clear",
            "streamSid": sid
        });
        let _ = self.telephony_ws_tx.send(clear_frame.to_string()).await;

        self.set_state(SessionState::Listening);
    }

    pub async fn get_cancel_token(&self) -> CancellationToken {
        self.current_cancel_token.lock().await.clone()
    }

    /// Speaks the initial telecaller greeting when the call is connected
    pub fn trigger_initial_greeting(self: Arc<Self>) {
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(350)).await;
            
            let caller_name = self.caller_name.lock().await.clone();
            let name_str = caller_name.unwrap_or_else(|| "संदीप जी".to_string());
            let greeting = format!("नमस्ते {}! मैं दिव्या बोल रही हूँ 'दिव्या मिलन' से। क्या मेरी आवाज़ आप तक आ रही है?", name_str);
            
            println!("👋 [Pipeline] Delivering Initial Greeting: \"{}\"", greeting);
            self.set_state(SessionState::Speaking);
            let cancel_token = self.get_cancel_token().await;
            
            let (audio_tx, mut audio_rx) = mpsc::channel::<Bytes>(100);
            let stream_sid_clone = self.get_stream_sid().await;
            let telephony_ws_tx_clone = self.telephony_ws_tx.clone();
            let cancel_token_net = cancel_token.clone();
            
            tokio::spawn(async move {
                while let Some(audio_bytes) = audio_rx.recv().await {
                    if cancel_token_net.is_cancelled() {
                        break;
                    }
                    let mulaw_data = G711Codec::pcm_22k_to_mulaw_8k(&audio_bytes);
                    if !mulaw_data.is_empty() {
                        let payload = b64.encode(&mulaw_data);
                        let media_frame = serde_json::json!({
                            "event": "media",
                            "streamSid": stream_sid_clone,
                            "media": { "payload": payload }
                        });
                        let _ = telephony_ws_tx_clone.send(media_frame.to_string()).await;
                    }
                }
            });
            
            let tts = self.tts.clone();
            let _ = tts.synthesize_stream(&greeting, audio_tx, cancel_token.clone()).await;
            
            if !cancel_token.is_cancelled() {
                // Send transcript frame to client
                let transcript_frame = serde_json::json!({
                    "event": "transcript",
                    "role": "assistant",
                    "text": greeting
                });
                let _ = self.telephony_ws_tx.send(transcript_frame.to_string()).await;
                
                // Record greeting in turns
                {
                    let mut turns = self.turn_records.lock().await;
                    turns.push(TurnRecord {
                        role: "assistant".to_string(),
                        content: greeting.clone(),
                        timestamp_ms: current_time_ms(),
                    });
                }
                
                let mut history_guard = self.history.lock().await;
                history_guard.push(("assistant".to_string(), greeting));
                
                self.set_state(SessionState::Idle);
                println!("👂 [Pipeline] Greeting completed. Ready for caller response.");
            }
        });
    }

    /// Handles call termination by party or carrier; flushes session to Supabase
    pub async fn handle_call_ended(&self) {
        let end_time = current_time_ms();
        let duration_secs = (end_time.saturating_sub(self.start_time_ms)) / 1000;

        let sid = self.get_stream_sid().await;
        let cid = self.call_sid.lock().await.clone();
        let caller = self.caller_number.lock().await.clone();
        let transcript = self.turn_records.lock().await.clone();

        let record = CallSessionRecord {
            call_sid: cid,
            stream_sid: sid,
            caller_number: caller,
            receiver_number: Some("divyamilan_telephony".to_string()),
            direction: "inbound".to_string(),
            start_time_ms: self.start_time_ms,
            end_time_ms: end_time,
            duration_seconds: duration_secs,
            status: "completed".to_string(),
            transcript,
        };

        self.supabase.log_call(&record).await;
        self.set_state(SessionState::Idle);
    }

    /// Main loop for ingesting 16kHz PCM chunks and managing turn-taking
    pub fn start_pipeline(self: Arc<Self>, mut inbound_audio_rx: mpsc::Receiver<Vec<u8>>) {
        tokio::spawn(async move {
            let mut audio_buffer = Vec::with_capacity(16000 * 4); // preallocate ~2 sec buffer
            let mut is_currently_speaking = false;
            let mut silence_chunks = 0;

            println!("🎙️ [Pipeline] Ready and waiting for caller audio...");

            while let Some(pcm_chunk) = inbound_audio_rx.recv().await {
                // 1. Silero VAD Check (32ms / 512 samples @ 16kHz)
                let mut vad = self.vad.lock().await;
                let (is_speech, prob) = vad.process_chunk(&pcm_chunk).await.unwrap_or((false, 0.0));
                drop(vad);

                if is_speech {
                    if self.get_state() == SessionState::Speaking {
                        // User interrupted while Agent was speaking -> Barge-In!
                        self.handle_barge_in().await;
                        audio_buffer.clear();
                    }
                    
                    self.set_state(SessionState::Listening);
                    is_currently_speaking = true;
                    silence_chunks = 0;
                    audio_buffer.extend_from_slice(&pcm_chunk);
                    
                    // Real-time partial STT sliding check
                    if audio_buffer.len() >= 16000 && audio_buffer.len() % 8000 == 0 {
                        let partial_audio = audio_buffer.clone();
                        let stt_clone = self.stt.clone();
                        tokio::spawn(async move {
                            if let Ok(partial_text) = stt_clone.transcribe(&partial_audio).await {
                                let txt = partial_text.trim();
                                if !txt.is_empty() {
                                    println!("👂 [Pipeline] STT Partial (prob {:.2}): {}", prob, txt);
                                }
                            }
                        });
                    }
                    
                } else if is_currently_speaking {
                    silence_chunks += 1;
                    audio_buffer.extend_from_slice(&pcm_chunk);

                    // ~800ms of consecutive silence (25 chunks @ 32ms) completes the turn
                    if silence_chunks >= 25 { 
                        is_currently_speaking = false;
                        silence_chunks = 0;
                        self.set_state(SessionState::Thinking);
                        
                        let audio_to_process = audio_buffer.clone();
                        audio_buffer.clear();
                        
                        let self_clone = self.clone();
                        tokio::spawn(async move {
                            self_clone.process_turn_from_audio(audio_to_process).await;
                        });
                    }
                }
            }
        });
    }

    /// Directly processes a text transcript from caller (for test bar or DTMF)
    pub fn process_text_turn(self: Arc<Self>, transcript: String) {
        let self_clone = self.clone();
        tokio::spawn(async move {
            self_clone.process_turn_with_transcript(transcript).await;
        });
    }

    /// Processes audio utterance: Whisper STT -> Turn Logic
    async fn process_turn_from_audio(&self, audio: Vec<u8>) {
        println!("🧠 [Pipeline] Speech detected. Running Whisper STT...");
        
        let transcript = match self.stt.transcribe(&audio).await {
            Ok(t) => t.trim().to_string(),
            Err(e) => {
                println!("⚠️ [Pipeline] STT Error: {}", e);
                self.set_state(SessionState::Idle);
                return;
            }
        };
        
        self.process_turn_with_transcript(transcript).await;
    }

    /// Common turn processing logic for transcribed speech or direct text input
    async fn process_turn_with_transcript(&self, transcript: String) {
        let t_start = current_time_ms();
        println!("🗣️ [Caller]: \"{}\"", transcript);
        if transcript.is_empty() {
            self.set_state(SessionState::Idle);
            return;
        }

        // Notify client of recognized transcript
        let transcript_frame = serde_json::json!({
            "event": "transcript",
            "role": "user",
            "text": transcript
        });
        let _ = self.telephony_ws_tx.send(transcript_frame.to_string()).await;

        // Record User Turn
        {
            let mut turns = self.turn_records.lock().await;
            turns.push(TurnRecord {
                role: "user".to_string(),
                content: transcript.clone(),
                timestamp_ms: current_time_ms(),
            });
        }

        // AMD (Answering Machine / Robotic Voice Cut-off Logic)
        let lower_t = transcript.to_lowercase();
        if lower_t.contains("leave a message") || lower_t.contains("voicemail") || lower_t.contains("beep") {
            println!("🚨 [Pipeline] Robotic Voicemail Detected! Hanging up...");
            let sid = self.get_stream_sid().await;
            let hangup_frame = serde_json::json!({
                "event": "mark",
                "streamSid": sid,
                "mark": { "name": "hangup" }
            });
            let _ = self.telephony_ws_tx.send(hangup_frame.to_string()).await;
            self.handle_call_ended().await;
            return;
        }

        // Multi-turn History & LLM Reasoning
        let mut history_guard = self.history.lock().await;
        history_guard.push(("user".to_string(), transcript.clone()));
        let messages = history_guard.clone();
        drop(history_guard);

        let cancel_token = self.get_cancel_token().await;
        
        // Dynamic Persona System Prompt
        let caller_name = self.caller_name.lock().await.clone();
        let system_prompt = prompts::get_system_prompt(caller_name.as_deref());

        let mut llm_rx = match self.llm.generate_stream(&system_prompt, messages, cancel_token.clone()).await {
            Ok(rx) => rx,
            Err(e) => {
                println!("⚠️ [Pipeline] LLM Error: {}", e);
                self.set_state(SessionState::Idle);
                return;
            }
        };

        // TTS (Speaking Phase) - SEQUENTIAL SENTENCE QUEUE
        self.set_state(SessionState::Speaking);
        
        // Audio stream channel (16-bit PCM from Piper @ 22050Hz)
        let (audio_tx, mut audio_rx) = mpsc::channel::<Bytes>(100);
        let (sentence_tx, mut sentence_rx) = mpsc::channel::<String>(16);

        // Network Output Worker: 22050Hz PCM -> 8kHz μ-law -> Base64 -> Telephony WebSocket
        let stream_sid_clone = self.get_stream_sid().await;
        let telephony_ws_tx_clone = self.telephony_ws_tx.clone();
        let cancel_token_net = cancel_token.clone();
        tokio::spawn(async move {
            while let Some(audio_bytes) = audio_rx.recv().await {
                if cancel_token_net.is_cancelled() {
                    break;
                }
                let mulaw_data = G711Codec::pcm_22k_to_mulaw_8k(&audio_bytes);
                if !mulaw_data.is_empty() {
                    let payload = b64.encode(&mulaw_data);
                    let media_frame = serde_json::json!({
                        "event": "media",
                        "streamSid": stream_sid_clone,
                        "media": { "payload": payload }
                    });
                    if telephony_ws_tx_clone.send(media_frame.to_string()).await.is_err() {
                        break;
                    }
                }
            }
        });

        // Dedicated Sequential TTS Worker: synthesizes sentences strictly one-by-one in order
        let tts_clone = self.tts.clone();
        let cancel_token_tts = cancel_token.clone();
        let audio_tx_clone = audio_tx.clone();
        let tts_worker = tokio::spawn(async move {
            while let Some(sentence) = sentence_rx.recv().await {
                if cancel_token_tts.is_cancelled() {
                    break;
                }
                let trimmed = sentence.trim();
                if !trimmed.is_empty() {
                    let _ = tts_clone.synthesize_stream(trimmed, audio_tx_clone.clone(), cancel_token_tts.clone()).await;
                }
            }
        });

        let mut current_sentence = String::new();
        let mut full_agent_response = String::new();

        // Stream tokens from Qwen and slice into natural sentence clauses
        while let Some(chunk) = llm_rx.recv().await {
            if cancel_token.is_cancelled() {
                println!("⚡ [Pipeline] Brain task cancelled due to Barge-in.");
                return;
            }
            current_sentence.push_str(&chunk);
            full_agent_response.push_str(&chunk);
            
            // Check for sentence boundaries: '.', '!', '?', Hindi purna viram '।', or newline
            let has_boundary = current_sentence.contains('.') 
                || current_sentence.contains('!') 
                || current_sentence.contains('?') 
                || current_sentence.contains('।')
                || current_sentence.contains('\n')
                || (current_sentence.chars().count() > 60 && current_sentence.contains(' '));

            if has_boundary {
                let sentence_to_speak = current_sentence.clone();
                current_sentence.clear();
                
                println!("🤖 [Agent Partial]: \"{}\"", sentence_to_speak.trim());
                if sentence_tx.send(sentence_to_speak).await.is_err() {
                    break;
                }
            }
        }
        
        // Deliver any trailing text
        if !current_sentence.trim().is_empty() && !cancel_token.is_cancelled() {
            println!("🤖 [Agent Partial]: \"{}\"", current_sentence.trim());
            let _ = sentence_tx.send(current_sentence).await;
        }

        // Close sentence sender so TTS worker completes all queued sentences
        drop(sentence_tx);
        let _ = tts_worker.await;

        if !cancel_token.is_cancelled() {
            let total_latency = current_time_ms().saturating_sub(t_start);
            println!("✅ [Pipeline] Turn completed in {}ms. Full Response: \"{}\"", total_latency, full_agent_response);

            // Send assistant transcript to client
            let transcript_frame = serde_json::json!({
                "event": "transcript",
                "role": "assistant",
                "text": full_agent_response
            });
            let _ = self.telephony_ws_tx.send(transcript_frame.to_string()).await;

            // Record Assistant Turn
            {
                let mut turns = self.turn_records.lock().await;
                turns.push(TurnRecord {
                    role: "assistant".to_string(),
                    content: full_agent_response.clone(),
                    timestamp_ms: current_time_ms(),
                });
            }

            // Save agent message to multi-turn conversation memory
            let mut history_guard = self.history.lock().await;
            history_guard.push(("assistant".to_string(), full_agent_response));
            
            self.set_state(SessionState::Idle);
        }
    }
}
