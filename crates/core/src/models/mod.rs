pub mod adapters;
pub mod onnx_adapters;
pub mod dsp;
pub mod phonemizer;
pub mod yolo_onnx;

pub use yolo_onnx::{COCO_CLASSES, YoloDetection, YoloV8Detector};
