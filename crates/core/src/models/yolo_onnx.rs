//! YOLOv8 / YOLOv11 ONNX Object Detection Engine
//! Part of ai_core centralized framework.
//! Provides high-speed SIMD inference, letterbox preprocessing, and NMS postprocessing.

use anyhow::{anyhow, Result};
use image::{imageops::FilterType, Rgb, RgbImage};
use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::Tensor;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Single detected object from YOLO
#[derive(Debug, Clone)]
pub struct YoloDetection {
    pub class_id: usize,
    pub label: String,
    pub confidence: f32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// 80 COCO Dataset Class Names
pub const COCO_CLASSES: [&str; 80] = [
    "person", "bicycle", "car", "motorcycle", "airplane", "bus", "train", "truck", "boat",
    "traffic light", "fire hydrant", "stop sign", "parking meter", "bench", "bird", "cat",
    "dog", "horse", "sheep", "cow", "elephant", "bear", "zebra", "giraffe", "backpack",
    "umbrella", "handbag", "tie", "suitcase", "frisbee", "skis", "snowboard", "sports ball",
    "kite", "baseball bat", "baseball glove", "skateboard", "surfboard", "tennis racket",
    "bottle", "wine glass", "cup", "fork", "knife", "spoon", "bowl", "banana", "apple",
    "sandwich", "orange", "broccoli", "carrot", "hot dog", "pizza", "donut", "cake", "chair",
    "couch", "potted plant", "bed", "dining table", "toilet", "tv", "laptop", "mouse",
    "remote", "keyboard", "cell phone", "microwave", "oven", "toaster", "sink", "refrigerator",
    "book", "clock", "vase", "scissors", "teddy bear", "hair drier", "toothbrush",
];

/// YOLOv8 ONNX Detector
pub struct YoloV8Detector {
    session: Option<Arc<Mutex<Session>>>,
    pub model_path: String,
    pub conf_threshold: f32,
    pub iou_threshold: f32,
    pub input_size: (u32, u32),
}

impl YoloV8Detector {
    /// Create a new YOLO detector. If the model file exists, loads ONNX session; otherwise runs in standby.
    pub fn new(model_path: &str, conf_threshold: f32) -> Result<Self> {
        let path = Path::new(model_path);
        let session = if path.exists() {
            println!("⚡ [ai_core] Loading YOLO ONNX Model: {:?}", path);
            let s = Session::builder()
                .map_err(|e| anyhow!("{e}"))?
                .with_optimization_level(GraphOptimizationLevel::Level3)
                .map_err(|e| anyhow!("{e}"))?
                .with_intra_threads(4) // 4 threads for Quad-Core CPU SIMD
                .map_err(|e| anyhow!("{e}"))?
                .commit_from_file(path)
                .map_err(|e| anyhow!("{e}"))?;
            Some(Arc::new(Mutex::new(s)))
        } else {
            println!("ℹ️  [ai_core] Model file not yet found at {:?}. Ready for drop-in.", path);
            None
        };

        Ok(Self {
            session,
            model_path: model_path.to_string(),
            conf_threshold,
            iou_threshold: 0.45,
            input_size: (640, 640),
        })
    }

    /// Preprocesses image with standard YOLO letterboxing (aspect ratio preserving padding)
    pub fn letterbox(image: &RgbImage, target_w: u32, target_h: u32) -> (RgbImage, f32, f32, f32) {
        let (orig_w, orig_h) = image.dimensions();
        let scale = ((target_w as f32) / (orig_w as f32)).min((target_h as f32) / (orig_h as f32));
        let new_w = ((orig_w as f32) * scale).round() as u32;
        let new_h = ((orig_h as f32) * scale).round() as u32;

        let resized = image::imageops::resize(image, new_w, new_h, FilterType::Triangle);

        let pad_x = ((target_w - new_w) as f32) / 2.0;
        let pad_y = ((target_h - new_h) as f32) / 2.0;

        let mut canvas = RgbImage::from_pixel(target_w, target_h, Rgb([114, 114, 114]));
        image::imageops::overlay(&mut canvas, &resized, pad_x.round() as i64, pad_y.round() as i64);

        (canvas, scale, pad_x, pad_y)
    }

    /// Calculates Intersection-over-Union (IoU) between two bounding boxes
    pub fn calculate_iou(box1: &YoloDetection, box2: &YoloDetection) -> f32 {
        let x1 = box1.x.max(box2.x);
        let y1 = box1.y.max(box2.y);
        let x2 = (box1.x + box1.width).min(box2.x + box2.width);
        let y2 = (box1.y + box1.height).min(box2.y + box2.height);

        let intersection_area = (x2 - x1).max(0.0) * (y2 - y1).max(0.0);
        let box1_area = box1.width * box1.height;
        let box2_area = box2.width * box2.height;
        let union_area = box1_area + box2_area - intersection_area;

        if union_area <= 0.0 {
            0.0
        } else {
            intersection_area / union_area
        }
    }

