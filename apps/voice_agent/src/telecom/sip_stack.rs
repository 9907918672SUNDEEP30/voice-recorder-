//! # RFC 3261 Native SIP (Session Initiation Protocol) Stack
//!
//! Enterprise carrier-grade SIP User Agent (UAS/UAC) over UDP.
//! Directly integrates with telecom SIP trunks (Twilio Elastic SIP Trunking, Exotel, Tata Tele, Asterisk).
//!
//! Zero external proxy or softswitch dependency required.

#![allow(dead_code)]

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, Mutex};

use super::rtp_stream::RtpSession;
use crate::pipeline::PipelineOrchestrator;
use crate::supabase::SupabaseClient;
use ai_core::models::adapters::{LlmAdapter, SttAdapter, TtsAdapter, VADAdapter};

/// Standard SIP Methods
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SipMethod {
    Invite,
    Ack,
    Bye,
    Cancel,
    Options,
    Register,
    Other(String),
}

impl SipMethod {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "INVITE" => Self::Invite,
            "ACK" => Self::Ack,
            "BYE" => Self::Bye,
            "CANCEL" => Self::Cancel,
            "OPTIONS" => Self::Options,
            "REGISTER" => Self::Register,
            other => Self::Other(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Invite => "INVITE",
            Self::Ack => "ACK",
            Self::Bye => "BYE",
            Self::Cancel => "CANCEL",
            Self::Options => "OPTIONS",
            Self::Register => "REGISTER",
            Self::Other(s) => s.as_str(),
        }
    }
}

/// Parsed SDP (Session Description Protocol) Payload (RFC 4566)
#[derive(Debug, Clone)]
pub struct SdpSession {
    pub connection_ip: String,
    pub audio_port: u16,
    pub codecs: Vec<u8>,
}

impl SdpSession {
    /// Parses essential media attributes from an SDP text body
    pub fn parse(sdp: &str) -> Result<Self> {
        let mut connection_ip = String::new();
        let mut audio_port = 0u16;
        let mut codecs = Vec::new();

        for line in sdp.lines() {
            let line = line.trim();
            if line.starts_with("c=IN IP4 ") {
                connection_ip = line[9..].trim().to_string();
            } else if line.starts_with("m=audio ") {
                let parts: Vec<&str> = line[8..].split_whitespace().collect();
                if parts.len() >= 3 {
                    audio_port = parts[0].parse::<u16>().unwrap_or(0);
                    for &pt in &parts[2..] {
                        if let Ok(num) = pt.parse::<u8>() {
                            codecs.push(num);
                        }
                    }
                }
            }
        }

        if connection_ip.is_empty() || audio_port == 0 {
            return Err(anyhow!("Invalid SDP: missing connection IP or audio port"));
        }

        Ok(Self {
            connection_ip,
            audio_port,
            codecs,
        })
    }

    /// Generates local 200 OK SDP answer for PCMU (PT=0, 8000Hz)
    pub fn generate_answer(local_ip: &str, local_rtp_port: u16) -> String {
        format!(
            "v=0\r\n\
            o=ShambhaviVoice 1000 1000 IN IP4 {ip}\r\n\
            s=VoiceSession\r\n\
            c=IN IP4 {ip}\r\n\
            t=0 0\r\n\
            m=audio {port} RTP/AVP 0 101\r\n\
            a=rtpmap:0 PCMU/8000\r\n\
            a=rtpmap:101 telephone-event/8000\r\n\
            a=fmtp:101 0-16\r\n\
            a=sendrecv\r\n\
            a=ptime:20\r\n",
            ip = local_ip,
            port = local_rtp_port
        )
    }
}

/// Parsed SIP Message (Request or Response)
#[derive(Debug, Clone)]
pub struct SipMessage {
    pub is_request: bool,
    pub method: Option<SipMethod>,
    pub uri: Option<String>,
    pub status_code: Option<u16>,
    pub reason: Option<String>,
    pub via: Vec<String>,
    pub from: String,
    pub to: String,
    pub call_id: String,
    pub cseq: String,
    pub contact: Option<String>,
    pub content_type: Option<String>,
    pub body: String,
}

