// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2022-2025, Verdant Consultants, LLC. (original PiTrac code)
// Copyright (C) 2026, LaunchTrac contributors
//
// This file is part of LaunchTrac, a derivative work of PiTrac
// (https://github.com/jeshernandez/PiTrac). Both projects are licensed
// under the GNU General Public License v2.0.
//
use launchtrac_common::config::Config;
use launchtrac_common::shot::ShotData;
use launchtrac_sim_proto::SimulatorInterface;
use launchtrac_sim_proto::e6::E6Interface;
use launchtrac_sim_proto::gspro::GsProInterface;
use tokio::sync::{broadcast, mpsc};

/// Results router actor — fans out shot data to all connected outputs.
///
/// Destinations:
///   1. GSPro simulator (TCP, if configured)
///   2. E6/TruGolf simulator (TCP, if configured)
///   3. Web dashboard (broadcast channel → WebSocket)
///   4. Cloud uploader (future)
pub async fn run(
    config: Config,
    mut shot_rx: mpsc::Receiver<ShotData>,
    broadcast_tx: broadcast::Sender<ShotData>,
) -> anyhow::Result<()> {
    tracing::info!("Results router actor starting");

    // Initialize simulator connections
    let mut simulators: Vec<Box<dyn SimulatorInterface>> = Vec::new();

    if !config.network.gspro_address.is_empty() {
        let mut gspro = GsProInterface::new(&config.network.gspro_address);
        match gspro.connect().await {
            Ok(()) => simulators.push(Box::new(gspro)),
            Err(e) => tracing::warn!("Failed to connect to GSPro: {e}"),
        }
    }

    if !config.network.e6_address.is_empty() {
        let mut e6 = E6Interface::new(&config.network.e6_address);
        match e6.connect().await {
            Ok(()) => simulators.push(Box::new(e6)),
            Err(e) => tracing::warn!("Failed to connect to E6: {e}"),
        }
    }

    tracing::info!(
        sim_count = simulators.len(),
        "Simulator connections established"
    );

    while let Some(shot) = shot_rx.recv().await {
        // Send to all simulators
        for sim in &mut simulators {
            if let Err(e) = sim.send_shot(&shot).await {
                tracing::error!(sim = sim.name(), "Failed to send shot: {e}");
            }
        }

        // Broadcast to web dashboard subscribers
        let _ = broadcast_tx.send(shot);
    }

    // Clean disconnect
    for sim in &mut simulators {
        let _ = sim.disconnect().await;
    }

    tracing::info!("Results router actor shutting down");
    Ok(())
}
