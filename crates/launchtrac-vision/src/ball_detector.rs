// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2022-2025, Verdant Consultants, LLC. (original PiTrac code)
// Copyright (C) 2026, LaunchTrac contributors
//
// This file is part of LaunchTrac, a derivative work of PiTrac
// (https://github.com/jeshernandez/PiTrac). Both projects are licensed
// under the GNU General Public License v2.0.
//
use launchtrac_common::error::LaunchTracError;
use launchtrac_hal::ImageFrame;

use crate::BallDetection;

/// Detection method configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DetectionMethod {
    /// YOLO v8 via NCNN (primary, recommended)
    Yolo,
    /// Hough circle transform (fallback, no model needed)
    Hough,
}

/// Ball detector using YOLO v8 via NCNN backend.
///
/// Architecture:
///   1. Preprocess: resize to 736x544, normalize
///   2. NCNN forward pass (MobileNet-based YOLO v8 nano)
///   3. Parse output: bounding boxes + confidence
///   4. Non-max suppression
///   5. Convert boxes to circle detections (center + radius)
///
/// Fallback: Hough circle transform when model is unavailable.
pub struct BallDetector {
    method: DetectionMethod,
    confidence_threshold: f64,
    nms_threshold: f64,
    model_loaded: bool,
    // NCNN net handle would go here
    // ncnn_net: Option<ncnn::Net>,
}

/// YOLO model input dimensions
const MODEL_INPUT_W: u32 = 736;
const MODEL_INPUT_H: u32 = 544;

/// Hough circle detection parameters (ported from LaunchTrac v1)
const HOUGH_DP: f64 = 1.0;
const HOUGH_MIN_DIST: f64 = 50.0;
const HOUGH_PARAM1: f64 = 120.0;
const HOUGH_PARAM2_INITIAL: f64 = 50.0;
const HOUGH_MIN_RADIUS: i32 = 20;
const HOUGH_MAX_RADIUS: i32 = 150;

impl BallDetector {
    pub fn new() -> Result<Self, LaunchTracError> {
        // Try to load YOLO model, fall back to Hough
        let (method, model_loaded) = match Self::load_yolo_model() {
            Ok(()) => {
                tracing::info!("YOLO ball detector loaded");
                (DetectionMethod::Yolo, true)
            }
            Err(e) => {
                tracing::warn!("YOLO model not available ({e}), using Hough circle fallback");
                (DetectionMethod::Hough, false)
            }
        };

        Ok(Self {
            method,
            confidence_threshold: 0.5,
            nms_threshold: 0.45,
            model_loaded,
        })
    }

    /// Force a specific detection method
    pub fn with_method(mut self, method: DetectionMethod) -> Self {
        self.method = method;
        self
    }

    fn load_yolo_model() -> Result<(), LaunchTracError> {
        // TODO: Load NCNN model files
        // let mut net = ncnn::Net::new();
        // net.load_param("models/yolo-ball.param")?;
        // net.load_model("models/yolo-ball.bin")?;
        Err(LaunchTracError::Vision("NCNN model files not found".into()))
    }

    /// Detect golf balls in a single frame
    pub fn detect(&self, frame: &ImageFrame) -> Result<Vec<BallDetection>, LaunchTracError> {
        match self.method {
            DetectionMethod::Yolo => self.detect_yolo(frame),
            DetectionMethod::Hough => self.detect_hough(frame),
        }
    }

    /// YOLO-based detection via NCNN
    fn detect_yolo(&self, frame: &ImageFrame) -> Result<Vec<BallDetection>, LaunchTracError> {
        if !self.model_loaded {
            return Err(LaunchTracError::Vision("YOLO model not loaded".into()));
        }

        // TODO: Full NCNN inference pipeline
        // 1. Resize frame to MODEL_INPUT_W x MODEL_INPUT_H
        // 2. Create ncnn::Mat from pixel data
        // 3. Forward pass through network
        // 4. Parse YOLO output layer
        // 5. Apply NMS
        // 6. Return detections

        Ok(vec![])
    }

    /// Hough circle transform detection (fallback).
    ///
    /// Ported from LaunchTrac v1 ball_image_proc.cpp GetBall().
    /// Simplified: single-pass with adaptive param2.
    fn detect_hough(&self, frame: &ImageFrame) -> Result<Vec<BallDetection>, LaunchTracError> {
        let w = frame.width as usize;
        let h = frame.height as usize;

        if frame.data.len() != w * h {
            return Err(LaunchTracError::Vision(format!(
                "Frame size mismatch: expected {}x{}={}, got {}",
                w,
                h,
                w * h,
                frame.data.len()
            )));
        }

        // Step 1: Gaussian blur (5x5 kernel approximation)
        let blurred = gaussian_blur_5x5(&frame.data, w, h);

        // Step 2: Find bright circular regions using simple peak detection
        // This is a simplified version - real impl uses OpenCV HoughCircles
        let detections = find_bright_circles(&blurred, w, h, HOUGH_MIN_RADIUS, HOUGH_MAX_RADIUS);

        Ok(detections)
    }
}