    /// Applies Non-Maximum Suppression (NMS) to eliminate duplicate overlapping bounding boxes
    pub fn apply_nms(mut candidates: Vec<YoloDetection>, iou_threshold: f32) -> Vec<YoloDetection> {
        // Sort by confidence descending
        candidates.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

        let mut keep = Vec::new();

        while !candidates.is_empty() {
            let current = candidates.remove(0);
            candidates.retain(|candidate| {
                if candidate.class_id != current.class_id {
                    true // Different class: keep candidate
                } else {
                    let iou = Self::calculate_iou(&current, candidate);
                    iou <= iou_threshold // Discard if overlapping above threshold
                }
            });
            keep.push(current);
        }

        keep
    }

    /// Run inference on an RGB image
    pub fn detect(&self, image: &RgbImage) -> Result<Vec<YoloDetection>> {
        let session_arc = match &self.session {
            Some(s) => s,
            None => {
                // Strictly fail closed if model is not loaded — no fake person simulation!
                return Err(anyhow!(
                    "YOLO model not loaded. Model file missing at '{}'. Please provide yolov8m.onnx.",
                    self.model_path
                ));
            }
        };

        let (orig_w, orig_h) = image.dimensions();
        let (in_w, in_h) = self.input_size;

        // 1. Aspect-ratio preserving letterbox preprocessing
        let (letterboxed, scale, pad_x, pad_y) = Self::letterbox(image, in_w, in_h);

        // 2. Prepare CHW normalized float tensor: [1, 3, 640, 640]
        let mut tensor_data = Vec::with_capacity((3 * in_w * in_h) as usize);
        for c in 0..3 {
            for y in 0..in_h {
                for x in 0..in_w {
                    let p = letterboxed.get_pixel(x, y);
                    let val = (p[c] as f32) / 255.0;
                    tensor_data.push(val);
                }
            }
        }

        // 3. Run ONNX Session forward pass
        let mut session = session_arc.lock().map_err(|_| anyhow!("Mutex poison"))?;
        let input_tensor = Tensor::from_array((vec![1, 3, in_h as usize, in_w as usize], tensor_data))
            .map_err(|e| anyhow!("{e}"))?;

        let inputs = ort::inputs!["images" => input_tensor];
        let outputs = session.run(inputs)?;

        // 4. Parse YOLOv8 raw output tensor [1, 84, 8400]
        let (shape, raw_slice) = outputs[0].try_extract_tensor::<f32>()
            .map_err(|e| anyhow!("{e}"))?;

        let mut raw_candidates = Vec::new();
        if shape.len() == 3 && shape[1] >= 84 {
            let num_candidates = shape[2] as usize;

            for i in 0..num_candidates {
                let mut max_class_id = 0;
                let mut max_score = 0.0f32;

                for c in 0..80 {
                    let score_idx = (4 + c) * num_candidates + i;
                    if score_idx < raw_slice.len() {
                        let score = raw_slice[score_idx];
                        if score > max_score {
                            max_score = score;
                            max_class_id = c;
                        }
                    }
                }

                if max_score >= self.conf_threshold {
                    let cx = raw_slice[0 * num_candidates + i];
                    let cy = raw_slice[1 * num_candidates + i];
                    let w = raw_slice[2 * num_candidates + i];
                    let h = raw_slice[3 * num_candidates + i];

                    // Unpad and rescale back to original image coordinates
                    let box_x = cx - w / 2.0;
                    let box_y = cy - h / 2.0;

                    let orig_x = ((box_x - pad_x) / scale).max(0.0).min(orig_w as f32);
                    let orig_y = ((box_y - pad_y) / scale).max(0.0).min(orig_h as f32);
                    let orig_w_box = (w / scale).min(orig_w as f32 - orig_x);
                    let orig_h_box = (h / scale).min(orig_h as f32 - orig_y);

                    let label = COCO_CLASSES.get(max_class_id).unwrap_or(&"unknown").to_string();

                    raw_candidates.push(YoloDetection {
                        class_id: max_class_id,
                        label,
                        confidence: max_score,
                        x: orig_x,
                        y: orig_y,
                        width: orig_w_box,
                        height: orig_h_box,
                    });
                }
            }
        }

        // 5. Apply true IoU Non-Maximum Suppression (NMS)
        let nms_detections = Self::apply_nms(raw_candidates, self.iou_threshold);

        Ok(nms_detections)
    }
}
