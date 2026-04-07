// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2022-2025, Verdant Consultants, LLC. (original PiTrac code)
// Copyright (C) 2026, LaunchTrac contributors
//
// This file is part of LaunchTrac, a derivative work of PiTrac
// (https://github.com/PiTracLM/PiTrac). Both projects are licensed
// under the GNU General Public License v2.0.
//
mod actors;
mod web;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use launchtrac_common::config::Config;

#[derive(Parser)]
#[command(name = "launchtrac", about = "LaunchTrac Golf Launch Monitor")]
struct Cli {
    /// Run in mock mode (replay test fixtures, no real hardware)
    #[arg(long)]
    mock: bool,

    /// Path to fixture directory for mock mode
    #[arg(long)]
    fixture: Option<String>,

    /// GSPro simulator address (e.g., "192.168.1.100")
    #[arg(long)]
    gspro: Option<String>,

    /// E6/TruGolf simulator address (e.g., "192.168.1.100")
    #[arg(long)]
    e6: Option<String>,

    /// Web UI port
    #[arg(long, default_value = "8080")]
    port: u16,

    /// Log level (error, warn, info, debug, trace)
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&cli.log_level)),
        )
        .init();

    tracing::info!("LaunchTrac v2 starting");

    // Load config
    let mut config = Config::load().unwrap_or_else(|e| {
        tracing::warn!("Failed to load config, using defaults: {e}");
        Config::default()
    });

    // Apply CLI overrides
    if let Some(ref addr) = cli.gspro {
        config.network.gspro_address = addr.clone();
    }
    if let Some(ref addr) = cli.e6 {
        config.network.e6_address = addr.clone();
    }
    config.network.web_port = cli.port;

    // Start the actor pipeline
    let pipeline = actors::Pipeline::new(config.clone(), cli.mock, cli.fixture)?;

    // Start web server in parallel with the pipeline
    let web_handle = tokio::spawn({
        let config = config.clone();
        let shot_rx = pipeline.shot_subscriber();
        async move { web::start_server(config, shot_rx).await }
    });

    // Run the main pipeline
    let pipeline_handle = tokio::spawn(async move { pipeline.run().await });

    // Wait for either to finish (or ctrl-c)
    tokio::select! {
        result = pipeline_handle => {
            tracing::info!("Pipeline exited: {:?}", result);
        }
        result = web_handle => {
            tracing::info!("Web server exited: {:?}", result);
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Shutting down");
        }
    }

    Ok(())
}
