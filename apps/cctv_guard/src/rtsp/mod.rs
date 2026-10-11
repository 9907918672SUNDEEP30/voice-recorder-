pub mod client;
pub mod frame;

pub use client::{RtspClient, RtspClientConfig, RtspStreamSource};
pub use frame::{CctvFrame, DownscaledFrame};
