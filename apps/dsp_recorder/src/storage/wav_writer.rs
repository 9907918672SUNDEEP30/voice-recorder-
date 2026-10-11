use anyhow::{anyhow, Result};
use crossbeam_channel::{bounded, Receiver, Sender};
use hound::{SampleFormat, WavSpec, WavWriter};
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

/// Supported PCM bit depths for studio recording.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitDepth {
    Pcm16,
    Pcm24,
    Float32,
}

/// WAV file recording output configuration.
#[derive(Debug, Clone)]
pub struct WavWriterConfig {
    pub file_path: PathBuf,
    pub sample_rate: u32,
    pub channels: u16,
    pub bit_depth: BitDepth,
}

impl Default for WavWriterConfig {
    fn default() -> Self {
        Self {
            file_path: PathBuf::from("recording_output.wav"),
            sample_rate: 48000,
            channels: 1,
            bit_depth: BitDepth::Pcm16,
        }
    }
}

/// Real-time WAV writing metrics.
#[derive(Debug, Default)]
pub struct WriterMetrics {
    pub total_samples_written: AtomicU64,
    pub total_chunks_processed: AtomicU64,
}

/// Asynchronous, non-blocking WAV file recorder running on a dedicated worker thread.
pub struct AsyncWavRecorder {
    chunk_sender: Sender<Vec<f32>>,
    stop_signal: Arc<AtomicBool>,
    worker_handle: Option<JoinHandle<Result<u64>>>,
    metrics: Arc<WriterMetrics>,
    file_path: PathBuf,
}

impl AsyncWavRecorder {
    /// Spawns a dedicated background file-writing thread and starts recording to the specified WAV path.
    pub fn start(config: WavWriterConfig) -> Result<Self> {
        let spec = match config.bit_depth {
            BitDepth::Pcm16 => WavSpec {
                channels: config.channels,
                sample_rate: config.sample_rate,
                bits_per_sample: 16,
                sample_format: SampleFormat::Int,
            },
            BitDepth::Pcm24 => WavSpec {
                channels: config.channels,
                sample_rate: config.sample_rate,
                bits_per_sample: 24,
                sample_format: SampleFormat::Int,
            },
            BitDepth::Float32 => WavSpec {
                channels: config.channels,
                sample_rate: config.sample_rate,
                bits_per_sample: 32,
                sample_format: SampleFormat::Float,
            },
        };

        // Create directory parent if required
        if let Some(parent) = config.file_path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let file = File::create(&config.file_path)?;
        let buf_writer = BufWriter::with_capacity(128 * 1024, file); // 128KB buffer for low disk churn
        let mut wav_writer = WavWriter::new(buf_writer, spec)?;

        // Bounded channel: allows smooth buffering without unbounded memory growth
        let (tx, rx): (Sender<Vec<f32>>, Receiver<Vec<f32>>) = bounded(512);
        let stop_signal = Arc::new(AtomicBool::new(false));
        let stop_clone = Arc::clone(&stop_signal);
        let metrics = Arc::new(WriterMetrics::default());
        let metrics_clone = Arc::clone(&metrics);

        let bit_depth = config.bit_depth;
        let worker_handle = thread::spawn(move || -> Result<u64> {
            let mut samples_count = 0u64;

            while !stop_clone.load(Ordering::Relaxed) || !rx.is_empty() {
                match rx.recv_timeout(std::time::Duration::from_millis(50)) {
                    Ok(chunk) => {
                        match bit_depth {
                            BitDepth::Pcm16 => {
                                for sample in &chunk {
                                    let clamped = sample.clamp(-1.0, 1.0);
                                    let s16 = (clamped * 32767.0) as i16;
                                    wav_writer.write_sample(s16)?;
                                }
                            }
                            BitDepth::Pcm24 => {
                                for sample in &chunk {
                                    let clamped = sample.clamp(-1.0, 1.0);
                                    // 24-bit signed integer range: -8388608 to 8388607
                                    let s24 = (clamped * 8388607.0) as i32;
                                    wav_writer.write_sample(s24)?;
                                }
                            }
                            BitDepth::Float32 => {
                                for sample in &chunk {
                                    wav_writer.write_sample(*sample)?;
                                }
                            }
                        }

                        let len = chunk.len() as u64;
                        samples_count += len;
                        metrics_clone.total_samples_written.fetch_add(len, Ordering::Relaxed);
                        metrics_clone.total_chunks_processed.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                        // Keep looping until stop_signal is set and channel is fully drained
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                        break;
                    }
                }
            }

            wav_writer.finalize().map_err(|e| anyhow!("Hound finalize error: {:?}", e))?;
            Ok(samples_count)
        });

        Ok(Self {
            chunk_sender: tx,
            stop_signal,
            worker_handle: Some(worker_handle),
            metrics,
            file_path: config.file_path,
        })
    }

    /// Queues an in-memory chunk of audio samples to be written to disk.
    pub fn write_chunk(&self, chunk: Vec<f32>) -> Result<()> {
        self.chunk_sender
            .send(chunk)
            .map_err(|_| anyhow!("Failed to send audio chunk to WAV writer (worker disconnected)"))
    }

    /// Gracefully stops recording, flushes all buffers, finalizes WAV headers, and joins the thread.
    pub fn finalize(mut self) -> Result<u64> {
        self.stop_signal.store(true, Ordering::SeqCst);
        // Dropping sender triggers channel close
        drop(self.chunk_sender);

        if let Some(handle) = self.worker_handle.take() {
            match handle.join() {
                Ok(res) => res,
                Err(_) => Err(anyhow!("WAV writer background thread panicked")),
            }
        } else {
            Ok(0)
        }
    }

    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    pub fn total_samples_written(&self) -> u64 {
        self.metrics.total_samples_written.load(Ordering::Relaxed)
    }

    /// Synchronously writes an entire in-memory slice of f32 samples to a WAV file.
    pub fn write_slice_to_file<P: AsRef<Path>>(
        file_path: P,
        samples: &[f32],
        sample_rate: u32,
        bit_depth: BitDepth,
    ) -> Result<()> {
        let spec = match bit_depth {
            BitDepth::Pcm16 => WavSpec {
                channels: 1,
                sample_rate,
                bits_per_sample: 16,
                sample_format: SampleFormat::Int,
            },
            BitDepth::Pcm24 => WavSpec {
                channels: 1,
                sample_rate,
                bits_per_sample: 24,
                sample_format: SampleFormat::Int,
            },
            BitDepth::Float32 => WavSpec {
                channels: 1,
                sample_rate,
                bits_per_sample: 32,
                sample_format: SampleFormat::Float,
            },
        };

        let mut writer = WavWriter::create(file_path, spec)?;
        match bit_depth {
            BitDepth::Pcm16 => {
                for &sample in samples {
                    let s16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
                    writer.write_sample(s16)?;
                }
            }
            BitDepth::Pcm24 => {
                for &sample in samples {
                    let s24 = (sample.clamp(-1.0, 1.0) * 8388607.0) as i32;
                    writer.write_sample(s24)?;
                }
            }
            BitDepth::Float32 => {
                for &sample in samples {
                    writer.write_sample(sample)?;
                }
            }
        }
        writer.finalize()?;
        Ok(())
    }
}
