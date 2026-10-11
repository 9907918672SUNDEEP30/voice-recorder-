use anyhow::Result;
use crossbeam_channel::{bounded, Receiver, Sender};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::capture::{AudioCaptureStream, CaptureConfig};
use crate::dsp::{BiquadCascade, ToneCloner, ToneEqMatch, ToneProfile};
use crate::storage::{AsyncWavRecorder, BitDepth, WavWriterConfig};

use ai_core::models::adapters::VADAdapter;
use ai_core::models::dsp::AudioResampler;
use ai_core::models::onnx_adapters::SileroVadLocal;

/// Processing mode for the audio pipeline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PipelineMode {
    /// Continuous recording with real-time DSP tone styling.
    Continuous,
    /// Smart recording gated by Voice Activity Detection (silence is skipped).
    VadGated,
}

/// Pipeline configuration options.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub output_wav_path: PathBuf,
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub bit_depth: BitDepth,
    pub mode: PipelineMode,
    pub vad_model_path: Option<PathBuf>,
    pub vad_threshold: f32,
    pub initial_bass_db: f32,
    pub initial_presence_db: f32,
    pub initial_air_db: f32,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            output_wav_path: PathBuf::from("output_styled.wav"),
            sample_rate: 48000,
            buffer_size: 256,
            bit_depth: BitDepth::Pcm16,
            mode: PipelineMode::Continuous,
            vad_model_path: None,
            vad_threshold: 0.5,
            initial_bass_db: 3.0,
            initial_presence_db: 2.5,
            initial_air_db: 2.0,
        }
    }
}

/// Real-time live status snapshot produced by the DSP pipeline.
#[derive(Debug, Clone, Copy, Default)]
pub struct LivePipelineStats {
    pub rms_db: f32,
    pub peak_level: f32,
    pub is_speech_active: bool,
    pub speech_prob: f32,
    pub total_recorded_seconds: f32,
}

/// Studio-grade Audio Pipeline orchestrating Capture -> VAD -> IIR DSP Styling -> WAV Serialization.
pub struct StudioAudioPipeline {
    config: PipelineConfig,
    cascade: Arc<Mutex<BiquadCascade>>,
    is_running: Arc<AtomicBool>,
    stats: Arc<Mutex<LivePipelineStats>>,
    worker_handle: Option<JoinHandle<Result<u64>>>,
    capture_stream: Option<AudioCaptureStream>,
    vad_resampler: Option<AudioResampler>,
    vad_adapter: Option<SileroVadLocal>,
}

impl StudioAudioPipeline {
    /// Constructs a new pipeline ready for execution.
    pub fn new(config: PipelineConfig) -> Result<Self> {
        let mut cascade = BiquadCascade::new(config.sample_rate as f32);
        cascade.set_bass_gain_db(config.initial_bass_db);
        cascade.set_presence_gain_db(config.initial_presence_db);
        cascade.set_air_gain_db(config.initial_air_db);

        // Inherit optional Silero VAD from AI Core Framework
        let (vad_adapter, vad_resampler) = if let Some(ref model_path) = config.vad_model_path {
            if model_path.exists() {
                let adapter = SileroVadLocal::new(model_path.to_str().unwrap_or_default())?;
                let chunk_size = config.buffer_size as usize;
                // Resample from hardware sample rate (e.g. 48kHz) down to 16kHz for Silero VAD
                let resampler = AudioResampler::new(config.sample_rate as usize, 16000, chunk_size)?;
                (Some(adapter), Some(resampler))
            } else {
                eprintln!("⚠️ VAD model not found at '{:?}', running in un-gated mode.", model_path);
                (None, None)
            }
        } else {
            (None, None)
        };

        Ok(Self {
            config,
            cascade: Arc::new(Mutex::new(cascade)),
            is_running: Arc::new(AtomicBool::new(false)),
            stats: Arc::new(Mutex::new(LivePipelineStats::default())),
            worker_handle: None,
            capture_stream: None,
            vad_resampler,
            vad_adapter,
        })
    }

    /// Calibrates the DSP EQ by cloning the acoustic tone signature of a reference WAV file.
    pub fn calibrate_from_reference<P: AsRef<Path>>(&self, reference_wav: P) -> Result<ToneEqMatch> {
        let mut cloner = ToneCloner::new();
        let target_profile = cloner.analyze_wav_file(reference_wav)?;

        // Default neutral vocal input profile benchmark
        let mut neutral_profile = ToneProfile::default();
        neutral_profile.band_energies = [0.05, 0.15, 0.25, 0.30, 0.15, 0.08, 0.02];

        let eq_match = cloner.calculate_match_eq(&neutral_profile, &target_profile);

        // Apply calibrated tone directly to active biquad cascade
        let mut cascade = self.cascade.lock().unwrap();
        eq_match.apply_to(&mut cascade);

        Ok(eq_match)
    }

    /// Dynamically sets bass boost/cut in decibels.
    pub fn set_bass_gain_db(&self, db: f32) {
        let mut cascade = self.cascade.lock().unwrap();
        cascade.set_bass_gain_db(db);
    }

    /// Dynamically sets vocal presence in decibels.
    pub fn set_presence_gain_db(&self, db: f32) {
        let mut cascade = self.cascade.lock().unwrap();
        cascade.set_presence_gain_db(db);
    }

    /// Dynamically sets vocal air / sheen in decibels.
    pub fn set_air_gain_db(&self, db: f32) {
        let mut cascade = self.cascade.lock().unwrap();
        cascade.set_air_gain_db(db);
    }

