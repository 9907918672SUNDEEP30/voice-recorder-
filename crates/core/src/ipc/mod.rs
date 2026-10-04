//! # Zero-Server IPC Bridge
//!
//! Standard IO stream and raw pipe communication layer without heavy web dependencies.

pub mod bridge;
pub mod message;

pub use bridge::IpcBridge;
pub use message::{InboundCommand, OutboundMessage};