/// Simple 5x5 Gaussian blur (sigma ~1.0)
fn gaussian_blur_5x5(data: &[u8], width: usize, height: usize) -> Vec<u8> {
    // Kernel weights (normalized to sum=256 for integer math)
    const KERNEL: [[u16; 5]; 5] = [
        [1, 4, 7, 4, 1],
        [4, 16, 26, 16, 4],
        [7, 26, 41, 26, 7],
        [4, 16, 26, 16, 4],
        [1, 4, 7, 4, 1],
    ];
    const KERNEL_SUM: u16 = 273;

    let mut output = vec![0u8; data.len()];

    for y in 2..height.saturating_sub(2) {
        for x in 2..width.saturating_sub(2) {
            let mut sum: u32 = 0;
            for ky in 0..5 {
                for kx in 0..5 {
                    let px = data[(y + ky - 2) * width + (x + kx - 2)] as u32;
                    sum += px * KERNEL[ky][kx] as u32;
                }
            }
            output[y * width + x] = (sum / KERNEL_SUM as u32).min(255) as u8;
        }
    }

    output
}

/// Find bright circular regions in a grayscale image.
///
/// Simplified circle detection using radial intensity profile analysis.
/// For production, this would use OpenCV's HoughCircles or the YOLO model.
fn find_bright_circles(
    data: &[u8],
    width: usize,
    height: usize,
    min_radius: i32,
    max_radius: i32,
) -> Vec<BallDetection> {
    let mut detections = Vec::new();

    // Divide image into a grid and look for bright peaks
    let grid_size = max_radius as usize * 2;
    let threshold = compute_adaptive_threshold(data);

    for gy in (0..height).step_by(grid_size.max(1)) {
        for gx in (0..width).step_by(grid_size.max(1)) {
            // Find brightest pixel in this grid cell
            let mut max_val: u8 = 0;
            let mut max_x: usize = gx;
            let mut max_y: usize = gy;

            let end_y = (gy + grid_size).min(height);
            let end_x = (gx + grid_size).min(width);

            for y in gy..end_y {
                for x in gx..end_x {
                    let val = data[y * width + x];
                    if val > max_val {
                        max_val = val;
                        max_x = x;
                        max_y = y;
                    }
                }
            }

            if max_val < threshold {
                continue;
            }

            // Check if this peak has circular symmetry
            if let Some(detection) =
                verify_circle(data, width, height, max_x, max_y, min_radius, max_radius)
            {
                detections.push(detection);
            }
        }
    }

    // Non-max suppression: remove overlapping detections
    nms(&mut detections, 0.5);

    detections
}

/// Compute adaptive brightness threshold (Otsu-like)
fn compute_adaptive_threshold(data: &[u8]) -> u8 {
    let mut histogram = [0u32; 256];
    for &pixel in data {
        histogram[pixel as usize] += 1;
    }

    let total = data.len() as f64;
    let mut sum: f64 = 0.0;
    for (i, &count) in histogram.iter().enumerate() {
        sum += i as f64 * count as f64;
    }

    let mut sum_b: f64 = 0.0;
    let mut w_b: f64 = 0.0;
    let mut max_variance: f64 = 0.0;
    let mut threshold: u8 = 128;

    for (i, &count) in histogram.iter().enumerate() {
        w_b += count as f64;
        if w_b == 0.0 {
            continue;
        }
        let w_f = total - w_b;
        if w_f == 0.0 {
            break;
        }
        sum_b += i as f64 * count as f64;
        let mean_b = sum_b / w_b;
        let mean_f = (sum - sum_b) / w_f;
        let variance = w_b * w_f * (mean_b - mean_f).powi(2);
        if variance > max_variance {
            max_variance = variance;
            threshold = i as u8;
        }
    }

    // Use a higher threshold to only catch bright balls
    threshold.saturating_add(30)
}

