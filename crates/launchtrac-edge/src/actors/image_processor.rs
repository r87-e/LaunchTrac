// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2022-2025, Verdant Consultants, LLC. (original PiTrac code)
// Copyright (C) 2026, LaunchTrac contributors
//
// This file is part of LaunchTrac, a derivative work of PiTrac
// (https://github.com/PiTracLM/PiTrac). Both projects are licensed
// under the GNU General Public License v2.0.
//
use launchtrac_common::shot::ShotData;
use launchtrac_common::types::ClubType;
use launchtrac_hal::ImageFrame;
use launchtrac_vision::VisionPipeline;
use tokio::sync::mpsc;

/// Image processor actor — runs the vision pipeline on strobed frames.
///
/// Receives captured frame sequences and produces ShotData results.
/// This is where YOLO detection, trajectory analysis, and spin estimation happen.
pub async fn run(
    mut strobed_rx: mpsc::Receiver<Vec<ImageFrame>>,
    shot_tx: mpsc::Sender<ShotData>,
) -> anyhow::Result<()> {
    tracing::info!("Image processor actor starting");

    let pipeline = VisionPipeline::new()?;
    let mut shot_number: u32 = 0;

    while let Some(frames) = strobed_rx.recv().await {
        tracing::info!(frames = frames.len(), "Processing shot");

        match pipeline.process_shot(&frames) {
            Ok(analysis) => {
                shot_number += 1;
                let shot = analysis.to_shot_data(shot_number, ClubType::Driver);

                tracing::info!(
                    shot = shot_number,
                    speed = format!("{:.1}", shot.speed_mph),
                    vla = format!("{:.1}", shot.vla_deg),
                    hla = format!("{:.1}", shot.hla_deg),
                    backspin = shot.backspin_rpm,
                    sidespin = shot.sidespin_rpm,
                    confidence = format!("{:.2}", shot.confidence),
                    time_ms = shot.processing_time_ms,
                    "Shot processed"
                );

                if shot_tx.send(shot).await.is_err() {
                    break;
                }
            }
            Err(e) => {
                tracing::warn!("Shot processing failed: {e}");
            }
        }
    }

    tracing::info!("Image processor actor shutting down");
    Ok(())
}
