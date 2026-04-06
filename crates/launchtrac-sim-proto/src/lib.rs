// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2022-2025, Verdant Consultants, LLC. (original PiTrac code)
// Copyright (C) 2026, LaunchTrac contributors
//
// This file is part of LaunchTrac, a derivative work of PiTrac
// (https://github.com/jeshernandez/PiTrac). Both projects are licensed
// under the GNU General Public License v2.0.
//
pub mod e6;
pub mod gspro;

use launchtrac_common::error::LaunchTracError;
use launchtrac_common::shot::{Heartbeat, ShotData};

/// Trait for all golf simulator connections
#[async_trait::async_trait]
pub trait SimulatorInterface: Send + Sync {
    /// Connect to the simulator
    async fn connect(&mut self) -> Result<(), LaunchTracError>;

    /// Send shot data to the simulator
    async fn send_shot(&mut self, shot: &ShotData) -> Result<(), LaunchTracError>;

    /// Send a heartbeat/keep-alive message
    async fn send_heartbeat(&mut self, heartbeat: &Heartbeat) -> Result<(), LaunchTracError>;

    /// Check if connected
    fn is_connected(&self) -> bool;

    /// Disconnect from the simulator
    async fn disconnect(&mut self) -> Result<(), LaunchTracError>;

    /// Simulator name for logging
    fn name(&self) -> &str;
}
