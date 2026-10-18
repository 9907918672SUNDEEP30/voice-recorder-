//! # RFC 3550 Real-time Transport Protocol (RTP) Engine
//!
//! Provides deterministic RTP packetization, depacketization, and streaming
//! for carrier-grade telephony (Twilio Elastic SIP Trunking, Exotel, Asterisk, FreeSWITCH).
//!
//! Default payload format: G.711 μ-law (PCMU, Payload Type 0, 8000Hz, 20ms frames).

#![allow(dead_code)]

use anyhow::{anyhow, Result};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

use crate::codec::G711Codec;

/// Standard 12-byte RTP fixed header (RFC 3550 Section 5.1)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtpHeader {
    pub version: u8,         // 2 bits (must be 2)
    pub padding: bool,       // 1 bit
    pub extension: bool,     // 1 bit
    pub csrc_count: u8,      // 4 bits
    pub marker: bool,        // 1 bit
    pub payload_type: u8,    // 7 bits (0 = PCMU / G.711 μ-law)
    pub sequence_number: u16, // 16 bits
    pub timestamp: u32,      // 32 bits
    pub ssrc: u32,           // 32 bits
}

/// Parsed RTP Packet
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtpPacket {
    pub header: RtpHeader,
    pub payload: Vec<u8>,
}

impl RtpPacket {
    /// Creates a new RTP packet with G.711 μ-law payload (PT=0)
    pub fn new_pcmu(seq: u16, timestamp: u32, ssrc: u32, marker: bool, payload: Vec<u8>) -> Self {
        Self {
            header: RtpHeader {
                version: 2,
                padding: false,
                extension: false,
                csrc_count: 0,
                marker,
                payload_type: 0, // PCMU 8000Hz
                sequence_number: seq,
                timestamp,
                ssrc,
            },
            payload,
        }
    }

    /// Serializes RTP packet into RFC 3550 byte buffer
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(12 + self.payload.len());

        // Byte 0: V=2 (bit 6-7), P (bit 5), X (bit 4), CC (bit 0-3)
        let b0 = (self.header.version << 6)
            | ((self.header.padding as u8) << 5)
            | ((self.header.extension as u8) << 4)
            | (self.header.csrc_count & 0x0F);
        buf.push(b0);

        // Byte 1: M (bit 7), PT (bit 0-6)
        let b1 = ((self.header.marker as u8) << 7) | (self.header.payload_type & 0x7F);
        buf.push(b1);

        // Bytes 2-3: Sequence Number (big-endian)
        buf.extend_from_slice(&self.header.sequence_number.to_be_bytes());

        // Bytes 4-7: Timestamp (big-endian)
        buf.extend_from_slice(&self.header.timestamp.to_be_bytes());

        // Bytes 8-11: SSRC (big-endian)
        buf.extend_from_slice(&self.header.ssrc.to_be_bytes());

        // Payload
        buf.extend_from_slice(&self.payload);

        buf
    }

    /// Parses an RFC 3550 RTP packet from raw UDP datagram bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 12 {
            return Err(anyhow!("RTP packet too short: {} bytes (minimum 12)", data.len()));
        }

        let b0 = data[0];
        let version = (b0 >> 6) & 0x03;
        if version != 2 {
            return Err(anyhow!("Unsupported RTP version: {} (expected 2)", version));
        }

        let padding = (b0 & 0x20) != 0;
        let extension = (b0 & 0x10) != 0;
        let csrc_count = b0 & 0x0F;

        let b1 = data[1];
        let marker = (b1 & 0x80) != 0;
        let payload_type = b1 & 0x7F;

        let sequence_number = u16::from_be_bytes([data[2], data[3]]);
        let timestamp = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let ssrc = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);

        let mut offset = 12 + (csrc_count as usize * 4);
        if data.len() < offset {
            return Err(anyhow!("Datagram truncated in CSRC list"));
        }

        // Skip extension header if present
        if extension {
            if data.len() < offset + 4 {
                return Err(anyhow!("Datagram truncated in extension header"));
            }
            let ext_len = u16::from_be_bytes([data[offset + 2], data[offset + 3]]) as usize * 4;
            offset += 4 + ext_len;
            if data.len() < offset {
                return Err(anyhow!("Datagram truncated after extension header"));
            }
        }

        let mut payload_end = data.len();
        if padding {
            if payload_end <= offset {
                return Err(anyhow!("Malformed padding in RTP packet"));
            }
            let pad_count = data[payload_end - 1] as usize;
            if pad_count == 0 || pad_count > (payload_end - offset) {
                return Err(anyhow!("Invalid RTP padding count: {}", pad_count));
            }
            payload_end -= pad_count;
        }

        let payload = data[offset..payload_end].to_vec();

        Ok(Self {
            header: RtpHeader {
                version,
                padding,
                extension,
                csrc_count,
                marker,
                payload_type,
                sequence_number,
                timestamp,
                ssrc,
            },
            payload,
        })
    }
}

/// Bidirectional Real-Time Audio Streaming Session
pub struct RtpSession {
    socket: Arc<UdpSocket>,
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
    ssrc: u32,
    seq: AtomicU16,
    timestamp: AtomicU32,
    running: Arc<AtomicBool>,
}