/// Verify a peak is a circle by checking radial intensity profile
fn verify_circle(
    data: &[u8],
    width: usize,
    height: usize,
    cx: usize,
    cy: usize,
    min_radius: i32,
    max_radius: i32,
) -> Option<BallDetection> {
    let center_val = data[cy * width + cx] as f64;

    // Test radii from min to max, find best circle fit
    let mut best_score: f64 = 0.0;
    let mut best_radius: i32 = 0;

    for r in min_radius..=max_radius {
        let mut inside_sum: f64 = 0.0;
        let mut inside_count: u32 = 0;
        let mut edge_sum: f64 = 0.0;
        let mut edge_count: u32 = 0;

        // Sample points on and inside the circle
        let num_angles = 16;
        for a in 0..num_angles {
            let angle = (a as f64) * std::f64::consts::TAU / num_angles as f64;
            let cos_a = angle.cos();
            let sin_a = angle.sin();

            // Point on circle edge
            let ex = cx as f64 + r as f64 * cos_a;
            let ey = cy as f64 + r as f64 * sin_a;

            if ex >= 0.0 && ex < width as f64 && ey >= 0.0 && ey < height as f64 {
                edge_sum += data[ey as usize * width + ex as usize] as f64;
                edge_count += 1;
            }

            // Point at half radius (inside)
            let ix = cx as f64 + (r as f64 * 0.5) * cos_a;
            let iy = cy as f64 + (r as f64 * 0.5) * sin_a;

            if ix >= 0.0 && ix < width as f64 && iy >= 0.0 && iy < height as f64 {
                inside_sum += data[iy as usize * width + ix as usize] as f64;
                inside_count += 1;
            }
        }

        if inside_count == 0 || edge_count == 0 {
            continue;
        }

        let inside_avg = inside_sum / inside_count as f64;
        let edge_avg = edge_sum / edge_count as f64;

        // Good circle: bright inside, darker at edge (ball with IR reflection)
        let contrast = inside_avg - edge_avg;
        if contrast > best_score && contrast > 20.0 {
            best_score = contrast;
            best_radius = r;
        }
    }

    if best_radius > 0 {
        let confidence = (best_score / 128.0).min(1.0);
        Some(BallDetection {
            cx: cx as f64,
            cy: cy as f64,
            radius: best_radius as f64,
            confidence,
        })
    } else {
        None
    }
}

/// Non-max suppression: remove overlapping detections
fn nms(detections: &mut Vec<BallDetection>, overlap_threshold: f64) {
    detections.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

    let mut keep = vec![true; detections.len()];
    for i in 0..detections.len() {
        if !keep[i] {
            continue;
        }
        for j in (i + 1)..detections.len() {
            if !keep[j] {
                continue;
            }
            let dx = detections[i].cx - detections[j].cx;
            let dy = detections[i].cy - detections[j].cy;
            let dist = (dx * dx + dy * dy).sqrt();
            let max_r = detections[i].radius.max(detections[j].radius);

            if dist < max_r * (1.0 + overlap_threshold) {
                keep[j] = false;
            }
        }
    }

    let mut i = 0;
    detections.retain(|_| {
        let k = keep[i];
        i += 1;
        k
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_frame_with_ball(width: u32, height: u32, cx: u32, cy: u32, radius: u32) -> ImageFrame {
        let mut data = vec![30u8; (width * height) as usize];

        // Draw a bright circle
        for y in 0..height {
            for x in 0..width {
                let dx = x as f64 - cx as f64;
                let dy = y as f64 - cy as f64;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < radius as f64 {
                    // Bright inside, dimmer toward edge
                    let brightness = 200.0 - (dist / radius as f64) * 80.0;
                    data[(y * width + x) as usize] = brightness as u8;
                }
            }
        }

        ImageFrame {
            data,
            width,
            height,
            timestamp: chrono::Utc::now(),
            sequence: 0,
        }
    }

    #[test]
    fn gaussian_blur_preserves_dimensions() {
        let data = vec![128u8; 100 * 100];
        let blurred = gaussian_blur_5x5(&data, 100, 100);
        assert_eq!(blurred.len(), data.len());
    }

    #[test]
    fn adaptive_threshold_works() {
        let mut data = vec![20u8; 1000];
        // Add some bright pixels
        for i in 0..100 {
            data[i] = 200;
        }
        let threshold = compute_adaptive_threshold(&data);
        assert!(threshold > 20);
        assert!(threshold < 230);
    }

    #[test]
    fn nms_removes_overlapping() {
        let mut dets = vec![
            BallDetection {
                cx: 100.0,
                cy: 100.0,
                radius: 50.0,
                confidence: 0.9,
            },
            BallDetection {
                cx: 110.0,
                cy: 105.0,
                radius: 48.0,
                confidence: 0.7,
            },
            BallDetection {
                cx: 500.0,
                cy: 500.0,
                radius: 50.0,
                confidence: 0.8,
            },
        ];
        nms(&mut dets, 0.5);
        assert_eq!(dets.len(), 2); // First two overlap, third kept
    }

    #[test]
    fn detects_synthetic_ball() {
        let frame = make_frame_with_ball(400, 400, 200, 200, 50);
        let detector = BallDetector {
            method: DetectionMethod::Hough,
            confidence_threshold: 0.3,
            nms_threshold: 0.45,
            model_loaded: false,
        };

        let detections = detector.detect(&frame).unwrap();
        // Should find at least one detection near the ball center
        assert!(!detections.is_empty(), "Should detect the synthetic ball");

        let best = &detections[0];
        let dist = ((best.cx - 200.0).powi(2) + (best.cy - 200.0).powi(2)).sqrt();
        assert!(
            dist < 60.0,
            "Detection should be near ball center, got dist={dist}"
        );
    }
}
