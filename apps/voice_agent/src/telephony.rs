use std::sync::Arc;
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

struct AppState {
    // In a real app, this would be a map of active sessions
}

pub async fn start_telephony_server() -> anyhow::Result<()> {
    let state = Arc::new(AppState {});

    let app = Router::new()
        .route("/ws/telephony", get(ws_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Telephony WebSocket server listening on ws://0.0.0.0:3000/ws/telephony");
    
    axum::serve(listener, app).await?;
    Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade, State(_state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(socket: WebSocket) {
    println!("New telephony connection established.");
    let (mut sender, mut receiver) = socket.split();

    // Channels for orchestrator communication
    let (telephony_tx, mut telephony_rx) = mpsc::channel::<String>(100);
    let (inbound_audio_tx, mut _inbound_audio_rx) = mpsc::channel::<Vec<u8>>(100);
    
    let _orchestrator = PipelineOrchestrator::new(telephony_tx, "session_1".to_string());

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
