// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2022-2025, Verdant Consultants, LLC. (original PiTrac code)
// Copyright (C) 2026, LaunchTrac contributors
//
// This file is part of LaunchTrac, a derivative work of PiTrac
// (https://github.com/jeshernandez/PiTrac). Both projects are licensed
// under the GNU General Public License v2.0.
//
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use launchtrac_common::error::LaunchTracError;

use crate::{CameraCapture, GpioController, ImageFrame, PulseTiming, PwmStrobe};

/// Record of a GPIO call, for test assertions
#[derive(Debug, Clone)]
pub enum HardwareCall {
    GpioSet { pin: u32, value: bool },
    GpioRead { pin: u32 },
    PulseTrain { pulses: Vec<PulseTiming> },
    CameraTrigger { duration_us: u64 },
}

/// Metadata for a fixture frame (read from metadata.json)
#[derive(Debug, Clone, serde::Deserialize)]
struct FrameMeta {
    index: usize,
    ball_cx: f64,
    ball_cy: f64,
    ball_radius: f64,
    time_offset_us: u64,
}

/// Fixture metadata (read from metadata.json)
#[derive(Debug, Clone, serde::Deserialize)]
struct FixtureMeta {
    name: String,
    width: u32,
    height: u32,
    frame_count: usize,
    frames: Vec<FrameMeta>,
}

/// Mock hardware that replays captured image sequences.
/// Used for development and testing without a physical Pi + cameras.
///
/// Usage:
///   let mock = MockHardware::from_fixture("tests/fixtures/driver_shot_001/");
///   // Contains: frame_000.raw, frame_001.raw, ..., metadata.json
pub struct MockHardware {
    calls: Arc<Mutex<Vec<HardwareCall>>>,
    frame_index: Arc<Mutex<usize>>,
    frames: Vec<ImageFrame>,
    width: u32,
    height: u32,
}

impl MockHardware {
    /// Create a mock with no fixture data (returns blank frames)
    pub fn new() -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
            frame_index: Arc::new(Mutex::new(0)),
            frames: Vec::new(),
            width: 1456,
            height: 1088,
        }
    }

    /// Create a mock that replays images from a fixture directory.
    ///
    /// Expected directory layout:
    ///   metadata.json     — frame metadata (positions, timing)
    ///   frame_000.raw     — raw grayscale pixels (width * height bytes)
    ///   frame_001.raw
    ///   ...
    pub fn from_fixture(path: impl Into<PathBuf>) -> Result<Self, LaunchTracError> {
        let path = path.into();
        tracing::info!(fixture = %path.display(), "Loading mock hardware fixture");

        // Load metadata
        let meta_path = path.join("metadata.json");
        let meta_str = std::fs::read_to_string(&meta_path).map_err(|e| {
            LaunchTracError::Hardware(format!("Failed to read {}: {e}", meta_path.display()))
        })?;
        let meta: FixtureMeta = serde_json::from_str(&meta_str).map_err(|e| {
            LaunchTracError::Hardware(format!("Failed to parse metadata.json: {e}"))
        })?;

        tracing::info!(
            name = %meta.name,
            frames = meta.frame_count,
            size = format!("{}x{}", meta.width, meta.height),
            "Fixture loaded"
        );

        // Load each frame
        let base_time = chrono::Utc::now();
        let mut frames = Vec::with_capacity(meta.frame_count);

        for frame_meta in &meta.frames {
            let frame_path = path.join(format!("frame_{:03}.raw", frame_meta.index));
            let data = std::fs::read(&frame_path).map_err(|e| {
                LaunchTracError::Hardware(format!("Failed to read {}: {e}", frame_path.display()))
            })?;

            let expected_size = (meta.width * meta.height) as usize;
            if data.len() != expected_size {
                return Err(LaunchTracError::Hardware(format!(
                    "Frame {} size mismatch: expected {} bytes, got {}",
                    frame_meta.index,
                    expected_size,
                    data.len()
                )));
            }

            let timestamp =
                base_time + chrono::Duration::microseconds(frame_meta.time_offset_us as i64);

            frames.push(ImageFrame {
                data,
                width: meta.width,
                height: meta.height,
                timestamp,
                sequence: frame_meta.index as u64,
            });

            tracing::debug!(
                frame = frame_meta.index,
                ball_cx = frame_meta.ball_cx,
                ball_cy = frame_meta.ball_cy,
                ball_r = frame_meta.ball_radius,
                time_us = frame_meta.time_offset_us,
                "Frame loaded"
            );
        }

        Ok(Self {
            calls: Arc::new(Mutex::new(Vec::new())),
            frame_index: Arc::new(Mutex::new(0)),
            frames,
            width: meta.width,
            height: meta.height,
        })
    }

    /// Create a mock from pre-built ImageFrames (for unit tests)
    pub fn from_frames(frames: Vec<ImageFrame>) -> Self {
        let (width, height) = if let Some(f) = frames.first() {
            (f.width, f.height)
        } else {
            (1456, 1088)
        };

        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
            frame_index: Arc::new(Mutex::new(0)),
            frames,
            width,
            height,
        }
    }

    /// Get all recorded hardware calls (for test assertions)
    pub fn calls(&self) -> Vec<HardwareCall> {
        self.calls.lock().unwrap().clone()
    }

    /// Clear recorded calls
    pub fn clear_calls(&self) {
        self.calls.lock().unwrap().clear();
    }

    /// Reset frame index to replay from beginning
    pub fn reset(&self) {
        *self.frame_index.lock().unwrap() = 0;
    }

    /// Get total number of loaded frames
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    fn record(&self, call: HardwareCall) {
        self.calls.lock().unwrap().push(call);
    }

    fn next_frame(&self) -> ImageFrame {
        let mut idx = self.frame_index.lock().unwrap();
        let frame = if *idx < self.frames.len() {
            self.frames[*idx].clone()
        } else {
            Self::blank_frame(*idx as u64, self.width, self.height)
        };
        *idx += 1;
        frame
    }

    fn blank_frame(sequence: u64, width: u32, height: u32) -> ImageFrame {
        ImageFrame {
            data: vec![128u8; (width * height) as usize],
            width,
            height,
            timestamp: chrono::Utc::now(),
            sequence,
        }
    }
}

