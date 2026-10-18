//! # Voice AI Agent Library
//!
//! Provides telephony gateways, native SIP/RTP telecom engine,
//! full-duplex conversational pipeline, G.711 μ-law codecs, and Supabase integration.

pub mod codec;
pub mod pipeline;
pub mod prompts;
pub mod supabase;
pub mod telecom;
pub mod telephony;

pub use ai_core::models::adapters::SessionState;
pub use codec::G711Codec;
pub use pipeline::PipelineOrchestrator;
pub use supabase::{CallSessionRecord, TurnRecord};
pub use telecom::{ActiveCallSession, RtpHeader, RtpPacket, RtpSession, SdpSession, SipMessage, SipMethod, SipServer};
pub use telephony::{start_telephony_server, AppState};
