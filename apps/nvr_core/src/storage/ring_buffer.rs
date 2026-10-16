//! In-Memory Rolling Circular Ring Buffer
//! Maintains past 30–60 seconds of video packets in RAM for instant pre-incident evidence clamping.

use bytes::Bytes;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Single compressed video or audio packet stored in the rolling buffer
#[derive(Debug, Clone)]
pub struct BufferedPacket {
    pub is_keyframe: bool,
    pub timestamp_ms: u64,
    pub received_at: Instant,
    pub payload: Bytes,
}

/// Circular ring buffer for a single camera stream
#[derive(Debug)]
pub struct CameraRingBuffer {
    pub camera_id: String,
    pub max_duration: Duration,
    buffer: VecDeque<BufferedPacket>,
    total_bytes: usize,
}

impl CameraRingBuffer {
    /// Create a new ring buffer with maximum retention duration (e.g. 45 seconds)
    pub fn new(camera_id: impl Into<String>, max_duration: Duration) -> Self {
        Self {
            camera_id: camera_id.into(),
            max_duration,
            buffer: VecDeque::with_capacity(1200), // ~45s @ 25fps
            total_bytes: 0,
        }
    }

    /// Push a newly demuxed packet into the circular buffer and prune expired packets
    pub fn push(&mut self, packet: BufferedPacket) {
        self.total_bytes += packet.payload.len();
        self.buffer.push_back(packet);

        let now = Instant::now();
        // Remove packets older than max_duration
        while let Some(front) = self.buffer.front() {
            if now.duration_since(front.received_at) > self.max_duration {
                if let Some(removed) = self.buffer.pop_front() {
                    self.total_bytes = self.total_bytes.saturating_sub(removed.payload.len());
                }
            } else {
                break;
            }
        }
    }

    /// Clamp and clone the entire pre-incident buffer starting from the nearest keyframe
    pub fn clamp_pre_incident(&self) -> Vec<BufferedPacket> {
        // Find the first keyframe so the clamped clip is cleanly decodable
        let first_keyframe_idx = self.buffer.iter().position(|p| p.is_keyframe).unwrap_or(0);
        self.buffer.iter().skip(first_keyframe_idx).cloned().collect()
    }

    /// Current packet count in the buffer
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Total memory occupied in RAM by this camera's buffer in bytes
    pub fn memory_usage_bytes(&self) -> usize {
        self.total_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_retention() {
        let mut rb = CameraRingBuffer::new("cam_01", Duration::from_millis(50));
        let p1 = BufferedPacket {
            is_keyframe: true,
            timestamp_ms: 0,
            received_at: Instant::now(),
            payload: Bytes::from_static(b"frame1"),
        };
        rb.push(p1);
        assert_eq!(rb.len(), 1);

        std::thread::sleep(Duration::from_millis(60));

        let p2 = BufferedPacket {
            is_keyframe: false,
            timestamp_ms: 60,
            received_at: Instant::now(),
            payload: Bytes::from_static(b"frame2"),
        };
        rb.push(p2);

        // Older packet p1 should have expired
        assert_eq!(rb.len(), 1);
        assert_eq!(rb.clamp_pre_incident()[0].timestamp_ms, 60);
    }
}
