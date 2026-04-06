// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2022-2025, Verdant Consultants, LLC. (original PiTrac code)
// Copyright (C) 2026, LaunchTrac contributors
//
// This file is part of LaunchTrac, a derivative work of PiTrac
// (https://github.com/jeshernandez/PiTrac). Both projects are licensed
// under the GNU General Public License v2.0.
//
pub mod camera1;
pub mod cloud_uploader;
pub mod image_processor;
pub mod motion_detector;
pub mod results_router;
pub mod strobe_controller;

use launchtrac_common::config::Config;
use launchtrac_common::error::LaunchTracError;
use launchtrac_common::shot::ShotData;
use launchtrac_hal::ImageFrame;
use tokio::sync::{broadcast, mpsc};

/// Messages flowing between actors
#[derive(Debug, Clone)]
pub enum PipelineMsg {
    /// New frame from camera 1 (tee watcher)
    PreviewFrame(ImageFrame),

    /// Motion detected — ball is moving
    MotionDetected,

    /// Strobe + capture complete — here are the strobed frames
    StrobedFrames(Vec<ImageFrame>),

    /// Vision pipeline produced a shot result
    ShotResult(ShotData),

    /// System command
    Shutdown,
}

/// The main actor pipeline orchestrator.
///
/// Wires together all actors with typed message channels:
///
/// ```text
/// [Camera1] → [MotionDetector] → [StrobeController]
///                                        ↓
///                                 [ImageProcessor]
///                                        ↓
///                                 [ResultsRouter] → [SimBridge]
///                                        ↓
///                                 [CloudUploader]
/// ```
pub struct Pipeline {
    config: Config,
    mock_mode: bool,
    fixture_path: Option<String>,
    shot_tx: broadcast::Sender<ShotData>,
}

impl Pipeline {
    pub fn new(
        config: Config,
        mock_mode: bool,
        fixture_path: Option<String>,
    ) -> Result<Self, LaunchTracError> {
        let (shot_tx, _) = broadcast::channel(32);

        Ok(Self {
            config,
            mock_mode,
            fixture_path,
            shot_tx,
        })
    }

    /// Get a receiver for shot results (used by web server)
    pub fn shot_subscriber(&self) -> broadcast::Receiver<ShotData> {
        self.shot_tx.subscribe()
    }

    /// Run the pipeline — this is the main loop
    pub async fn run(self) -> Result<(), LaunchTracError> {
        tracing::info!(mock = self.mock_mode, "Starting pipeline");

        // Create inter-actor channels
        let (frame_tx, frame_rx) = mpsc::channel::<ImageFrame>(16);
        let (motion_tx, motion_rx) = mpsc::channel::<()>(4);
        let (strobed_tx, strobed_rx) = mpsc::channel::<Vec<ImageFrame>>(4);
        let (shot_tx, shot_rx) = mpsc::channel::<ShotData>(16);

        // Spawn actors
        let camera1_handle = tokio::spawn({
            let mock = self.mock_mode;
            let fixture = self.fixture_path.clone();
            async move { camera1::run(mock, fixture, frame_tx).await }
        });

        let motion_handle =
            tokio::spawn(async move { motion_detector::run(frame_rx, motion_tx).await });

        let strobe_handle = tokio::spawn({
            let config = self.config.clone();
            let mock = self.mock_mode;
            async move { strobe_controller::run(config, mock, motion_rx, strobed_tx).await }
        });

        let processor_handle =
            tokio::spawn(async move { image_processor::run(strobed_rx, shot_tx).await });

        let router_handle = tokio::spawn({
            let config = self.config.clone();
            let broadcast_tx = self.shot_tx.clone();
            async move { results_router::run(config, shot_rx, broadcast_tx).await }
        });

        // Wait for all actors
        tokio::select! {
            r = camera1_handle => tracing::info!("Camera1 actor exited: {:?}", r),
            r = motion_handle => tracing::info!("Motion detector exited: {:?}", r),
            r = strobe_handle => tracing::info!("Strobe controller exited: {:?}", r),
            r = processor_handle => tracing::info!("Image processor exited: {:?}", r),
            r = router_handle => tracing::info!("Results router exited: {:?}", r),
        }

        Ok(())
    }
}
