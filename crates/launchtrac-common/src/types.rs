// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2022-2025, Verdant Consultants, LLC. (original PiTrac code)
// Copyright (C) 2026, LaunchTrac contributors
//
// This file is part of LaunchTrac, a derivative work of PiTrac
// (https://github.com/jeshernandez/PiTrac). Both projects are licensed
// under the GNU General Public License v2.0.
//
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Golf club classification
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ClubType {
    #[default]
    Driver,
    Wood3,
    Wood5,
    Hybrid,
    Iron3,
    Iron4,
    Iron5,
    Iron6,
    Iron7,
    Iron8,
    Iron9,
    PitchingWedge,
    GapWedge,
    SandWedge,
    LobWedge,
    Putter,
}

impl ClubType {
    /// GSPro club code (2-letter abbreviation)
    pub fn gspro_code(&self) -> &'static str {
        match self {
            Self::Driver => "DR",
            Self::Wood3 => "3W",
            Self::Wood5 => "5W",
            Self::Hybrid => "HY",
            Self::Iron3 => "3I",
            Self::Iron4 => "4I",
            Self::Iron5 => "5I",
            Self::Iron6 => "6I",
            Self::Iron7 => "7I",
            Self::Iron8 => "8I",
            Self::Iron9 => "9I",
            Self::PitchingWedge => "PW",
            Self::GapWedge => "GW",
            Self::SandWedge => "SW",
            Self::LobWedge => "LW",
            Self::Putter => "PT",
        }
    }
}

/// Player handedness
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Handedness {
    #[serde(rename = "RH")]
    #[default]
    Right,
    #[serde(rename = "LH")]
    Left,
}

/// Unique device identifier for a LaunchTrac unit
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId(pub Uuid);

impl DeviceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for DeviceId {
    fn default() -> Self {
        Self::new()
    }
}

/// Supported measurement units
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Units {
    #[default]
    Imperial, // mph, yards
    Metric, // m/s, meters
}