impl Default for MockHardware {
    fn default() -> Self {
        Self::new()
    }
}

impl GpioController for MockHardware {
    fn set_pin(&self, pin: u32, value: bool) -> Result<(), LaunchTracError> {
        self.record(HardwareCall::GpioSet { pin, value });
        Ok(())
    }

    fn read_pin(&self, pin: u32) -> Result<bool, LaunchTracError> {
        self.record(HardwareCall::GpioRead { pin });
        Ok(false)
    }
}

impl PwmStrobe for MockHardware {
    fn send_pulse_train(&self, pulses: &[PulseTiming]) -> Result<(), LaunchTracError> {
        self.record(HardwareCall::PulseTrain {
            pulses: pulses.to_vec(),
        });
        Ok(())
    }

    fn trigger_camera(&self, duration_us: u64) -> Result<(), LaunchTracError> {
        self.record(HardwareCall::CameraTrigger { duration_us });
        Ok(())
    }
}

impl CameraCapture for MockHardware {
    fn start_preview(&mut self) -> Result<(), LaunchTracError> {
        tracing::info!("Mock camera: starting preview");
        Ok(())
    }

    fn capture_still(&mut self) -> Result<ImageFrame, LaunchTracError> {
        Ok(self.next_frame())
    }

    fn capture_strobed(&mut self) -> Result<Vec<ImageFrame>, LaunchTracError> {
        // Return all loaded frames as the strobed sequence
        if self.frames.is_empty() {
            return Ok(vec![self.next_frame(), self.next_frame()]);
        }
        // Return all remaining frames
        let mut result = Vec::new();
        let idx = *self.frame_index.lock().unwrap();
        for i in idx..self.frames.len() {
            result.push(self.frames[i].clone());
        }
        *self.frame_index.lock().unwrap() = self.frames.len();
        if result.is_empty() {
            result.push(Self::blank_frame(0, self.width, self.height));
        }
        Ok(result)
    }

    fn stop(&mut self) -> Result<(), LaunchTracError> {
        tracing::info!("Mock camera: stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_records_gpio_calls() {
        let mock = MockHardware::new();
        mock.set_pin(25, true).unwrap();
        mock.set_pin(18, false).unwrap();

        let calls = mock.calls();
        assert_eq!(calls.len(), 2);
    }

    #[test]
    fn mock_records_pulse_train() {
        let mock = MockHardware::new();
        let pulses = vec![
            PulseTiming {
                delay_us: 700,
                duration_us: 20,
            },
            PulseTiming {
                delay_us: 1800,
                duration_us: 20,
            },
        ];
        mock.send_pulse_train(&pulses).unwrap();

        let calls = mock.calls();
        assert_eq!(calls.len(), 1);
    }

    #[test]
    fn mock_camera_returns_frames() {
        let mut mock = MockHardware::new();
        let frame = mock.capture_still().unwrap();
        assert_eq!(frame.width, 1456);
        assert_eq!(frame.height, 1088);
        assert_eq!(frame.data.len(), 1456 * 1088);
    }

    #[test]
    fn mock_from_frames_replays_in_order() {
        let frames = vec![
            ImageFrame {
                data: vec![100u8; 100],
                width: 10,
                height: 10,
                timestamp: chrono::Utc::now(),
                sequence: 0,
            },
            ImageFrame {
                data: vec![200u8; 100],
                width: 10,
                height: 10,
                timestamp: chrono::Utc::now(),
                sequence: 1,
            },
        ];

        let mut mock = MockHardware::from_frames(frames);
        let f1 = mock.capture_still().unwrap();
        let f2 = mock.capture_still().unwrap();

        assert_eq!(f1.data[0], 100);
        assert_eq!(f2.data[0], 200);
        assert_eq!(f1.sequence, 0);
        assert_eq!(f2.sequence, 1);
    }
}
