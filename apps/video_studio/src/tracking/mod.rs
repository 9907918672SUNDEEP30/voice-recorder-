pub mod auto_framing;
pub mod face_detector;

pub use auto_framing::{AutoFramer, AutoFramingConfig, CropWindow};
pub use face_detector::{FaceBox, FaceDetector, FaceLandmarks};
