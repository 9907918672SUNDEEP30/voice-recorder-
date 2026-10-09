use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_util::sync::CancellationToken;
use bytes::Bytes;

use ai_core::models::adapters::{SessionState, VADAdapter, SttAdapter, LlmAdapter, TtsAdapter};
use crate::codec::G711Codec;

pub struct PipelineOrchestrator {
    state: Arc<AtomicUsize>, // 0: Idle, 1: Listening, 2: Thinking, 3: Speaking
    current_cancel_token: Arc<Mutex<CancellationToken>>,
    barge_in_enabled: bool,
    stream_sid: String,
    
    // Channels
    pub telephony_ws_tx: mpsc::Sender<String>,

    // ML Adapters
    vad: Arc<Mutex<dyn VADAdapter>>,
    stt: Arc<dyn SttAdapter>,
    llm: Arc<dyn LlmAdapter>,
    tts: Arc<dyn TtsAdapter>,
    history: Arc<Mutex<Vec<(String, String)>>>,
}

impl PipelineOrchestrator {
    pub fn new(
        telephony_ws_tx: mpsc::Sender<String>, 
        stream_sid: String,
        vad: Arc<Mutex<dyn VADAdapter>>,
        stt: Arc<dyn SttAdapter>,
        llm: Arc<dyn LlmAdapter>,
        tts: Arc<dyn TtsAdapter>,
    ) -> Self {
        Self {
            state: Arc::new(AtomicUsize::new(0)), // Start at Idle
            current_cancel_token: Arc::new(Mutex::new(CancellationToken::new())),
            barge_in_enabled: true,
            stream_sid,
            telephony_ws_tx,
            vad,
            stt,
            llm,
            tts,
            history: Arc::new(Mutex::new(Vec::new())),
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

    pub async fn handle_barge_in(&self) {
        if !self.barge_in_enabled || self.get_state() != SessionState::Speaking {
            return;
        }

        println!("[Pipeline] Barge-in triggered! Cancelling current TTS and LLM tasks.");

        let mut token_guard = self.current_cancel_token.lock().await;
        token_guard.cancel(); // Cancel ongoing generation/TTS
        *token_guard = CancellationToken::new(); // Reset for next turn

        // Send Clear Frame to Telephony WebSocket to instantly stop playing audio
        let clear_frame = serde_json::json!({
            "event": "clear",
            "streamSid": self.stream_sid
        });
        let _ = self.telephony_ws_tx.send(clear_frame.to_string()).await;

        self.set_state(SessionState::Listening);
    }

    pub async fn get_cancel_token(&self) -> CancellationToken {
        self.current_cancel_token.lock().await.clone()
    }

    /// Starts the main loop for ingesting audio and processing the pipeline
    pub fn start_pipeline(self: Arc<Self>, mut inbound_audio_rx: mpsc::Receiver<Vec<u8>>) {
        tokio::spawn(async move {
            let mut audio_buffer = Vec::new();
            let mut is_currently_speaking = false;
            let mut silence_chunks = 0;

            println!("[Pipeline] Ready and waiting for audio...");

            while let Some(pcm_chunk) = inbound_audio_rx.recv().await {
                // 1. VAD Check (Listening Phase)
                let mut vad = self.vad.lock().await;
                let (is_speech, _prob) = vad.process_chunk(&pcm_chunk).await.unwrap_or((false, 0.0));
                drop(vad);

                if is_speech {
                    if self.get_state() == SessionState::Speaking {
                        // User interrupted while Agent was speaking
                        self.handle_barge_in().await;
                        audio_buffer.clear(); // Discard previous context if barged in
                    }
                    
                    self.set_state(SessionState::Listening);
                    is_currently_speaking = true;
                    silence_chunks = 0;
                    audio_buffer.extend_from_slice(&pcm_chunk);
                    
                    // -- LOCAL STREAMING STT (Sliding Window) --
                    // Every few chunks, process the audio buffer through Whisper to get partial text
                    // This keeps STT working in real-time while the user is still speaking.
                    if audio_buffer.len() % (5 * pcm_chunk.len()) == 0 {
                        let partial_audio = audio_buffer.clone();
                        let stt_clone = self.stt.clone();
                        tokio::spawn(async move {
                            if let Ok(partial_text) = stt_clone.transcribe(&partial_audio).await {
                                let txt = partial_text.trim();
                                if !txt.is_empty() {
                                    println!("[Pipeline] STT (Real-time Partial): {}", txt);
                                }
                            }
                        });
                    }
                    
                } else if is_currently_speaking {
                    silence_chunks += 1;
                    audio_buffer.extend_from_slice(&pcm_chunk);

                    // If silence continues for approx ~1 second (assuming chunk size), end of speech
                    if silence_chunks > 20 { 
                        is_currently_speaking = false;
                        self.set_state(SessionState::Thinking);
                        
                        let audio_to_process = audio_buffer.clone();
                        audio_buffer.clear();
                        
                        let self_clone = self.clone();
                        tokio::spawn(async move {
                            self_clone.process_turn(audio_to_process).await;
                        });
                    }
                }
            }
        });
    }

    /// Processes a completed utterance: STT -> LLM -> TTS -> Network
    async fn process_turn(&self, audio: Vec<u8>) {
        println!("[Pipeline] Speech detected. Running STT...");
        
        // 1. STT (Understanding Phase)
        let transcript = match self.stt.transcribe(&audio).await {
            Ok(t) => t.trim().to_string(),
            Err(e) => {
                println!("[Pipeline] STT Error: {}", e);
                self.set_state(SessionState::Idle);
                return;
            }
        };
        
        println!("[Pipeline] User said: {}", transcript);
        if transcript.is_empty() {
            self.set_state(SessionState::Idle);
            return;
        }

        // 2. AMD (Answering Machine / Robotic Voice Cut-off Logic)
        let lower_t = transcript.to_lowercase();
        if lower_t.contains("leave a message") || lower_t.contains("voicemail") || lower_t.contains("beep") {
            println!("🚨 [Pipeline] Robotic Voice/Voicemail Detected! Hanging up...");
            let hangup_frame = serde_json::json!({
                "event": "mark",
                "streamSid": self.stream_sid,
                "name": "hangup" // Mocking a hangup marker or Twilio clear/hangup
            });
            let _ = self.telephony_ws_tx.send(hangup_frame.to_string()).await;
            self.set_state(SessionState::Idle);
            return;
        }

        // 3. LLM (Thinking Phase)
        println!("[Pipeline] Sending to Brain (Qwen)...");
        
        // Save user message to history
        let mut history_guard = self.history.lock().await;
        history_guard.push(("user".to_string(), transcript.clone()));
        let messages = history_guard.clone();
        drop(history_guard); // Drop early to not block

        let cancel_token = self.get_cancel_token().await;
        
        // Hardcoded testing prompt (Short-Term Memory)
        let system_prompt = "तुम एक टेलीकॉलर हो। तुम्हारा नाम दिव्या है। तुम 'दिव्या मिलन' (divyamilan.in) से बात कर रही हो, जो कि सिंधी समाज के लिए एक बेहतरीन मैट्रिमोनियल वेबसाइट है। तुम संदीप जी से बात कर रही हो। तुम्हारा लक्ष्य बहुत ही शालीनता और मधुरता से संदीप जी को दिव्या मिलन की प्रीमियम मेंबरशिप के फायदों के बारे में बताना है। अगर वे सवाल पूछें तो बहुत ही छोटे और स्वाभाविक वाक्यों में जवाब दो। बिल्कुल इंसानों की तरह बात करो, रोबोट की तरह नहीं। जवाब छोटे रखो ताकि बातचीत रियल लगे।";

        let mut llm_rx = match self.llm.generate_stream(system_prompt, messages, cancel_token.clone()).await {
            Ok(rx) => rx,
            Err(e) => {
                println!("[Pipeline] LLM Error: {}", e);
                self.set_state(SessionState::Idle);
                return;
            }
        };

        // 4. TTS (Speaking Phase) - STREAMING SENTENCE BY SENTENCE
        self.set_state(SessionState::Speaking);
        
        let mut current_sentence = String::new();
        let (audio_tx, mut audio_rx) = mpsc::channel(100);
        
        // 5. Output via WebSocket (Streaming back to user instantly as audio chunks arrive)
        let stream_sid_clone = self.stream_sid.clone();
        let telephony_ws_tx_clone = self.telephony_ws_tx.clone();
        let cancel_token_out = cancel_token.clone();
        tokio::spawn(async move {
            use base64::{Engine as _, engine::general_purpose::STANDARD as b64};
            while let Some(audio_bytes) = audio_rx.recv().await {
                if cancel_token_out.is_cancelled() {
                    break;
                }
                let mulaw_data = G711Codec::pcm_16k_to_mulaw_8k(&audio_bytes);
                let payload = b64.encode(&mulaw_data);
                let media_frame = serde_json::json!({
                    "event": "media",
                    "streamSid": stream_sid_clone,
                    "media": { "payload": payload }
                });
                let _ = telephony_ws_tx_clone.send(media_frame.to_string()).await;
            }
        });

        let mut full_agent_response = String::new();
        // Brain to TTS Hand-off (Instant 100ms processing per sentence)
        while let Some(chunk) = llm_rx.recv().await {
            if cancel_token.is_cancelled() {
                println!("[Pipeline] Brain task cancelled due to Barge-in.");
                return;
            }
            current_sentence.push_str(&chunk);
            full_agent_response.push_str(&chunk);
            
            // Check for sentence boundaries
            if current_sentence.contains('.') || current_sentence.contains('!') || current_sentence.contains('?') || current_sentence.contains('।') {
                let sentence_to_speak = current_sentence.clone();
                current_sentence.clear();
                
                println!("[Pipeline] Agent generated partial sentence: {}", sentence_to_speak);
                
                let tts_clone = self.tts.clone();
                let audio_tx_clone = audio_tx.clone();
                let token_clone = cancel_token.clone();
                
                // Fire TTS immediately for this sentence
                tokio::spawn(async move {
                    let _ = tts_clone.synthesize_stream(&sentence_to_speak, audio_tx_clone, token_clone).await;
                });
            }
        }
        
        // Process any leftover text
        if !current_sentence.trim().is_empty() && !cancel_token.is_cancelled() {
            println!("[Pipeline] Agent generated partial sentence: {}", current_sentence);
            let tts_clone = self.tts.clone();
            let audio_tx_clone = audio_tx.clone();
            let token_clone = cancel_token.clone();
            tokio::spawn(async move {
                let _ = tts_clone.synthesize_stream(&current_sentence, audio_tx_clone, token_clone).await;
            });
        }

        if !cancel_token.is_cancelled() {
            // Save agent message to history
            let mut history_guard = self.history.lock().await;
            history_guard.push(("assistant".to_string(), full_agent_response));
            
            self.set_state(SessionState::Idle);
            println!("[Pipeline] Turn completed. Returning to Idle.");
        }
    }
}
