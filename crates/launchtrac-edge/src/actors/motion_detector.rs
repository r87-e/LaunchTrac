// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2022-2025, Verdant Consultants, LLC. (original PiTrac code)
// Copyright (C) 2026, LaunchTrac contributors
//
// This file is part of LaunchTrac, a derivative work of PiTrac
// (https://github.com/jeshernandez/PiTrac). Both projects are licensed
// under the GNU General Public License v2.0.
//
use launchtrac_hal::ImageFrame;
use tokio::sync::mpsc;

/// Motion detector actor — analyzes preview frames for ball movement.
///
/// Detects when a golf ball starts moving (club impact) by comparing
/// consecutive frames for significant pixel change in the ball region.
///
/// When motion is detected, triggers the strobe controller.
pub async fn run(
    mut frame_rx: mpsc::Receiver<ImageFrame>,
    motion_tx: mpsc::Sender<()>,
) -> anyhow::Result<()> {
    tracing::info!("Motion detector actor starting");

    let mut prev_frame: Option<ImageFrame> = None;

    while let Some(frame) = frame_rx.recv().await {
        if let Some(ref prev) = prev_frame {
            if detect_motion(prev, &frame) {
                tracing::info!(seq = frame.sequence, "Motion detected!");
                if motion_tx.send(()).await.is_err() {
                    break;
                }
                // Cool down — don't re-trigger immediately
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }

        prev_frame = Some(frame);
    }

    tracing::info!("Motion detector actor shutting down");
    Ok(())
}

/// Simple frame differencing to detect ball motion.
///
/// Compares pixel intensity sum in a region of interest.
/// Real implementation will use the ball's known position from Camera 1.
fn detect_motion(prev: &ImageFrame, curr: &ImageFrame) -> bool {
    if prev.data.len() != curr.data.len() {
        return false;
    }

    // Sum of absolute differences across all pixels
    let diff_sum: u64 = prev
        .data
        .iter()
        .zip(curr.data.iter())
        .map(|(&a, &b)| (a as i32 - b as i32).unsigned_abs() as u64)
        .sum();

    let pixel_count = prev.data.len() as u64;
    let avg_diff = diff_sum / pixel_count.max(1);

    // Threshold: average per-pixel change > 10 indicates significant motion
    const MOTION_THRESHOLD: u64 = 10;
    avg_diff > MOTION_THRESHOLD
}