impl RtpSession {
    /// Binds a new UDP socket on a random or requested port and initializes RTP session
    pub async fn bind(local_port: u16, remote_addr: SocketAddr, ssrc: u32) -> Result<Self> {
        let bind_addr = format!("0.0.0.0:{}", local_port);
        let socket = UdpSocket::bind(&bind_addr).await?;
        let local_addr = socket.local_addr()?;

        Ok(Self {
            socket: Arc::new(socket),
            local_addr,
            remote_addr,
            ssrc,
            seq: AtomicU16::new(100),
            timestamp: AtomicU32::new(16000),
            running: Arc::new(AtomicBool::new(true)),
        })
    }

    /// Local bound address
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Local bound port
    pub fn local_port(&self) -> u16 {
        self.local_addr.port()
    }

    /// Stops session streaming loops
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Spawns the incoming RTP receiver loop.
    /// Extracts G.711 μ-law bytes, upsamples to 16kHz linear PCM, and streams to `inbound_audio_tx`.
    pub fn start_receiver(
        &self,
        inbound_audio_tx: mpsc::Sender<Vec<u8>>,
    ) -> tokio::task::JoinHandle<()> {
        let socket = self.socket.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut buf = [0u8; 1500]; // Standard Ethernet MTU
            println!("📡 [RTP Stream] Receiver worker started on UDP {}", socket.local_addr().unwrap());

            while running.load(Ordering::SeqCst) {
                match socket.recv_from(&mut buf).await {
                    Ok((len, _from)) => {
                        if len < 12 {
                            continue;
                        }

                        if let Ok(rtp) = RtpPacket::parse(&buf[..len]) {
                            // Support PCMU (PT 0)
                            if rtp.header.payload_type == 0 {
                                // Convert 8kHz μ-law -> 16kHz linear PCM for Silero VAD / Whisper STT
                                let pcm_16k = G711Codec::mulaw_8k_to_pcm_16k(&rtp.payload);
                                if !pcm_16k.is_empty() {
                                    if inbound_audio_tx.send(pcm_16k).await.is_err() {
                                        break; // Receiver channel closed
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if running.load(Ordering::SeqCst) {
                            eprintln!("⚠️ [RTP Stream] UDP recv error: {}", e);
                        }
                        break;
                    }
                }
            }
            println!("📡 [RTP Stream] Receiver worker shut down cleanly.");
        })
    }

    /// Sends a 20ms G.711 μ-law audio frame (160 bytes) to carrier
    pub async fn send_frame(&self, mulaw_frame: &[u8]) -> Result<()> {
        let seq = self.seq.fetch_add(1, Ordering::SeqCst);
        let ts = self.timestamp.fetch_add(160, Ordering::SeqCst); // 160 samples per 20ms @ 8000Hz

        let packet = RtpPacket::new_pcmu(seq, ts, self.ssrc, false, mulaw_frame.to_vec());
        let bytes = packet.serialize();
        self.socket.send_to(&bytes, self.remote_addr).await?;
        Ok(())
    }

    /// Transmits linear PCM audio (from TTS) as a sequence of timed 20ms RTP packets.
    pub async fn transmit_audio_stream(
        &self,
        pcm_bytes: &[u8],
        in_sample_rate: u32,
    ) -> Result<()> {
        // 1. Convert to 8kHz μ-law bytes
        let mulaw = G711Codec::pcm_to_mulaw_8k(pcm_bytes, in_sample_rate);
        if mulaw.is_empty() {
            return Ok(());
        }

        // 2. Transmit in 160-byte chunks every 20 milliseconds
        let mut interval = tokio::time::interval(Duration::from_millis(20));
        for chunk in mulaw.chunks(160) {
            interval.tick().await;
            if !self.running.load(Ordering::SeqCst) {
                break;
            }

            let mut frame = chunk.to_vec();
            if frame.len() < 160 {
                frame.resize(160, 0xFF); // 0xFF is silence in μ-law
            }

            let _ = self.send_frame(&frame).await;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtp_packet_serialization_and_parsing() {
        let original_payload = vec![0x7E; 160]; // 20ms of silence
        let packet = RtpPacket::new_pcmu(1001, 32000, 0x12345678, true, original_payload.clone());

        let raw = packet.serialize();
        assert_eq!(raw.len(), 12 + 160);

        let parsed = RtpPacket::parse(&raw).expect("Must parse valid RTP packet");
        assert_eq!(parsed.header.version, 2);
        assert_eq!(parsed.header.payload_type, 0);
        assert_eq!(parsed.header.sequence_number, 1001);
        assert_eq!(parsed.header.timestamp, 32000);
        assert_eq!(parsed.header.ssrc, 0x12345678);
        assert!(parsed.header.marker);
        assert_eq!(parsed.payload, original_payload);
    }

    #[test]
    fn test_rtp_truncated_packet_error() {
        let short_data = vec![0x80, 0x00, 0x01]; // only 3 bytes
        assert!(RtpPacket::parse(&short_data).is_err());
    }

    #[tokio::test]
    async fn test_rtp_session_socket_binding() {
        let remote: SocketAddr = "127.0.0.1:20000".parse().unwrap();
        let session = RtpSession::bind(0, remote, 0xABCD1234)
            .await
            .expect("Failed to bind UDP RTP session");

        assert!(session.local_port() > 0);
        session.stop();
    }
}
