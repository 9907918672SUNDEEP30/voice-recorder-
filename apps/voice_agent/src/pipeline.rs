use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use bytes::Bytes;

use crate::adapters::{SessionState, PipelineEvent};

pub struct PipelineOrchestrator {
    state: Arc<AtomicUsize>, // 0: Idle, 1: Listening, 2: Thinking, 3: Speaking
    current_cancel_token: CancellationToken,
    barge_in_enabled: bool,
    stream_sid: String,
    
    // Channels
    pub telephony_ws_tx: mpsc::Sender<String>,
}

impl PipelineOrchestrator {
    pub fn new(telephony_ws_tx: mpsc::Sender<String>, stream_sid: String) -> Self {
        Self {
            state: Arc::new(AtomicUsize::new(0)),
            current_cancel_token: CancellationToken::new(),
            barge_in_enabled: true,
            stream_sid,
            telephony_ws_tx,
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

    pub async fn handle_barge_in(&mut self, _audio_queue_rx: &mut mpsc::Receiver<Bytes>) {
        if !self.barge_in_enabled || self.get_state() != SessionState::Speaking {
            return;
        }

        println!("[Pipeline] Barge-in triggered! Cancelling current TTs and LLM tasks.");

        // 1. Cancel running LLM & TTS tasks
        self.current_cancel_token.cancel();

        // 2. Re-create a fresh CancellationToken for the new turn
        self.current_cancel_token = CancellationToken::new();

        // 3. Drain Outbound Audio Queue instantly
        while let Ok(_) = _audio_queue_rx.try_recv() {}

        // 4. Send Clear Frame to Telephony WebSocket
        let clear_frame = serde_json::json!({
            "event": "clear",
            "stream_sid": self.stream_sid
        });
        let _ = self.telephony_ws_tx.send(clear_frame.to_string()).await;

        // 5. Update state
        self.set_state(SessionState::Listening);
    }

    pub fn get_cancel_token(&self) -> CancellationToken {
        self.current_cancel_token.clone()
    }
}
