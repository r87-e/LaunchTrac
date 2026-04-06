// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2022-2025, Verdant Consultants, LLC. (original PiTrac code)
// Copyright (C) 2026, LaunchTrac contributors
//
// This file is part of LaunchTrac, a derivative work of PiTrac
// (https://github.com/jeshernandez/PiTrac). Both projects are licensed
// under the GNU General Public License v2.0.
//
pub mod ball_detector;
pub mod calibration;
pub mod spin_estimator;
pub mod trajectory;

use launchtrac_common::error::LaunchTracError;
use launchtrac_common::shot::ShotData;
use launchtrac_common::types::ClubType;
use launchtrac_hal::ImageFrame;

/// Result of ball detection in a single frame
#[derive(Debug, Clone)]
pub struct BallDetection {
    /// Center X in pixels
    pub cx: f64,
    /// Center Y in pixels
    pub cy: f64,
    /// Ball radius in pixels
    pub radius: f64,
    /// Detection confidence (0.0 - 1.0)
    pub confidence: f64,
}

/// Result of processing a full shot sequence
#[derive(Debug, Clone)]
pub struct ShotAnalysis {
    /// Ball detections across frames
    pub detections: Vec<BallDetection>,
    /// Computed ball speed in m/s
    pub speed_ms: f64,
    /// Vertical launch angle in degrees
    pub vla_deg: f64,
    /// Horizontal launch angle in degrees
    pub hla_deg: f64,
    /// Backspin in RPM
    pub backspin_rpm: i32,
    /// Sidespin in RPM
    pub sidespin_rpm: i32,
    /// Overall confidence
    pub confidence: f64,
    /// Processing time in ms
    pub processing_time_ms: u32,
}

impl ShotAnalysis {
    /// Convert to ShotData for output
    pub fn to_shot_data(&self, shot_number: u32, club: ClubType) -> ShotData {
        let speed_mph = self.speed_ms * 2.237;
        ShotData::new(
            shot_number,
            speed_mph,
            self.vla_deg,
            self.hla_deg,
            self.backspin_rpm,
            self.sidespin_rpm,
            club,
            self.confidence,
            self.processing_time_ms,
        )
    }
}

/// Main vision pipeline — processes a sequence of strobed frames into shot data
pub struct VisionPipeline {
    detector: ball_detector::BallDetector,
    spin_estimator: spin_estimator::SpinEstimator,
}

impl VisionPipeline {
    pub fn new() -> Result<Self, LaunchTracError> {
        Ok(Self {
            detector: ball_detector::BallDetector::new()?,
            spin_estimator: spin_estimator::SpinEstimator::new()?,
        })
    }

    /// Process a sequence of strobed frames from a single shot
    pub fn process_shot(&self, frames: &[ImageFrame]) -> Result<ShotAnalysis, LaunchTracError> {
        let start = std::time::Instant::now();

        if frames.len() < 2 {
            return Err(LaunchTracError::Vision(
                "Need at least 2 frames for shot analysis".into(),
            ));
        }

        // Step 1: Detect ball in each frame
        let detections: Vec<BallDetection> = frames
            .iter()
            .filter_map(|frame| self.detector.detect(frame).ok())
            .flatten()
            .collect();

        if detections.len() < 2 {
            return Err(LaunchTracError::Vision(format!(
                "Only found {} ball detections, need at least 2",
                detections.len()
            )));
        }

        // Step 2: Compute trajectory (speed + angles)
        let trajectory = trajectory::compute_trajectory(&detections, frames)?;

        // Step 3: Estimate spin from ball surface analysis
        let spin = self.spin_estimator.estimate(frames, &detections)?;

        let processing_time_ms = start.elapsed().as_millis() as u32;

        Ok(ShotAnalysis {
            detections,
            speed_ms: trajectory.speed_ms,
            vla_deg: trajectory.vla_deg,
            hla_deg: trajectory.hla_deg,
            backspin_rpm: spin.backspin_rpm,
            sidespin_rpm: spin.sidespin_rpm,
            confidence: trajectory.confidence.min(spin.confidence),
            processing_time_ms,
        })
    }
}
