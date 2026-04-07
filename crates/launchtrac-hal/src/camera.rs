// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2022-2025, Verdant Consultants, LLC. (original PiTrac code)
// Copyright (C) 2026, LaunchTrac contributors
//
// This file is part of LaunchTrac, a derivative work of PiTrac
// (https://github.com/PiTracLM/PiTrac). Both projects are licensed
// under the GNU General Public License v2.0.
//
use launchtrac_common::error::LaunchTracError;

use crate::{CameraCapture, ImageFrame};

/// Real camera using libcamera (Pi Camera interface)
///
/// Supports:
///   - InnoMaker IMX296 Global Shutter (1456x1088, monochrome)
///   - Pi Camera GS (1456x1088)
pub struct LibCamera {
    camera_index: u32,
    _width: u32,
    _height: u32,
}

impl LibCamera {
    pub fn new(camera_index: u32) -> Result<Self, LaunchTracError> {
        // Default to IMX296 resolution
        Ok(Self {
            camera_index,
            _width: 1456,
            _height: 1088,
        })
    }
}

impl CameraCapture for LibCamera {
    fn start_preview(&mut self) -> Result<(), LaunchTracError> {
        tracing::info!(camera = self.camera_index, "Starting camera preview");
        // TODO: libcamera-rs preview start
        Ok(())
    }

    fn capture_still(&mut self) -> Result<ImageFrame, LaunchTracError> {
        tracing::debug!(camera = self.camera_index, "Capturing still");
        // TODO: libcamera-rs still capture
        Err(LaunchTracError::Camera(
            "Real camera not yet implemented".into(),
        ))
    }

    fn capture_strobed(&mut self) -> Result<Vec<ImageFrame>, LaunchTracError> {
        tracing::debug!(camera = self.camera_index, "Capturing strobed sequence");
        // TODO: External trigger capture with GPIO25 sync
        Err(LaunchTracError::Camera(
            "Real camera not yet implemented".into(),
        ))
    }

    fn stop(&mut self) -> Result<(), LaunchTracError> {
        tracing::info!(camera = self.camera_index, "Stopping camera");
        Ok(())
    }
}
