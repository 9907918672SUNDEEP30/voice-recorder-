//! Telephony Gateway & WebSocket Server
//!
//! Provides the primary audio streaming and control gateway for Twilio, Exotel,
//! and the Browser Testing Client. Supports the standard Twilio Media Streams
//! specification (Start, Media, Stop, Clear, Mark events) with base64 G.711 μ-law audio.

use std::sync::Arc;
use tokio::sync::Mutex;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::mpsc;
use base64::{Engine as _, engine::general_purpose::STANDARD as b64};

use crate::pipeline::PipelineOrchestrator;
use crate::codec::G711Codec;
use crate::supabase::SupabaseClient;
use ai_core::models::adapters::{VADAdapter, SttAdapter, LlmAdapter, TtsAdapter};

#[derive(Clone)]
pub struct AppState {
    pub vad: Arc<Mutex<dyn VADAdapter>>,
    pub stt: Arc<dyn SttAdapter>,
    pub llm: Arc<dyn LlmAdapter>,
    pub tts: Arc<dyn TtsAdapter>,
    pub supabase: Arc<SupabaseClient>,
}

pub async fn start_telephony_server(
    vad: Arc<Mutex<dyn VADAdapter>>,
    stt: Arc<dyn SttAdapter>,
    llm: Arc<dyn LlmAdapter>,
    tts: Arc<dyn TtsAdapter>,
    supabase: Arc<SupabaseClient>,
) -> anyhow::Result<()> {
    let state = AppState { vad, stt, llm, tts, supabase };

    let app = Router::new()
        .route("/ws/telephony", get(ws_handler))
        .route("/health", get(health_handler))
        .route("/", get(test_client_handler))
        .route("/test", get(test_client_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("📞 Telephony Gateway listening on ws://0.0.0.0:3000/ws/telephony");
    println!("🌐 Browser Testing Dashboard available at http://localhost:3000/");
    
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_handler() -> impl IntoResponse {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "service": "voice_ai_calling_agent",
        "telephony_protocol": "twilio_media_streams_v1",
        "codec": "audio/x-mulaw 8000Hz"
    }))
}

async fn test_client_handler() -> Html<&'static str> {
    Html(include_str!("../test_client.html"))
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    println!("🔌 [Telephony Gateway] New caller connection established.");
    let (mut sender, mut receiver) = socket.split();

    // Channels for orchestrator communication
    let (telephony_tx, mut telephony_rx) = mpsc::channel::<String>(100);
    let (inbound_audio_tx, inbound_audio_rx) = mpsc::channel::<Vec<u8>>(100);
    
    // Create the orchestrator with shared models & Supabase logger
    let orchestrator = Arc::new(PipelineOrchestrator::new(
        telephony_tx, 
        "stream_init".to_string(),
        state.vad.clone(),
        state.stt.clone(),
        state.llm.clone(),
        state.tts.clone(),
        state.supabase.clone(),
    ));

    // Start full-duplex orchestrator listening loop
    orchestrator.clone().start_pipeline(inbound_audio_rx);

    let orchestrator_recv = orchestrator.clone();

    // Task 1: Handle incoming messages (Twilio Media Streams / Exotel / Browser Simulator)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                        let event = val["event"].as_str().unwrap_or("");
                        match event {
                            "connected" => {
                                println!("📞 [Telephony] Connected frame received: protocol={}", val["protocol"]);
                            }
                            "start" => {
                                let stream_sid = val["streamSid"].as_str()
                                    .or_else(|| val["start"]["streamSid"].as_str())
                                    .unwrap_or("browser_stream_1");
                                let call_sid = val["start"]["callSid"].as_str().unwrap_or("call_direct");
                                let caller = val["start"]["customParameters"]["caller"].as_str()
                                    .or_else(|| val["start"]["from"].as_str());
                                let name = val["start"]["customParameters"]["name"].as_str()
                                    .or_else(|| val["start"]["callerName"].as_str());

                                println!("🚀 [Telephony] Call Started! StreamSid: {}, CallSid: {}, Caller: {:?}", stream_sid, call_sid, caller);
                                orchestrator_recv.set_call_metadata(stream_sid, call_sid, caller, name).await;
                                // Immediately trigger initial greeting (Divya speaks first!)
                                orchestrator_recv.clone().trigger_initial_greeting();
                            }
                            "text_turn" | "text" => {
                                if let Some(user_text) = val["text"].as_str() {
                                    println!("💬 [Telephony] Direct text message from caller: {}", user_text);
                                    orchestrator_recv.clone().process_text_turn(user_text.to_string());
                                }
                            }
                            "media" => {
                                if let Some(b64_payload) = val["media"]["payload"].as_str() {
                                    if let Ok(mulaw_bytes) = b64.decode(b64_payload.trim()) {
                                        // 8kHz μ-law -> 16kHz PCM for VAD & STT
                                        let pcm16 = G711Codec::mulaw_8k_to_pcm_16k(&mulaw_bytes);
                                        let _ = inbound_audio_tx.send(pcm16).await;
                                    }
                                }
                            }
                            "stop" => {
                                println!("🛑 [Telephony] Stop event received. Remote party ended call.");
                                orchestrator_recv.handle_call_ended().await;
                                break;
                            }
                            "mark" => {
                                // Marker confirmation from Twilio
                            }
                            _ => {
                                // Other metadata frames
                            }
                        }
                    }
                }
                Message::Binary(data) => {
                    // Fallback for raw binary test clients: decode 8kHz μ-law to 16kHz PCM
                    let pcm = G711Codec::mulaw_8k_to_pcm_16k(&data);
                    let _ = inbound_audio_tx.send(pcm).await;
                }
                Message::Close(_) => {
                    println!("🚪 [Telephony] WebSocket connection closed by client.");
                    orchestrator_recv.handle_call_ended().await;
                    break;
                }
                _ => {}
            }
        }
        orchestrator_recv.handle_call_ended().await;
    });

    // Task 2: Handle outgoing messages to Telephony provider (Twilio Media / Clear / Mark frames)
    let mut send_task = tokio::spawn(async move {
        while let Some(out_msg) = telephony_rx.recv().await {
            if sender.send(Message::Text(out_msg)).await.is_err() {
                break;
            }
        }
    });

    tokio::select! {
        _ = (&mut recv_task) => send_task.abort(),
        _ = (&mut send_task) => recv_task.abort(),
    };
    
    println!("📴 [Telephony Gateway] Telephony session finished.");
}
