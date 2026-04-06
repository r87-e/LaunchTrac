// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2022-2025, Verdant Consultants, LLC. (original PiTrac code)
// Copyright (C) 2026, LaunchTrac contributors
//
// This file is part of LaunchTrac, a derivative work of PiTrac
// (https://github.com/jeshernandez/PiTrac). Both projects are licensed
// under the GNU General Public License v2.0.
//
/// Process a fixture through the full LaunchTrac vision pipeline.
///
/// This loads a fixture directory (raw frames + metadata), runs ball detection,
/// trajectory analysis, and spin estimation, then prints the shot result
/// and GSPro-ready JSON.
///
/// Run: cargo run -p launchtrac-edge --example process_fixture -- tests/fixtures/driver_shot_001
///   or: cargo run -p launchtrac-edge --example process_fixture -- tests/fixtures/iron7_shot_001
use std::env;

use launchtrac_common::types::ClubType;
use launchtrac_hal::CameraCapture;
use launchtrac_hal::mock::MockHardware;
use launchtrac_sim_proto::gspro::GsProInterface;
use launchtrac_vision::VisionPipeline;

fn main() {
    // Init logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    let args: Vec<String> = env::args().collect();
    let fixture_path = args.get(1).unwrap_or_else(|| {
        eprintln!("Usage: process_fixture <fixture_dir>");
        eprintln!("  e.g.: process_fixture tests/fixtures/driver_shot_001");
        std::process::exit(1);
    });

    println!("========================================");
    println!("  LaunchTrac v2 — Fixture Processor");
    println!("========================================\n");

    // Load fixture
    println!("Loading fixture: {fixture_path}");
    let mut mock = MockHardware::from_fixture(fixture_path).unwrap_or_else(|e| {
        eprintln!("Failed to load fixture: {e}");
        std::process::exit(1);
    });

    println!("  Loaded {} frames\n", mock.frame_count());

    // Capture all frames as a strobed sequence
    let frames = mock.capture_strobed().unwrap();
    println!("Captured {} strobed frames", frames.len());

    for (i, frame) in frames.iter().enumerate() {
        println!(
            "  Frame {}: {}x{} | seq={} | t=+{}us",
            i,
            frame.width,
            frame.height,
            frame.sequence,
            if i > 0 {
                (frame.timestamp - frames[0].timestamp)
                    .num_microseconds()
                    .unwrap_or(0)
            } else {
                0
            }
        );
    }

    // Initialize vision pipeline
    println!("\nInitializing vision pipeline...");
    let pipeline = VisionPipeline::new().unwrap();

    // Process the shot
    println!("Processing shot...\n");
    let start = std::time::Instant::now();
    let result = pipeline.process_shot(&frames);
    let elapsed = start.elapsed();

    match result {
        Ok(analysis) => {
            println!("========================================");
            println!("  SHOT RESULT");
            println!("========================================");
            println!();
            println!(
                "  Ball Detections: {}/{} frames",
                analysis.detections.len(),
                frames.len()
            );
            for (i, det) in analysis.detections.iter().enumerate() {
                println!(
                    "    [{i}] pos=({:.1}, {:.1})  radius={:.1}px  confidence={:.0}%",
                    det.cx,
                    det.cy,
                    det.radius,
                    det.confidence * 100.0
                );
            }

            println!();
            println!("  --- Trajectory ---");
            println!("  Speed:         {:.1} m/s", analysis.speed_ms);
            println!("  VLA:           {:.1} deg", analysis.vla_deg);
            println!("  HLA:           {:.1} deg", analysis.hla_deg);

            println!();
            println!("  --- Spin ---");
            println!("  Backspin:      {} rpm", analysis.backspin_rpm);
            println!("  Sidespin:      {} rpm", analysis.sidespin_rpm);

            println!();
            println!("  --- Meta ---");
            println!("  Confidence:    {:.0}%", analysis.confidence * 100.0);
            println!(
                "  Pipeline time: {:.1}s ({} ms)",
                elapsed.as_secs_f64(),
                elapsed.as_millis()
            );

            // Convert to ShotData (what simulators receive)
            let shot = analysis.to_shot_data(1, ClubType::Driver);

            println!();
            println!("========================================");
            println!("  SIMULATOR OUTPUT");
            println!("========================================");
            println!();
            println!("  Ball Speed:    {:.1} mph", shot.speed_mph);
            println!("  Launch Angle:  {:.1} deg (vertical)", shot.vla_deg);
            println!("  Launch Dir:    {:.1} deg (horizontal)", shot.hla_deg);
            println!("  Backspin:      {} rpm", shot.backspin_rpm);
            println!("  Sidespin:      {} rpm", shot.sidespin_rpm);
            println!("  Spin Axis:     {:.1} deg", shot.spin_axis_deg);
            println!("  Total Spin:    {:.0} rpm", shot.total_spin_rpm);

            // Print GSPro-ready JSON
            println!();
            println!("========================================");
            println!("  GSPro JSON (ready to send to port 921)");
            println!("========================================");
            let gspro = build_gspro_json(&shot);
            println!("{gspro}");
        }
        Err(e) => {
            println!("ERROR: Pipeline failed: {e}");
            println!();
            println!("This usually means the ball detector couldn't find the ball.");
            println!("Check that the fixture has bright circular objects in the frames.");
            std::process::exit(1);
        }
    }
}

fn build_gspro_json(shot: &launchtrac_common::shot::ShotData) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "DeviceID": "LaunchTrac LM",
        "Units": "Yards",
        "ShotNumber": shot.shot_number,
        "APIversion": "1",
        "BallData": {
            "Speed": format!("{:.1}", shot.speed_mph),
            "SpinAxis": format!("{:.1}", shot.spin_axis_deg),
            "TotalSpin": format!("{:.1}", shot.total_spin_rpm),
            "BackSpin": format!("{}", shot.backspin_rpm),
            "SideSpin": format!("{}", shot.sidespin_rpm),
            "HLA": format!("{:.1}", shot.hla_deg),
            "VLA": format!("{:.1}", shot.vla_deg)
        },
        "ShotDataOptions": {
            "ContainsBallData": true,
            "ContainsClubData": false,
            "LaunchMonitorIsReady": true,
            "LaunchMonitorBallDetected": true,
            "IsHeartBeat": false
        }
    }))
    .unwrap()
}
