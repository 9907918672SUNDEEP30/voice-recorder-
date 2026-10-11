use dsp_recorder::dsp::{
    BiquadCascade, BiquadCoefficients, BiquadFilter, ParameterSmoother, ToneCloner, ToneProfile,
};
use dsp_recorder::storage::{AsyncWavRecorder, BitDepth, WavWriterConfig};
use std::f32::consts::PI;

#[test]
fn test_biquad_coefficients_finite_and_stable() {
    let sample_rate = 48000.0;

    let ls = BiquadCoefficients::low_shelf(sample_rate, 100.0, 6.0, 0.7071);
    assert!(ls.b0.is_finite());
    assert!(ls.b1.is_finite());
    assert!(ls.b2.is_finite());
    assert!(ls.a1.is_finite());
    assert!(ls.a2.is_finite());

    let hs = BiquadCoefficients::high_shelf(sample_rate, 8000.0, 4.0, 0.7071);
    assert!(hs.b0.is_finite());
    assert!(hs.a1.is_finite());

    let peak = BiquadCoefficients::peaking(sample_rate, 3000.0, 3.0, 1.2);
    assert!(peak.b0.is_finite());
    assert!(peak.a2.is_finite());

    let hp = BiquadCoefficients::high_pass(sample_rate, 35.0, 0.7071);
    assert!(hp.b0.is_finite());
}

#[test]
fn test_biquad_filter_processing_sine_wave() {
    let sample_rate = 48000.0;
    let coeffs = BiquadCoefficients::low_shelf(sample_rate, 100.0, 6.0, 0.7071);
    let mut filter = BiquadFilter::new(coeffs);

    // Generate 100 Hz test tone
    let mut buffer = Vec::with_capacity(480);
    for n in 0..480 {
        let sample = (2.0 * PI * 100.0 * n as f32 / sample_rate).sin() * 0.5;
        buffer.push(sample);
    }

    filter.process_buffer(&mut buffer);

    for sample in buffer {
        assert!(sample.is_finite(), "Sample must be finite");
        assert!(sample.abs() <= 1.5, "Sample must not blow up");
    }
}

#[test]
fn test_biquad_cascade_warm_styling() {
    let mut cascade = BiquadCascade::new(48000.0);
    cascade.set_bass_gain_db(4.0);
    cascade.set_presence_gain_db(3.0);
    cascade.set_air_gain_db(2.0);

    let mut audio = vec![0.1f32; 1024];
    cascade.process_buffer(&mut audio);

    for sample in audio {
        assert!(sample.is_finite());
    }
}

#[test]
fn test_parameter_smoother_convergence() {
    let mut smoother = ParameterSmoother::new(0.0, 10.0, 48000.0);
    smoother.set_target(6.0);

    // After 4800 samples (100ms, 10x tau), should be settled at 6.0
    for _ in 0..4800 {
        smoother.next_sample();
    }

    assert!(smoother.is_settled(), "Smoother must settle at target");
    assert!((smoother.current() - 6.0).abs() < 0.01);
}

#[test]
fn test_tone_cloner_fft_spectral_bands() {
    let mut cloner = ToneCloner::new();
    let sample_rate = 48000.0;

    // Generate audio composed of 100Hz (Bass) and 3000Hz (Presence)
    let num_samples = 4096;
    let mut test_audio = Vec::with_capacity(num_samples);
    for n in 0..num_samples {
        let s_bass = (2.0 * PI * 100.0 * n as f32 / sample_rate).sin() * 0.4;
        let s_presence = (2.0 * PI * 3000.0 * n as f32 / sample_rate).sin() * 0.4;
        test_audio.push(s_bass + s_presence);
    }

    let profile = cloner.analyze_audio(&test_audio, sample_rate);
    assert_eq!(profile.frames_analyzed, 3);
    assert!(profile.band_energies[1] > 0.01, "Bass band must have energy");
    assert!(profile.band_energies[4] > 0.01, "High-Mid band must have energy");

    // Match against target profile
    let mut target = ToneProfile::default();
    target.band_energies = [0.02, 0.30, 0.20, 0.20, 0.15, 0.08, 0.05];

    let eq_match = cloner.calculate_match_eq(&profile, &target);
    assert!(eq_match.bass_shelf_db.is_finite());
    assert!(eq_match.presence_db.is_finite());
    assert!(eq_match.high_shelf_air_db.is_finite());
}

#[test]
fn test_wav_writer_lossless_roundtrip() {
    let temp_path = std::env::temp_dir().join("dsp_test_hound_roundtrip.wav");
    let sample_rate = 48000;
    let test_samples: Vec<f32> = (0..960)
        .map(|n| (2.0 * PI * 440.0 * n as f32 / sample_rate as f32).sin() * 0.7)
        .collect();

    // Write file using AsyncWavRecorder
    let config = WavWriterConfig {
        file_path: temp_path.clone(),
        sample_rate,
        channels: 1,
        bit_depth: BitDepth::Pcm16,
    };
    let recorder = AsyncWavRecorder::start(config).expect("Start recorder");
    recorder.write_chunk(test_samples.clone()).expect("Write chunk");
    let written = recorder.finalize().expect("Finalize writer");
    assert_eq!(written, 960);

    // Read back using hound
    let mut reader = hound::WavReader::open(&temp_path).expect("Open reader");
    assert_eq!(reader.spec().sample_rate, sample_rate);
    assert_eq!(reader.spec().channels, 1);
    assert_eq!(reader.spec().bits_per_sample, 16);

    let read_samples: Vec<i16> = reader.samples::<i16>().filter_map(|s| s.ok()).collect();
    assert_eq!(read_samples.len(), 960);

    let _ = std::fs::remove_file(temp_path);
}