impl SipMessage {
    /// Parses an RFC 3261 SIP datagram
    pub fn parse(raw: &str) -> Result<Self> {
        let mut lines = raw.lines();
        let first_line = lines.next().ok_or_else(|| anyhow!("Empty SIP message"))?.trim();

        let is_request;
        let mut method = None;
        let mut uri = None;
        let mut status_code = None;
        let mut reason = None;

        if first_line.starts_with("SIP/2.0 ") {
            is_request = false;
            let parts: Vec<&str> = first_line.splitn(3, ' ').collect();
            if parts.len() < 3 {
                return Err(anyhow!("Malformed SIP response status line: {}", first_line));
            }
            status_code = Some(parts[1].parse::<u16>()?);
            reason = Some(parts[2].to_string());
        } else {
            is_request = true;
            let parts: Vec<&str> = first_line.split_whitespace().collect();
            if parts.len() < 3 {
                return Err(anyhow!("Malformed SIP request line: {}", first_line));
            }
            method = Some(SipMethod::from_str(parts[0]));
            uri = Some(parts[1].to_string());
        }

        let mut via = Vec::new();
        let mut from = String::new();
        let mut to = String::new();
        let mut call_id = String::new();
        let mut cseq = String::new();
        let mut contact = None;
        let mut content_type = None;
        let mut content_length = 0usize;

        let mut in_body = false;
        let mut body = String::new();

        for line in lines {
            if in_body {
                body.push_str(line);
                body.push_str("\r\n");
                continue;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                in_body = true;
                continue;
            }

            if let Some((header_name, header_val)) = trimmed.split_once(':') {
                let name = header_name.trim().to_lowercase();
                let val = header_val.trim();

                match name.as_str() {
                    "via" | "v" => via.push(val.to_string()),
                    "from" | "f" => from = val.to_string(),
                    "to" | "t" => to = val.to_string(),
                    "call-id" | "i" => call_id = val.to_string(),
                    "cseq" => cseq = val.to_string(),
                    "contact" | "m" => contact = Some(val.to_string()),
                    "content-type" | "c" => content_type = Some(val.to_string()),
                    "content-length" | "l" => {
                        content_length = val.parse::<usize>().unwrap_or(0);
                    }
                    _ => {}
                }
            }
        }

        if content_length > 0 && body.len() > content_length {
            body.truncate(content_length);
        }

        Ok(Self {
            is_request,
            method,
            uri,
            status_code,
            reason,
            via,
            from,
            to,
            call_id,
            cseq,
            contact,
            content_type,
            body,
        })
    }

    /// Extracts clean caller number from From header (e.g. `sip:+919876543210@...`)
    pub fn extract_caller_number(&self) -> Option<String> {
        let from = &self.from;
        if let Some(start) = from.find("sip:") {
            let after = &from[start + 4..];
            if let Some(end) = after.find('@') {
                return Some(after[..end].to_string());
            }
        }
        None
    }

    /// Generates standard SIP Response to this request
    pub fn build_response(
        &self,
        status_code: u16,
        reason: &str,
        body: Option<&str>,
        local_contact: &str,
    ) -> String {
        let mut resp = format!("SIP/2.0 {} {}\r\n", status_code, reason);

        // Mirror Via headers exactly as received
        for v in &self.via {
            resp.push_str(&format!("Via: {}\r\n", v));
        }

        resp.push_str(&format!("From: {}\r\n", self.from));

        // Add to-tag if not present in request To header and response is 200 OK or Ringing
        let to_header = if !self.to.contains(";tag=") && status_code >= 180 {
            format!("{};tag=shambhavi_{}", self.to, 0x54321)
        } else {
            self.to.clone()
        };
        resp.push_str(&format!("To: {}\r\n", to_header));

        resp.push_str(&format!("Call-ID: {}\r\n", self.call_id));
        resp.push_str(&format!("CSeq: {}\r\n", self.cseq));
        resp.push_str(&format!("Contact: <{}>\r\n", local_contact));
        resp.push_str("User-Agent: Shambhavi-AI-Core-SIP/1.0\r\n");

        if let Some(content) = body {
            resp.push_str("Content-Type: application/sdp\r\n");
            resp.push_str(&format!("Content-Length: {}\r\n\r\n", content.len()));
            resp.push_str(content);
        } else {
            resp.push_str("Content-Length: 0\r\n\r\n");
        }

        resp
    }
}

