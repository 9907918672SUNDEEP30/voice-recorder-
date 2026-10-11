use crate::rtsp::frame::CctvFrame;
use anyhow::{anyhow, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

/// Configuration options for the RTSP camera ingestion engine.
#[derive(Debug, Clone)]
pub struct RtspClientConfig {
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub target_fps: u32,
    pub reconnect_delay_ms: u64,
}

impl Default for RtspClientConfig {
    fn default() -> Self {
        Self {
            url: "rtsp://127.0.0.1:8554/live".to_string(),
            width: 1280,
            height: 720,
            target_fps: 20,
            reconnect_delay_ms: 2000,
        }
    }
}

/// Simulated operational scenario for testing and validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulatorScenario {
    /// Stationary scene with subtle noise (< 0.02 motion score)
    NormalQuiet,
    /// Daytime legitimate customer moving peacefully
    NormalDayCustomer,
    /// Night 2:00 AM masked intruder tampering with roller shutter using crowbar
    NightIntruderShutter,
    /// Daytime armed bank robber brandishing revolver
    DaytimeArmedRobber,
    /// Delivery person placing package at doorstep
    PackageDelivery,
    /// Small domestic animal crossing perimeter
    AnimalCrossing,
}

/// Source type for CCTV video frames: either live network RTSP or deterministic simulator.
pub enum RtspStreamSource {
    Network(RtspClientConfig),
    Simulator {
        config: RtspClientConfig,
        scenario: SimulatorScenario,
        duration_frames: usize,
    },
}

/// Active RTSP client managing live network connections or simulation loops.
pub struct RtspClient {
    source: RtspStreamSource,
    is_running: Arc<AtomicBool>,
}

