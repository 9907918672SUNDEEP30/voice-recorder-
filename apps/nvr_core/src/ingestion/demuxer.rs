//! Video Demuxing and Packet Analysis Subsystem
//! Inspects raw H.264 / H.265 NAL units to detect keyframes (IDR), SPS, and PPS headers.

use bytes::Bytes;

/// Type of H.264 NAL Unit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum H264NalType {
    NonIdrSlice, // P / B frame
    IdrSlice,    // I frame (Keyframe)
    Sps,         // Sequence Parameter Set
    Pps,         // Picture Parameter Set
    Sei,         // Supplemental Enhancement Information
    Other(u8),
}

/// Incoming Raw Packet from Ingestion Source
#[derive(Debug, Clone)]
pub struct RawStreamPacket {
    pub camera_id: String,
    pub timestamp_ms: u64,
    pub is_keyframe: bool,
    pub payload: Bytes,
}

pub struct PacketDemuxer;

impl PacketDemuxer {
    /// Detect if a raw H.264 byte buffer contains an IDR keyframe (NAL type 5)
    pub fn is_h264_keyframe(data: &[u8]) -> bool {
        let mut i = 0;
        while i + 4 < data.len() {
            // Check for start codes: 0x00 0x00 0x01 or 0x00 0x00 0x00 0x01
            if data[i] == 0 && data[i + 1] == 0 {
                let nal_byte = if data[i + 2] == 1 {
                    data[i + 3]
                } else if data[i + 2] == 0 && data[i + 3] == 1 && i + 4 < data.len() {
                    data[i + 4]
                } else {
                    i += 1;
                    continue;
                };

                let nal_type = nal_byte & 0x1F;
                if nal_type == 5 {
                    return true;
                }
            }
            i += 1;
        }
        false
    }
}