/// Active Call Tracking Record
pub struct ActiveCallSession {
    pub call_id: String,
    pub caller_number: Option<String>,
    pub remote_rtp_addr: SocketAddr,
    pub rtp_session: Arc<RtpSession>,
    pub orchestrator: Option<Arc<PipelineOrchestrator>>,
}

/// Native SIP Signaling Server
pub struct SipServer {
    socket: Arc<UdpSocket>,
    local_ip: String,
    local_port: u16,
    calls: Arc<Mutex<HashMap<String, ActiveCallSession>>>,
    vad: Arc<tokio::sync::Mutex<dyn VADAdapter>>,
    stt: Arc<dyn SttAdapter>,
    llm: Arc<dyn LlmAdapter>,
    tts: Arc<dyn TtsAdapter>,
    supabase: Arc<SupabaseClient>,
}

impl SipServer {
    /// Binds SIP server on specified port (default 5060)
    pub async fn bind(
        port: u16,
        local_ip: &str,
        vad: Arc<tokio::sync::Mutex<dyn VADAdapter>>,
        stt: Arc<dyn SttAdapter>,
        llm: Arc<dyn LlmAdapter>,
        tts: Arc<dyn TtsAdapter>,
        supabase: Arc<SupabaseClient>,
    ) -> Result<Self> {
        let bind_addr = format!("0.0.0.0:{}", port);
        let socket = UdpSocket::bind(&bind_addr).await?;
        let local_port = socket.local_addr()?.port();

        println!("📞 [SIP Server] Native telecom stack listening on UDP 0.0.0.0:{}", local_port);

        Ok(Self {
            socket: Arc::new(socket),
            local_ip: local_ip.to_string(),
            local_port,
            calls: Arc::new(Mutex::new(HashMap::new())),
            vad,
            stt,
            llm,
            tts,
            supabase,
        })
    }

    /// Local port
    pub fn local_port(&self) -> u16 {
        self.local_port
    }

    /// Starts processing SIP messages
    pub async fn run(&self) -> Result<()> {
        let mut buf = [0u8; 4096];

        loop {
            let (len, remote_peer) = self.socket.recv_from(&mut buf).await?;
            let raw = String::from_utf8_lossy(&buf[..len]);

            match SipMessage::parse(&raw) {
                Ok(msg) => {
                    let _ = self.handle_sip_message(msg, remote_peer).await;
                }
                Err(e) => {
                    eprintln!("⚠️ [SIP Server] Bad datagram from {}: {}", remote_peer, e);
                }
            }
        }
    }

