use std::sync::Arc;
use tokio::sync::Mutex;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::mpsc;
use crate::pipeline::PipelineOrchestrator;
use crate::codec::G711Codec;

use ai_core::models::adapters::{VADAdapter, SttAdapter, LlmAdapter, TtsAdapter};

#[derive(Clone)]
pub struct AppState {
    vad: Arc<Mutex<dyn VADAdapter>>,
    stt: Arc<dyn SttAdapter>,
    llm: Arc<dyn LlmAdapter>,
    tts: Arc<dyn TtsAdapter>,
}

pub async fn start_telephony_server(
    vad: Arc<Mutex<dyn VADAdapter>>,
    stt: Arc<dyn SttAdapter>,
    llm: Arc<dyn LlmAdapter>,
    tts: Arc<dyn TtsAdapter>,
) -> anyhow::Result<()> {
    let state = AppState { vad, stt, llm, tts };

    let app = Router::new()
        .route("/ws/telephony", get(ws_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Telephony WebSocket server listening on ws://0.0.0.0:3000/ws/telephony");
    
    axum::serve(listener, app).await?;
    Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    println!("New telephony connection established.");
    let (mut sender, mut receiver) = socket.split();

    // Channels for orchestrator communication
    let (telephony_tx, mut telephony_rx) = mpsc::channel::<String>(100);
    let (inbound_audio_tx, inbound_audio_rx) = mpsc::channel::<Vec<u8>>(100);
    
    // Create the orchestrator with the shared models
    let orchestrator = Arc::new(PipelineOrchestrator::new(
        telephony_tx, 
        "session_1".to_string(), // In real app, this comes from WS query params
        state.vad.clone(),
        state.stt.clone(),
        state.llm.clone(),
        state.tts.clone(),
    ));

    // Start the orchestrator's listening loop
    orchestrator.start_pipeline(inbound_audio_rx);

    // Task 1: Handle incoming WebSocket messages (from Telephony provider)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Binary(data) => {
                    // Incoming μ-law data, decode to PCM16
                    let pcm = G711Codec::mulaw_8k_to_pcm_16k(&data);
                    let _ = inbound_audio_tx.send(pcm).await;
                }
                Message::Text(text) => {
                    println!("Received Text Event: {}", text);
                }
                _ => {}
            }
        }
    });

    // Task 2: Handle outgoing WebSocket messages (to Telephony provider)
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
    
    println!("Telephony connection closed.");
}