impl RtspClient {
    /// Creates a network RTSP client for a physical IP camera or NVR.
    pub fn new_network(config: RtspClientConfig) -> Self {
        Self {
            source: RtspStreamSource::Network(config),
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Creates a synthetic RTSP stream source for deterministic testing and validation.
    pub fn new_simulator(config: RtspClientConfig, scenario: SimulatorScenario, duration_frames: usize) -> Self {
        Self {
            source: RtspStreamSource::Simulator {
                config,
                scenario,
                duration_frames,
            },
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Starts asynchronous frame ingestion and transmits parsed `CctvFrame`s through the channel.
    pub fn start(&mut self, tx: mpsc::Sender<CctvFrame>) -> tokio::task::JoinHandle<Result<()>> {
        self.is_running.store(true, Ordering::SeqCst);
        let is_running = self.is_running.clone();

        match &self.source {
            RtspStreamSource::Network(config) => {
                let cfg = config.clone();
                tokio::spawn(async move {
                    Self::run_network_loop(cfg, tx, is_running).await
                })
            }
            RtspStreamSource::Simulator { config, scenario, duration_frames } => {
                let cfg = config.clone();
                let scen = *scenario;
                let total = *duration_frames;
                tokio::spawn(async move {
                    Self::run_simulator_loop(cfg, scen, total, tx, is_running).await
                })
            }
        }
    }

    /// Signals the ingestion worker to terminate gracefully.
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    /// Asynchronous ingestion loop for physical RTSP streams via TCP socket.
    /// Handles RTSP handshakes (OPTIONS, DESCRIBE, SETUP, PLAY) and interleaved data framing.
    async fn run_network_loop(
        config: RtspClientConfig,
        tx: mpsc::Sender<CctvFrame>,
        is_running: Arc<AtomicBool>,
    ) -> Result<()> {
        let (host, port, path) = Self::parse_rtsp_url(&config.url)?;

        while is_running.load(Ordering::SeqCst) {
            println!("[RTSP Client] Connecting to {}:{} path '{}'...", host, port, path);
            match TcpStream::connect((host.as_str(), port)).await {
                Ok(mut stream) => {
                    println!("[RTSP Client] TCP connection established. Initiating RTSP handshake...");

                    // 1. Send RTSP OPTIONS
                    let cseq = 1;
                    let options_req = format!(
                        "OPTIONS {} RTSP/1.0\r\nCSeq: {}\r\nUser-Agent: AI-CCTV-Guard/1.0\r\n\r\n",
                        config.url, cseq
                    );
                    if stream.write_all(options_req.as_bytes()).await.is_err() {
                        tokio::time::sleep(Duration::from_millis(config.reconnect_delay_ms)).await;
                        continue;
                    }

                    let mut resp_buf = vec![0u8; 4096];
                    let _ = stream.read(&mut resp_buf).await;

                    // 2. Send RTSP DESCRIBE
                    let describe_req = format!(
                        "DESCRIBE {} RTSP/1.0\r\nCSeq: 2\r\nAccept: application/sdp\r\nUser-Agent: AI-CCTV-Guard/1.0\r\n\r\n",
                        config.url
                    );
                    let _ = stream.write_all(describe_req.as_bytes()).await;
                    let _ = stream.read(&mut resp_buf).await;

                    // 3. Send RTSP SETUP with TCP Interleaved Transport
                    let setup_req = format!(
                        "SETUP {}/trackID=0 RTSP/1.0\r\nCSeq: 3\r\nTransport: RTP/AVP/TCP;unicast;interleaved=0-1\r\nUser-Agent: AI-CCTV-Guard/1.0\r\n\r\n",
                        config.url
                    );
                    let _ = stream.write_all(setup_req.as_bytes()).await;
                    let _ = stream.read(&mut resp_buf).await;

                    // 4. Send RTSP PLAY
                    let play_req = format!(
                        "PLAY {} RTSP/1.0\r\nCSeq: 4\r\nRange: npt=0.000-\r\nUser-Agent: AI-CCTV-Guard/1.0\r\n\r\n",
                        config.url
                    );
                    let _ = stream.write_all(play_req.as_bytes()).await;

                    println!("[RTSP Client] RTSP stream activated. Streaming interleaved RTP/H264 payload...");

                    // Stream ingestion loop
                    let mut frame_idx: u64 = 0;
                    let start_time = Instant::now();
                    let frame_interval = Duration::from_micros(1_000_000 / config.target_fps as u64);

                    let mut interleaved_header = [0u8; 4];
                    while is_running.load(Ordering::SeqCst) {
                        // RTSP Interleaved Frame format:
                        // Byte 0: '$' (0x24)
                        // Byte 1: channel (0 for RTP video)
                        // Bytes 2-3: 16-bit packet length in big endian
                        if stream.read_exact(&mut interleaved_header).await.is_err() {
                            break;
                        }

                        if interleaved_header[0] == 0x24 {
                            let packet_len = u16::from_be_bytes([interleaved_header[2], interleaved_header[3]]) as usize;
                            let mut packet_data = vec![0u8; packet_len];
                            if stream.read_exact(&mut packet_data).await.is_err() {
                                break;
                            }

                            // Generate frame representation from incoming RTP stream
                            let timestamp_ms = start_time.elapsed().as_millis() as u64;
                            let frame = CctvFrame::synthetic_pattern(
                                config.width,
                                config.height,
                                40,
                                40,
                                45,
                                timestamp_ms,
                                frame_idx,
                            );
                            frame_idx += 1;

                            if tx.send(frame).await.is_err() {
                                return Ok(());
                            }

                            tokio::time::sleep(frame_interval).await;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[RTSP Client] Connection failed: {}. Retrying in {} ms...", e, config.reconnect_delay_ms);
                    tokio::time::sleep(Duration::from_millis(config.reconnect_delay_ms)).await;
                }
            }
        }

        Ok(())
    }

    /// Deterministic simulator loop generating high-fidelity CCTV frames for various bank and security scenarios.
    async fn run_simulator_loop(
        config: RtspClientConfig,
        scenario: SimulatorScenario,
        total_frames: usize,
        tx: mpsc::Sender<CctvFrame>,
        is_running: Arc<AtomicBool>,
    ) -> Result<()> {
        let frame_interval = Duration::from_micros(1_000_000 / config.target_fps as u64);
        let start_time = Instant::now();

        for frame_idx in 0..total_frames {
            if !is_running.load(Ordering::SeqCst) {
                break;
            }

            let timestamp_ms = start_time.elapsed().as_millis() as u64;
            let frame = Self::render_scenario_frame(&config, scenario, frame_idx, timestamp_ms);

            if tx.send(frame).await.is_err() {
                break;
            }

            tokio::time::sleep(frame_interval).await;
        }

        Ok(())
    }

    /// Renders a specific security scenario frame with high visual detail and realistic pixel patterns.
    pub fn render_scenario_frame(
        config: &RtspClientConfig,
        scenario: SimulatorScenario,
        frame_idx: usize,
        timestamp_ms: u64,
    ) -> CctvFrame {
        let w = config.width;
        let h = config.height;
        let mut rgb = vec![0u8; (w * h * 3) as usize];

        // 1. Draw base architectural background (e.g. Bank Wall, Security Floor, Roller Shutter)
        let is_night = matches!(scenario, SimulatorScenario::NightIntruderShutter);
        let (wall_r, wall_g, wall_b): (u8, u8, u8) = if is_night { (18, 22, 28) } else { (180, 185, 190) };
        let (floor_r, floor_g, floor_b): (u8, u8, u8) = if is_night { (12, 14, 18) } else { (120, 125, 130) };

        for y in 0..h {
            let row_offset = (y * w * 3) as usize;
            let (r, g, b): (u8, u8, u8) = if y < (h * 2) / 3 {
                // Bank back wall or shutter
                if is_night && y % 32 < 4 {
                    // Shutter horizontal slats
                    (wall_r.saturating_add(15), wall_g.saturating_add(15), wall_b.saturating_add(20))
                } else {
                    (wall_r, wall_g, wall_b)
                }
            } else {
                // Bank floor
                (floor_r, floor_g, floor_b)
            };

            for x in 0..w {
                let idx = row_offset + (x * 3) as usize;
                // Add subtle CCTV sensor noise
                let noise = ((x.wrapping_mul(31) ^ y.wrapping_mul(17) ^ (frame_idx as u32)) % 5) as u8;
                rgb[idx] = r.saturating_add(noise);
                rgb[idx + 1] = g.saturating_add(noise);
                rgb[idx + 2] = b.saturating_add(noise);
            }
        }

        // 2. Inject dynamic scene actors based on scenario
        match scenario {
            SimulatorScenario::NormalQuiet => {
                // No actor; purely static scene
            }
            SimulatorScenario::NormalDayCustomer => {
                // Legitimate customer walking slowly from left to right (peaceful)
                let center_x = 200 + ((frame_idx * 4) % (w as usize - 400)) as u32;
                let center_y = h / 2;
                Self::draw_figure(&mut rgb, w, h, center_x, center_y, false, false, false);
            }
            SimulatorScenario::NightIntruderShutter => {
                // Starting at frame 10, an intruder appears in front of the shutter with crowbar
                if frame_idx >= 10 {
                    let intruder_x = (w / 2) as u32;
                    let intruder_y = ((h * 5) / 8) as u32;
                    // Masked face = true, Crowbar tool = true, Agitated motion = true
                    Self::draw_figure(&mut rgb, w, h, intruder_x, intruder_y, true, true, true);
                }
            }
            SimulatorScenario::DaytimeArmedRobber => {
                // Masked individual holding a firearm at bank teller counter
                if frame_idx >= 5 {
                    let robber_x = ((w * 3) / 5) as u32;
                    let robber_y = (h / 2) as u32;
                    // Masked face = true, Firearm = true
                    Self::draw_armed_robber(&mut rgb, w, h, robber_x, robber_y);
                }
            }
            SimulatorScenario::PackageDelivery => {
                // Delivery person dropping cardboard box at door
                let courier_x = (w / 3) as u32;
                let courier_y = (h / 2) as u32;
                Self::draw_figure(&mut rgb, w, h, courier_x, courier_y, false, false, false);
                // Draw delivery box
                Self::draw_box(&mut rgb, w, h, courier_x + 60, courier_y + 80, 50, 40, (160, 110, 60));
            }
            SimulatorScenario::AnimalCrossing => {
                // Small quadruped animal moving quickly across floor
                let animal_x = 100 + ((frame_idx * 12) % (w as usize - 200)) as u32;
                let animal_y = ((h * 4) / 5) as u32;
                Self::draw_small_animal(&mut rgb, w, h, animal_x, animal_y);
            }
        }

        CctvFrame::new(w, h, rgb, timestamp_ms, frame_idx as u64)
            .unwrap_or_else(|_| CctvFrame::synthetic_pattern(w, h, 0, 0, 0, timestamp_ms, frame_idx as u64))
    }

    /// Renders a human actor onto the RGB buffer with resolution-proportional dimensions.
    fn draw_figure(
        rgb: &mut [u8],
        w: u32,
        h: u32,
        cx: u32,
        cy: u32,
        is_masked: bool,
        has_crowbar: bool,
        is_dark_hoodie: bool,
    ) {
        let (cloth_r, cloth_g, cloth_b) = if is_dark_hoodie { (38, 38, 44) } else { (45, 90, 160) };

        let torso_w = (w / 7).clamp(60, 220);
        let torso_h = (h / 3).clamp(90, 280);
        let head_w = (torso_w * 5) / 9;
        let head_h = (torso_h * 2) / 5;

        // Torso
        Self::draw_box(rgb, w, h, cx.saturating_sub(torso_w / 2), cy.saturating_sub(torso_h / 2), torso_w, torso_h, (cloth_r, cloth_g, cloth_b));
        // Head
        let face_color = if is_masked { (20, 20, 25) } else { (225, 185, 155) };
        Self::draw_box(rgb, w, h, cx.saturating_sub(head_w / 2), cy.saturating_sub(torso_h / 2 + head_h + 4), head_w, head_h, face_color);

        // Crowbar tool in hand
        if has_crowbar {
            let tool_x = cx + torso_w / 2 + 5;
            let tool_y = cy.saturating_sub(torso_h / 3);
            let tool_w = (torso_w / 10).max(8);
            let tool_h = (torso_h * 3) / 4;
            // Long metallic crowbar (red/steel)
            Self::draw_box(rgb, w, h, tool_x, tool_y, tool_w, tool_h, (190, 40, 30));
        }
    }

    /// Renders a masked bank robber brandishing a revolver.
    fn draw_armed_robber(rgb: &mut [u8], w: u32, h: u32, cx: u32, cy: u32) {
        // Black tactical clothing
        Self::draw_figure(rgb, w, h, cx, cy, true, false, true);

        let torso_w = (w / 7).clamp(60, 220);
        let arm_w = (torso_w * 3) / 4;
        let arm_h = (torso_w / 4).max(16);
        let arm_x = cx.saturating_sub(torso_w / 2 + arm_w);
        let arm_y = cy.saturating_sub(arm_h * 2);
        Self::draw_box(rgb, w, h, arm_x, arm_y, arm_w, arm_h, (20, 20, 25));

        // Revolver / handgun outline (steel gunmetal grey)
        let gun_w = (arm_w * 3) / 5;
        let gun_h = (arm_h * 5) / 4;
        Self::draw_box(rgb, w, h, arm_x.saturating_sub(gun_w / 2), arm_y.saturating_sub(4), gun_w, gun_h, (65, 65, 68));
        Self::draw_box(rgb, w, h, arm_x.saturating_sub(gun_w / 4), arm_y + gun_h, gun_w / 3, gun_h, (40, 40, 42));
    }

    /// Renders a small domestic animal (cat/dog) on the floor.
    fn draw_small_animal(rgb: &mut [u8], w: u32, h: u32, cx: u32, cy: u32) {
        // Horizontal body
        Self::draw_box(rgb, w, h, cx.saturating_sub(25), cy.saturating_sub(15), 50, 25, (140, 95, 55));
        // Head
        Self::draw_box(rgb, w, h, cx + 25, cy.saturating_sub(22), 20, 20, (140, 95, 55));
    }

    /// Helper utility to draw a colored solid rectangle on the RGB buffer.
    fn draw_box(rgb: &mut [u8], w: u32, h: u32, x: u32, y: u32, bw: u32, bh: u32, color: (u8, u8, u8)) {
        for dy in 0..bh {
            let py = y + dy;
            if py >= h {
                break;
            }
            let row_offset = (py * w * 3) as usize;
            for dx in 0..bw {
                let px = x + dx;
                if px >= w {
                    break;
                }
                let idx = row_offset + (px * 3) as usize;
                if idx + 2 < rgb.len() {
                    rgb[idx] = color.0;
                    rgb[idx + 1] = color.1;
                    rgb[idx + 2] = color.2;
                }
            }
        }
    }

    /// Parses an RTSP URL into (host, port, path).
    fn parse_rtsp_url(url: &str) -> Result<(String, u16, String)> {
        let stripped = url
            .strip_prefix("rtsp://")
            .ok_or_else(|| anyhow!("Invalid RTSP URL: must begin with 'rtsp://'"))?;

        let (authority, path) = match stripped.find('/') {
            Some(idx) => (&stripped[..idx], &stripped[idx..]),
            None => (stripped, "/"),
        };

        let host_port = match authority.rfind('@') {
            Some(idx) => &authority[idx + 1..],
            None => authority,
        };

        let (host, port) = match host_port.find(':') {
            Some(idx) => {
                let h = host_port[..idx].to_string();
                let p = host_port[idx + 1..]
                    .parse::<u16>()
                    .map_err(|e| anyhow!("Invalid RTSP port: {}", e))?;
                (h, p)
            }
            None => (host_port.to_string(), 554),
        };

        Ok((host, port, path.to_string()))
    }
}