    /// Handles an incoming SIP request or response
    async fn handle_sip_message(&self, msg: SipMessage, peer: SocketAddr) -> Result<()> {
        let contact = format!("sip:voice@{}:{}", self.local_ip, self.local_port);

        match msg.method {
            Some(SipMethod::Invite) => {
                println!("📞 [SIP Server] Inbound INVITE from {} (Call-ID: {})", msg.from, msg.call_id);

                // 1. Send 100 Trying
                let trying = msg.build_response(100, "Trying", None, &contact);
                let _ = self.socket.send_to(trying.as_bytes(), peer).await;

                // 2. Parse SDP
                let sdp = match SdpSession::parse(&msg.body) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("❌ [SIP Server] SDP parse failed: {}", e);
                        let bad_sdp = msg.build_response(488, "Not Acceptable Here", None, &contact);
                        let _ = self.socket.send_to(bad_sdp.as_bytes(), peer).await;
                        return Ok(());
                    }
                };

                let remote_rtp: SocketAddr = format!("{}:{}", sdp.connection_ip, sdp.audio_port)
                    .parse()
                    .unwrap_or(peer);

                // 3. Allocate local RTP Session
                let rtp = Arc::new(RtpSession::bind(0, remote_rtp, 0x5348414D).await?);
                let local_rtp_port = rtp.local_port();

                // 4. Send 180 Ringing
                let ringing = msg.build_response(180, "Ringing", None, &contact);
                let _ = self.socket.send_to(ringing.as_bytes(), peer).await;

                // 5. Send 200 OK with SDP Answer
                let answer_sdp = SdpSession::generate_answer(&self.local_ip, local_rtp_port);
                let ok_resp = msg.build_response(200, "OK", Some(&answer_sdp), &contact);
                let _ = self.socket.send_to(ok_resp.as_bytes(), peer).await;

                // 6. Record session in calls map
                let caller = msg.extract_caller_number();
                let mut calls = self.calls.lock().await;
                calls.insert(
                    msg.call_id.clone(),
                    ActiveCallSession {
                        call_id: msg.call_id,
                        caller_number: caller,
                        remote_rtp_addr: remote_rtp,
                        rtp_session: rtp,
                        orchestrator: None,
                    },
                );
            }

