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
use crate::calibration::CameraCalibration;

/// Computed ball trajectory from multi-frame analysis
#[derive(Debug, Clone)]
pub struct TrajectoryResult {
    /// Ball speed in meters per second
    pub speed_ms: f64,
    /// Vertical launch angle in degrees (positive = upward)
    pub vla_deg: f64,
    /// Horizontal launch angle in degrees (negative = left)
    pub hla_deg: f64,
    /// Trajectory confidence (0.0 - 1.0)
    pub confidence: f64,
}

/// Known golf ball diameter in meters
const BALL_DIAMETER_M: f64 = 0.04267;

/// Compute ball trajectory from a sequence of detections and frames.
///
/// Physics:
///   - Speed: computed from 3D displacement between first and last detection
///   - VLA: vertical component of velocity vector
///   - HLA: horizontal component of velocity vector
///
/// 3D reconstruction uses known ball diameter (42.67mm) to estimate depth.
pub fn compute_trajectory(
    detections: &[BallDetection],
    frames: &[ImageFrame],
) -> Result<TrajectoryResult, LaunchTracError> {
    if detections.len() < 2 {
        return Err(LaunchTracError::Vision(
            "Need at least 2 detections for trajectory".into(),
        ));
    }
    if frames.len() < 2 {
        return Err(LaunchTracError::Vision(
            "Need at least 2 frames for trajectory".into(),
        ));
    }

    let first = &detections[0];
    let last = detections.last().unwrap();

    // Time delta between first and last frame in seconds
    let first_ts = frames[0].timestamp;
    let last_ts = frames[frames.len() - 1].timestamp;
    let time_delta_s = (last_ts - first_ts).num_microseconds().unwrap_or(1) as f64 / 1_000_000.0;

    if time_delta_s <= 0.0 {
        return Err(LaunchTracError::Vision(
            "Invalid timestamp delta between frames".into(),
        ));
    }

    // Estimate distance from camera using ball apparent size.
    // distance = (focal_length_px * real_diameter) / apparent_diameter_px
    let focal_length_px = estimate_focal_length(frames[0].width);

    let dist_first = estimate_depth(focal_length_px, first.radius);
    let dist_last = estimate_depth(focal_length_px, last.radius);

    // Convert pixel coordinates to 3D world coordinates
    let (x1, y1, z1) = pixel_to_3d(
        first.cx,
        first.cy,
        dist_first,
        focal_length_px,
        frames[0].width as f64,
        frames[0].height as f64,
    );

    let (x2, y2, z2) = pixel_to_3d(
        last.cx,
        last.cy,
        dist_last,
        focal_length_px,
        frames[0].width as f64,
        frames[0].height as f64,
    );

    // 3D displacement
    let dx = x2 - x1;
    let dy = y2 - y1; // Horizontal (left/right)
    let dz = z2 - z1; // Depth (away from camera)

    // Speed = 3D distance / time
    let distance_3d = (dx * dx + dy * dy + dz * dz).sqrt();
    let speed_ms = distance_3d / time_delta_s;

    // Vertical launch angle: angle of vertical component vs horizontal plane
    // In image coordinates: x = horizontal, y = vertical (inverted), z = depth
    let horizontal_speed = (dy * dy + dz * dz).sqrt();
    let vla_deg = (-dx).atan2(horizontal_speed).to_degrees(); // Negative y = upward in image

    // Horizontal launch angle: angle in the ground plane
    let hla_deg = dy.atan2(dz).to_degrees();

    // Confidence based on number of detections and consistency
    let confidence = compute_confidence(detections, time_delta_s);

    Ok(TrajectoryResult {
        speed_ms,
        vla_deg,
        hla_deg,
        confidence,
    })
}

/// Estimate depth (distance from camera) using apparent ball size.
/// Z = (f * D_real) / D_apparent
fn estimate_depth(focal_length_px: f64, radius_px: f64) -> f64 {
    let diameter_px = radius_px * 2.0;
    if diameter_px > 0.0 {
        (focal_length_px * BALL_DIAMETER_M) / diameter_px
    } else {
        1.0 // Default 1m if radius unknown
    }
}

