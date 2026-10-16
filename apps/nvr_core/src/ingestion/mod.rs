//! Ingestion Subsystem for Shambhavi AI Labs Pehredar Box

pub mod demuxer;
pub mod stream_source;

pub use demuxer::{H264NalType, PacketDemuxer, RawStreamPacket};
pub use stream_source::{SimulatedCameraFeed, SimulationScenario, VideoFrame};