            Some(SipMethod::Ack) => {
                println!("✅ [SIP Server] Received ACK for Call-ID: {}", msg.call_id);

                let mut calls = self.calls.lock().await;
                if let Some(session) = calls.get_mut(&msg.call_id) {
                    let (inbound_tx, inbound_rx) = mpsc::channel::<Vec<u8>>(100);
                    let (telephony_tx, mut telephony_rx) = mpsc::channel::<String>(100);

                    // Start incoming RTP receiver loop
                    session.rtp_session.start_receiver(inbound_tx);

                    // Create pipeline orchestrator
                    let orchestrator = Arc::new(PipelineOrchestrator::new(
                        telephony_tx,
                        format!("sip_{}", session.call_id),
                        self.vad.clone(),
                        self.stt.clone(),
                        self.llm.clone(),
                        self.tts.clone(),
                        self.supabase.clone(),
                    ));

                    if let Some(caller) = &session.caller_number {
                        orchestrator
                            .set_call_metadata(
                                &format!("sip_stream_{}", session.call_id),
                                &session.call_id,
                                Some(caller),
                                None,
                            )
                            .await;
                    }

                    orchestrator.clone().start_pipeline(inbound_rx);
                    orchestrator.clone().trigger_initial_greeting();

                    // Bridge outbound audio from orchestrator to carrier via RTP
                    let rtp_tx = session.rtp_session.clone();
                    tokio::spawn(async move {
                        while let Some(msg_json) = telephony_rx.recv().await {
                            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&msg_json) {
                                if v["event"] == "media" {
                                    if let Some(b64_payload) = v["media"]["payload"].as_str() {
                                        use base64::{Engine as _, engine::general_purpose::STANDARD as b64};
                                        if let Ok(mulaw_bytes) = b64.decode(b64_payload) {
                                            for chunk in mulaw_bytes.chunks(160) {
                                                let _ = rtp_tx.send_frame(chunk).await;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    });

                    session.orchestrator = Some(orchestrator);
                }
            }

            Some(SipMethod::Bye) => {
                println!("📴 [SIP Server] BYE received for Call-ID: {}", msg.call_id);

                let ok_resp = msg.build_response(200, "OK", None, &contact);
                let _ = self.socket.send_to(ok_resp.as_bytes(), peer).await;

                let mut calls = self.calls.lock().await;
                if let Some(session) = calls.remove(&msg.call_id) {
                    session.rtp_session.stop();
                    if let Some(orch) = session.orchestrator {
                        orch.handle_call_ended().await;
                    }
                }
            }

            Some(SipMethod::Options) => {
                // Heartbeat/ping from trunk provider
                let ok_resp = msg.build_response(200, "OK", None, &contact);
                let _ = self.socket.send_to(ok_resp.as_bytes(), peer).await;
            }

            Some(SipMethod::Cancel) => {
                let ok_resp = msg.build_response(200, "OK", None, &contact);
                let _ = self.socket.send_to(ok_resp.as_bytes(), peer).await;

                let mut calls = self.calls.lock().await;
                if let Some(session) = calls.remove(&msg.call_id) {
                    session.rtp_session.stop();
                }
            }

            _ => {
                // Return 501 Not Implemented for unsupported methods
                let resp = msg.build_response(501, "Not Implemented", None, &contact);
                let _ = self.socket.send_to(resp.as_bytes(), peer).await;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sip_invite_parsing() {
        let raw_invite = "INVITE sip:voice@10.0.0.1:5060 SIP/2.0\r\n\
            Via: SIP/2.0/UDP 192.168.1.50:5060;branch=z9hG4bK-3321-1\r\n\
            From: \"Alice\" <sip:+919876543210@carrier.com>;tag=12345\r\n\
            To: <sip:voice@10.0.0.1:5060>\r\n\
            Call-ID: call-998877@192.168.1.50\r\n\
            CSeq: 1 INVITE\r\n\
            Contact: <sip:alice@192.168.1.50:5060>\r\n\
            Content-Type: application/sdp\r\n\
            Content-Length: 128\r\n\
            \r\n\
            v=0\r\n\
            o=Alice 100 100 IN IP4 192.168.1.50\r\n\
            s=Talk\r\n\
            c=IN IP4 192.168.1.50\r\n\
            t=0 0\r\n\
            m=audio 16400 RTP/AVP 0 101\r\n";

        let msg = SipMessage::parse(raw_invite).expect("Failed to parse valid SIP INVITE");
        assert!(msg.is_request);
        assert_eq!(msg.method, Some(SipMethod::Invite));
        assert_eq!(msg.call_id, "call-998877@192.168.1.50");
        assert_eq!(msg.extract_caller_number(), Some("+919876543210".to_string()));

        // Test SDP extraction
        let sdp = SdpSession::parse(&msg.body).expect("Failed to parse SDP");
        assert_eq!(sdp.connection_ip, "192.168.1.50");
        assert_eq!(sdp.audio_port, 16400);
        assert_eq!(sdp.codecs, vec![0, 101]);

        // Test Response generation
        let resp = msg.build_response(200, "OK", Some("v=0\r\n"), "sip:voice@10.0.0.1:5060");
        assert!(resp.starts_with("SIP/2.0 200 OK\r\n"));
        assert!(resp.contains("Via: SIP/2.0/UDP 192.168.1.50:5060;branch=z9hG4bK-3321-1\r\n"));
        assert!(resp.contains("Call-ID: call-998877@192.168.1.50\r\n"));
    }

    #[test]
    fn test_sip_options_ping() {
        let raw_options = "OPTIONS sip:10.0.0.1:5060 SIP/2.0\r\n\
            Via: SIP/2.0/UDP 192.168.1.1:5060;branch=z9hG4bK-ping\r\n\
            From: <sip:trunk@carrier.com>;tag=p1\r\n\
            To: <sip:10.0.0.1:5060>\r\n\
            Call-ID: ping-123\r\n\
            CSeq: 100 OPTIONS\r\n\
            Content-Length: 0\r\n\
            \r\n";

        let msg = SipMessage::parse(raw_options).expect("Failed to parse OPTIONS");
        assert_eq!(msg.method, Some(SipMethod::Options));
        let resp = msg.build_response(200, "OK", None, "sip:voice@10.0.0.1:5060");
        assert!(resp.starts_with("SIP/2.0 200 OK\r\n"));
    }
}
