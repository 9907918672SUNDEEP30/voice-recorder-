use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Device, SampleFormat, Stream, StreamConfig};
use crossbeam_channel::Sender;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

/// Low-latency audio stream statistics for real-time monitoring and diagnostic metrics.
#[derive(Debug, Default)]
pub struct StreamMetrics {
    pub total_samples_captured: AtomicU64,
    pub buffer_overflow_drops: AtomicU64,
}

/// Configuration parameters for initializing hardware audio capture.
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    /// Preferred hardware buffer frame size (e.g. 128 or 256 for zero-latency)
    pub buffer_size: u32,
    /// Target sample rate (e.g. 48000 or 44100). If None, device default is used.
    pub target_sample_rate: Option<u32>,
    /// Optional specific input device name
    pub device_name: Option<String>,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            buffer_size: 256,
            target_sample_rate: Some(48000),
            device_name: None,
        }
    }
}

/// Active hardware audio capture controller.
pub struct AudioCaptureStream {
    _stream: Stream,
    sample_rate: u32,
    channels: u16,
    buffer_size: u32,
    metrics: Arc<StreamMetrics>,
    is_running: Arc<AtomicBool>,
}

impl AudioCaptureStream {
    /// Lists all available system audio input devices.
    pub fn list_input_devices() -> Result<Vec<String>> {
        let host = cpal::default_host();
        let devices = host.input_devices()?;
        let mut names = Vec::new();
        for device in devices {
            if let Ok(desc) = device.description() {
                names.push(desc.name().to_string());
            }
        }
        Ok(names)
    }

    /// Selects an audio input device matching the specified name or falls back to system default.
    fn resolve_device(device_name: Option<&str>) -> Result<Device> {
        let host = cpal::default_host();
        if let Some(name) = device_name {
            for device in host.input_devices()? {
                if let Ok(desc) = device.description() {
                    if desc.name().contains(name) {
                        return Ok(device);
                    }
                }
            }
            eprintln!("⚠️ Requested audio device '{}' not found. Using default input device.", name);
        }

        host.default_input_device()
            .ok_or_else(|| anyhow!("❌ No default audio input device found on the system."))
    }

    /// Starts a low-latency input stream, pushing mono f32 audio chunks to the provided crossbeam channel.
    pub fn start(
        config: CaptureConfig,
        pcm_sender: Sender<Vec<f32>>,
    ) -> Result<Self> {
        let device = Self::resolve_device(config.device_name.as_deref())?;
        let supported_config = device.default_input_config()?;

        let sample_rate = config
            .target_sample_rate
            .unwrap_or_else(|| supported_config.sample_rate());

        let channels = supported_config.channels();
        let sample_format = supported_config.sample_format();

        let stream_config = StreamConfig {
            channels,
            sample_rate,
            buffer_size: BufferSize::Fixed(config.buffer_size),
        };

        let metrics = Arc::new(StreamMetrics::default());
        let metrics_clone = Arc::clone(&metrics);
        let is_running = Arc::new(AtomicBool::new(true));
        let is_running_clone = Arc::clone(&is_running);

        let err_callback = |err: cpal::Error| {
            eprintln!("🚨 Audio Input Hardware Error: {:?}", err);
        };

        let stream = match sample_format {
            SampleFormat::F32 => Self::build_stream_f32(
                &device,
                stream_config,
                channels as usize,
                pcm_sender,
                metrics_clone,
                is_running_clone,
                err_callback,
            )?,
            SampleFormat::I16 => Self::build_stream_i16(
                &device,
                stream_config,
                channels as usize,
                pcm_sender,
                metrics_clone,
                is_running_clone,
                err_callback,
            )?,
            SampleFormat::U16 => {
                return Err(anyhow!("Unsupported hardware sample format: U16"));
            }
            _ => return Err(anyhow!("Unsupported audio sample format")),
        };

        stream.play()?;

        Ok(Self {
            _stream: stream,
            sample_rate,
            channels,
            buffer_size: config.buffer_size,
            metrics,
            is_running,
        })
    }

    fn build_stream_f32<F>(
        device: &Device,
        config: StreamConfig,
        channels: usize,
        sender: Sender<Vec<f32>>,
        metrics: Arc<StreamMetrics>,
        is_running: Arc<AtomicBool>,
        err_fn: F,
    ) -> Result<Stream>
    where
        F: FnMut(cpal::Error) + Send + 'static,
    {
        let stream = device.build_input_stream(
            config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if !is_running.load(Ordering::Relaxed) {
                    return;
                }

                let num_frames = data.len() / channels;
                let mut mono_chunk = Vec::with_capacity(num_frames);

                if channels == 1 {
                    mono_chunk.extend_from_slice(data);
                } else {
                    for frame in data.chunks_exact(channels) {
                        let sum: f32 = frame.iter().sum();
                        mono_chunk.push(sum / channels as f32);
                    }
                }

                metrics.total_samples_captured.fetch_add(mono_chunk.len() as u64, Ordering::Relaxed);

                // Non-blocking try_send into bounded channel to protect real-time audio thread
                if let Err(_) = sender.try_send(mono_chunk) {
                    metrics.buffer_overflow_drops.fetch_add(1, Ordering::Relaxed);
                }
            },
            err_fn,
            None,
        )?;

        Ok(stream)
    }

    fn build_stream_i16<F>(
        device: &Device,
        config: StreamConfig,
        channels: usize,
        sender: Sender<Vec<f32>>,
        metrics: Arc<StreamMetrics>,
        is_running: Arc<AtomicBool>,
        err_fn: F,
    ) -> Result<Stream>
    where
        F: FnMut(cpal::Error) + Send + 'static,
    {
        let stream = device.build_input_stream(
            config,
            move |data: &[i16], _: &cpal::InputCallbackInfo| {
                if !is_running.load(Ordering::Relaxed) {
                    return;
                }

                let num_frames = data.len() / channels;
                let mut mono_chunk = Vec::with_capacity(num_frames);
                let inv_scale = 1.0 / 32768.0f32;

                if channels == 1 {
                    for &sample in data {
                        mono_chunk.push(sample as f32 * inv_scale);
                    }
                } else {
                    for frame in data.chunks_exact(channels) {
                        let sum: i32 = frame.iter().map(|&s| s as i32).sum();
                        let avg = sum as f32 / channels as f32;
                        mono_chunk.push(avg * inv_scale);
                    }
                }

                metrics.total_samples_captured.fetch_add(mono_chunk.len() as u64, Ordering::Relaxed);

                if let Err(_) = sender.try_send(mono_chunk) {
                    metrics.buffer_overflow_drops.fetch_add(1, Ordering::Relaxed);
                }
            },
            err_fn,
            None,
        )?;

        Ok(stream)
    }

    /// Stops the capture stream.
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::Relaxed);
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }

    pub fn buffer_size(&self) -> u32 {
        self.buffer_size
    }

    pub fn metrics(&self) -> Arc<StreamMetrics> {
        Arc::clone(&self.metrics)
    }
}
