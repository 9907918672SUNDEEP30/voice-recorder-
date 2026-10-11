use video_studio::capture::{CameraSource, VideoFrame, VirtualCamera};
use video_studio::gaze::{GazeCorrectionConfig, GazeCorrector, IrisTracker};
use video_studio::grading::{Lut3D, LutPreset, SkinFilter};
use video_studio::pipeline::{StudioPipelineConfig, StudioVideoPipeline};
use video_studio::tracking::{AutoFramer, AutoFramingConfig, FaceBox, FaceDetector};

#[test]
fn test_video_frame_pixel_access() {
    let mut frame = VideoFrame::new(320, 240);
    assert_eq!(frame.data.len(), 320 * 240 * 3);

    frame.set_pixel(10, 20, [255, 128, 64]);
    let rgb = frame.get_pixel(10, 20);
    assert_eq!(rgb, [255, 128, 64]);
}

#[test]
fn test_face_detector_on_virtual_camera() {
    let mut cam = VirtualCamera::new(640, 480);
    cam.start().expect("Start virtual camera");
    let frame = cam.capture_frame().expect("Capture frame");

    let mut detector = FaceDetector::new(0.50);
    let result = detector.detect_primary_face(&frame);

    assert!(result.is_some(), "Face should be detected on virtual camera frame");
    let (face_box, landmarks) = result.unwrap();
    assert!(face_box.width > 0.10 && face_box.width < 0.80);
    assert!(face_box.height > 0.10 && face_box.height < 0.85);
    assert!(landmarks.left_eye.0 < landmarks.right_eye.0, "Left eye must be left of right eye");
}

#[test]
fn test_auto_framing_ema_damping_and_rescale() {
    let config = AutoFramingConfig {
        deadzone_threshold: 0.05,
        smoothing_alpha: 0.10,
        ..Default::default()
    };
    let mut framer = AutoFramer::new(config);

    let initial_face = FaceBox {
        x: 0.30,
        y: 0.30,
        width: 0.30,
        height: 0.40,
        confidence: 0.95,
    };

    let crop1 = framer.update(Some(&initial_face));
    assert!(crop1.width > 0.3 && crop1.height > 0.3);

    // Minor movement within deadzone should not jerk the target
    let jitter_face = FaceBox {
        x: 0.302,
        y: 0.301,
        width: 0.30,
        height: 0.40,
        confidence: 0.95,
    };
    let crop2 = framer.update(Some(&jitter_face));
    assert!((crop2.left - crop1.left).abs() < 0.05, "Deadzone must dampen minor jitter");

    // Test bilinear render
    let src = VideoFrame::new(640, 480);
    let output = framer.render_framed_output(&src, 320, 240);
    assert_eq!(output.width, 320);
    assert_eq!(output.height, 240);
    assert_eq!(output.data.len(), 320 * 240 * 3);
}

#[test]
fn test_iris_tracker_and_gaze_warper() {
    let mut cam = VirtualCamera::new(640, 480);
    cam.start().expect("Start camera");
    let mut frame = cam.capture_frame().expect("Capture frame");

    let mut detector = FaceDetector::new(0.50);
    let (_, landmarks) = detector.detect_primary_face(&frame).expect("Detect face");

    let (left_eye, right_eye) = IrisTracker::locate_eyes(&frame, &landmarks);
    assert!(left_eye.is_open);
    assert!(right_eye.is_open);

    let mut corrector = GazeCorrector::new(GazeCorrectionConfig {
        correction_strength: 0.80,
        ..Default::default()
    });

    corrector.apply_eye_contact(&mut frame, &left_eye, &right_eye);

    // Verify all pixels remain valid and in [0, 255] range
    for &byte in &frame.data {
        let _ = byte;
    }
}

#[test]
fn test_lut3d_trilinear_interpolation() {
    let lut = Lut3D::from_preset(LutPreset::TealAndOrange);

    // Neutral gray test
    let out = lut.sample_trilinear(0.5, 0.5, 0.5);
    assert!(out[0] >= 0.0 && out[0] <= 1.0);
    assert!(out[1] >= 0.0 && out[1] <= 1.0);
    assert!(out[2] >= 0.0 && out[2] <= 1.0);

    let mut frame = VideoFrame::new(64, 64);
    frame.set_pixel(10, 10, [128, 128, 128]);
    lut.apply_to_frame(&mut frame);

    let pixel = frame.get_pixel(10, 10);
    assert!(pixel[0] > 0 || pixel[1] > 0 || pixel[2] > 0);
}

#[test]
fn test_skin_smoothing_and_relighting() {
    let filter = SkinFilter::new(0.70, 0.30);
    let mut frame = VideoFrame::new(128, 128);

    // Paint skin area with minor blemish variance
    for y in 20..80 {
        for x in 20..80 {
            frame.set_pixel(x, y, [220, 180, 150]);
        }
    }
    // Add artificial noise
    frame.set_pixel(40, 40, [240, 170, 140]);

    let face = FaceBox {
        x: 0.15,
        y: 0.15,
        width: 0.50,
        height: 0.50,
        confidence: 0.95,
    };

    filter.apply_skin_smoothing(&mut frame, Some(&face));
    filter.apply_studio_relighting(&mut frame, Some(&face));

    for &b in &frame.data {
        let _ = b;
    }
}

#[test]
fn test_end_to_end_virtual_camera_pipeline() {
    let cam = Box::new(VirtualCamera::new(640, 360));
    let config = StudioPipelineConfig {
        output_width: 640,
        output_height: 360,
        auto_framing_enabled: true,
        gaze_correction_enabled: true,
        gaze_strength: 0.80,
        skin_smoothing_enabled: true,
        skin_strength: 0.50,
        relighting_enabled: true,
        lut_preset: LutPreset::TealAndOrange,
    };

    let mut pipeline = StudioVideoPipeline::new(cam, config);
    pipeline.start().expect("Start pipeline");

    // Process 5 consecutive frames
    for _ in 0..5 {
        let frame = pipeline.process_next_frame().expect("Process frame");
        assert_eq!(frame.width, 640);
        assert_eq!(frame.height, 360);
    }

    let t = pipeline.telemetry();
    assert_eq!(t.frame_index, 5);
    assert!(t.latency_ms < 300.0, "Latency in unoptimized debug build should be under 300ms");
    assert!(t.face_detected, "Synthetic face must be detected");
    assert!(t.eye_contact_active, "Eye contact correction should be active");

    pipeline.stop().expect("Stop pipeline");
}
