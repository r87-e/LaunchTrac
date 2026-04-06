// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2022-2025, Verdant Consultants, LLC. (original PiTrac code)
// Copyright (C) 2026, LaunchTrac contributors
//
// This file is part of LaunchTrac, a derivative work of PiTrac
// (https://github.com/jeshernandez/PiTrac). Both projects are licensed
// under the GNU General Public License v2.0.
//
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::ClubType;

/// Core shot data produced by the vision pipeline.
/// This is the single source of truth for a golf shot measurement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShotData {
    /// Unique shot identifier
    pub id: Uuid,

    /// Sequential shot number within a session
    pub shot_number: u32,

    /// Ball speed in miles per hour
    pub speed_mph: f64,

    /// Vertical launch angle in degrees (positive = upward)
    pub vla_deg: f64,

    /// Horizontal launch angle in degrees (negative = left)
    pub hla_deg: f64,

    /// Backspin in RPM
    pub backspin_rpm: i32,

    /// Sidespin in RPM (negative = left spin / draw)
    pub sidespin_rpm: i32,

    /// Spin axis in degrees, derived from backspin and sidespin.
    /// atan(sidespin / backspin) * 180 / PI
    pub spin_axis_deg: f64,

    /// Total spin magnitude in RPM
    pub total_spin_rpm: f64,

    /// Club used for this shot
    pub club: ClubType,

    /// Confidence score from the vision pipeline (0.0 - 1.0)
    pub confidence: f64,

    /// Processing time from trigger to result, in milliseconds
    pub processing_time_ms: u32,

    /// When the shot was captured
    pub timestamp: DateTime<Utc>,
}

impl ShotData {
    /// Create a new shot from raw measurements
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        shot_number: u32,
        speed_mph: f64,
        vla_deg: f64,
        hla_deg: f64,
        backspin_rpm: i32,
        sidespin_rpm: i32,
        club: ClubType,
        confidence: f64,
        processing_time_ms: u32,
    ) -> Self {
        let spin_axis_deg = if backspin_rpm != 0 {
            (sidespin_rpm as f64 / backspin_rpm as f64)
                .atan()
                .to_degrees()
        } else {
            0.0
        };

        let total_spin_rpm = ((backspin_rpm as f64).powi(2) + (sidespin_rpm as f64).powi(2)).sqrt();

        Self {
            id: Uuid::new_v4(),
            shot_number,
            speed_mph,
            vla_deg,
            hla_deg,
            backspin_rpm,
            sidespin_rpm,
            spin_axis_deg,
            total_spin_rpm,
            club,
            confidence,
            processing_time_ms,
            timestamp: Utc::now(),
        }
    }

    /// Convert speed to meters per second
    pub fn speed_ms(&self) -> f64 {
        self.speed_mph * 0.44704
    }
}

/// Heartbeat message sent to simulators to maintain connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    /// Whether the launch monitor has detected a ball on the tee
    pub ball_detected: bool,

    /// Whether the launch monitor is ready to capture
    pub ready: bool,
}

impl Default for Heartbeat {
    fn default() -> Self {
        Self {
            ball_detected: false,
            ready: true,
        }
    }
}