/// Convert pixel coordinates to 3D world coordinates (camera frame).
/// Uses pinhole camera model: x_world = (u - cx) * Z / f
fn pixel_to_3d(
    px: f64,
    py: f64,
    depth: f64,
    focal_length_px: f64,
    img_width: f64,
    img_height: f64,
) -> (f64, f64, f64) {
    let cx = img_width / 2.0;
    let cy = img_height / 2.0;

    let x = (py - cy) * depth / focal_length_px; // Vertical (image y → world vertical)
    let y = (px - cx) * depth / focal_length_px; // Horizontal (image x → world horizontal)
    let z = depth; // Depth

    (x, y, z)
}

/// Estimate focal length in pixels from image width.
/// For 6mm lens on IMX296 (pixel size 3.45um):
///   f_px = f_mm / pixel_size_mm = 6.0 / 0.00345 ≈ 1739
/// For 3.6mm lens: f_px ≈ 1043
fn estimate_focal_length(image_width: u32) -> f64 {
    // Default to 6mm lens on IMX296
    // This should come from calibration in production
    if image_width >= 1400 {
        1739.0 // 6mm lens
    } else {
        1043.0 // 3.6mm lens
    }
}

/// Compute trajectory confidence from detection quality
fn compute_confidence(detections: &[BallDetection], time_delta_s: f64) -> f64 {
    let mut confidence = 0.5;

    // More detections = higher confidence
    if detections.len() >= 4 {
        confidence += 0.2;
    } else if detections.len() >= 3 {
        confidence += 0.1;
    }

    // Higher detection confidence = higher trajectory confidence
    let avg_det_conf: f64 =
        detections.iter().map(|d| d.confidence).sum::<f64>() / detections.len() as f64;
    confidence += avg_det_conf * 0.2;

    // Reasonable time delta (not too short, not too long)
    if (0.001..=0.1).contains(&time_delta_s) {
        confidence += 0.1;
    }

    confidence.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn make_detection(cx: f64, cy: f64, radius: f64) -> BallDetection {
        BallDetection {
            cx,
            cy,
            radius,
            confidence: 0.9,
        }
    }

    fn make_frame(seq: u64, offset_us: i64) -> ImageFrame {
        ImageFrame {
            data: vec![128u8; 1456 * 1088],
            width: 1456,
            height: 1088,
            timestamp: Utc::now() + Duration::microseconds(offset_us),
            sequence: seq,
        }
    }

    #[test]
    fn estimate_depth_from_ball_size() {
        let focal_px = 1739.0;
        // Ball at ~40cm should have radius ~87px (from LaunchTrac v1 calibration)
        let depth = estimate_depth(focal_px, 87.0);
        // Expected: (1739 * 0.04267) / 174 ≈ 0.426m
        assert!(
            (depth - 0.426).abs() < 0.05,
            "Depth should be ~0.43m, got {depth}"
        );
    }

    #[test]
    fn trajectory_requires_two_detections() {
        let frames = vec![make_frame(0, 0), make_frame(1, 10000)];
        let detections = vec![make_detection(100.0, 100.0, 50.0)];
        assert!(compute_trajectory(&detections, &frames).is_err());
    }

    #[test]
    fn moving_ball_has_positive_speed() {
        let frames = vec![make_frame(0, 0), make_frame(1, 5000)]; // 5ms apart
        let detections = vec![
            make_detection(400.0, 600.0, 80.0), // Ball position 1
            make_detection(600.0, 500.0, 70.0), // Ball position 2 (moved right and up)
        ];

        let result = compute_trajectory(&detections, &frames).unwrap();
        assert!(result.speed_ms > 0.0, "Speed should be positive");
    }

    #[test]
    fn upward_ball_has_positive_vla() {
        let frames = vec![make_frame(0, 0), make_frame(1, 5000)];
        let detections = vec![
            make_detection(728.0, 800.0, 80.0), // Center-ish, lower
            make_detection(728.0, 400.0, 70.0), // Center-ish, higher (smaller y = higher)
        ];

        let result = compute_trajectory(&detections, &frames).unwrap();
        assert!(
            result.vla_deg > 0.0,
            "VLA should be positive for upward ball, got {}",
            result.vla_deg
        );
    }
}