    /// Returns the latest live pipeline telemetry and audio levels.
    pub fn get_live_stats(&self) -> LivePipelineStats {
        *self.stats.lock().unwrap()
    }

    /// Starts live audio capture and processing.
    pub fn start(&mut self) -> Result<()> {
        if self.is_running.load(Ordering::SeqCst) {
            return Ok(());
        }

        // Bounded channel from capture thread to DSP worker (512 chunks = ~2.7s buffer at 256 samples/chunk)
        let (capture_tx, capture_rx): (Sender<Vec<f32>>, Receiver<Vec<f32>>) = bounded(512);

        // Start CPAL hardware capture stream
        let capture_config = CaptureConfig {
            buffer_size: self.config.buffer_size,
            target_sample_rate: Some(self.config.sample_rate),
            device_name: None,
        };
        let stream = AudioCaptureStream::start(capture_config, capture_tx)?;
        self.capture_stream = Some(stream);

        // Start Async WAV Writer
        let writer_config = WavWriterConfig {
            file_path: self.config.output_wav_path.clone(),
            sample_rate: self.config.sample_rate,
            channels: 1,
            bit_depth: self.config.bit_depth,
        };
        let wav_recorder = AsyncWavRecorder::start(writer_config)?;

        self.is_running.store(true, Ordering::SeqCst);
        let is_running_clone = Arc::clone(&self.is_running);
        let cascade_clone = Arc::clone(&self.cascade);
        let stats_clone = Arc::clone(&self.stats);
        let sample_rate = self.config.sample_rate;
        let mode = self.config.mode;
        let vad_threshold = self.config.vad_threshold;

        let mut vad_opt = self.vad_adapter.take();
        let mut resampler_opt = self.vad_resampler.take();

        // Spawn central DSP & storage processing thread
        let handle = thread::spawn(move || -> Result<u64> {
            let start_time = Instant::now();
            let mut total_samples_written = 0u64;

            while is_running_clone.load(Ordering::Relaxed) {
                match capture_rx.recv_timeout(Duration::from_millis(50)) {
                    Ok(mut chunk) => {
                        if chunk.is_empty() {
                            continue;
                        }

                        // 1. Calculate input metrics (RMS and Peak)
                        let mut sum_sq = 0.0f32;
                        let mut peak = 0.0f32;
                        for &s in &chunk {
                            let abs_s = s.abs();
                            if abs_s > peak {
                                peak = abs_s;
                            }
                            sum_sq += s * s;
                        }
                        let rms = (sum_sq / chunk.len() as f32).sqrt();
                        let rms_db = if rms > 1e-5 { 20.0 * rms.log10() } else { -96.0 };

                        // 2. Voice Activity Detection (if configured)
                        let mut is_speech = true;
                        let mut speech_prob = 1.0f32;

                        if let (Some(ref mut vad), Some(ref mut resampler)) =
                            (&mut vad_opt, &mut resampler_opt)
                        {
                            if let Ok(resampled_16k) = resampler.resample(&chunk) {
                                // Convert f32 samples to 16-bit PCM bytes for Silero VAD
                                let mut pcm_bytes = Vec::with_capacity(resampled_16k.len() * 2);
                                for &sample in &resampled_16k {
                                    let s16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
                                    pcm_bytes.extend_from_slice(&s16.to_le_bytes());
                                }

                                // Run VAD chunk classification synchronously on worker thread
                                let rt = tokio::runtime::Builder::new_current_thread()
                                    .enable_all()
                                    .build();
                                if let Ok(rt) = rt {
                                    if let Ok((active, prob)) = rt.block_on(vad.process_chunk(&pcm_bytes)) {
                                        is_speech = active && prob >= vad_threshold;
                                        speech_prob = prob;
                                    }
                                }
                            }
                        }

                        // Update live stats
                        {
                            let mut st = stats_clone.lock().unwrap();
                            st.rms_db = rms_db;
                            st.peak_level = peak;
                            st.is_speech_active = is_speech;
                            st.speech_prob = speech_prob;
                            st.total_recorded_seconds =
                                total_samples_written as f32 / sample_rate as f32;
                        }

                        // 3. Process through Biquad Cascade DSP Engine (Low-Shelf, Presence, Air, Soft-Saturation)
                        {
                            let mut dsp = cascade_clone.lock().unwrap();
                            dsp.process_buffer(&mut chunk);
                        }

                        // 4. Stream to WAV Writer based on Pipeline Mode
                        let should_write = match mode {
                            PipelineMode::Continuous => true,
                            PipelineMode::VadGated => is_speech,
                        };

                        if should_write {
                            let len = chunk.len() as u64;
                            wav_recorder.write_chunk(chunk)?;
                            total_samples_written += len;
                        }
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
                    Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
                }
            }

            // Finalize WAV file on shutdown
            let final_samples = wav_recorder.finalize()?;
            println!(
                "🛑 Recording finished. Total duration: {:.2}s ({:?} samples). Elapsed: {:.2}s",
                final_samples as f32 / sample_rate as f32,
                final_samples,
                start_time.elapsed().as_secs_f32()
            );

            Ok(final_samples)
        });

        self.worker_handle = Some(handle);
        Ok(())
    }

    /// Stops audio capture and finalizes the recording session.
    pub fn stop(&mut self) -> Result<u64> {
        self.is_running.store(false, Ordering::SeqCst);

        if let Some(stream) = self.capture_stream.take() {
            stream.stop();
        }

        if let Some(handle) = self.worker_handle.take() {
            match handle.join() {
                Ok(res) => res,
                Err(_) => Err(anyhow::anyhow!("Pipeline thread panicked during execution")),
            }
        } else {
            Ok(0)
        }
    }
}
