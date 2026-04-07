// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2022-2025, Verdant Consultants, LLC. (original PiTrac code)
// Copyright (C) 2026, LaunchTrac contributors
//
// This file is part of LaunchTrac, a derivative work of PiTrac
// (https://github.com/PiTracLM/PiTrac). Both projects are licensed
// under the GNU General Public License v2.0.
//
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use launchtrac_common::error::LaunchTracError;
use launchtrac_common::shot::{Heartbeat, ShotData};

const DEFAULT_PORT: u16 = 2483;
const INTER_MESSAGE_DELAY_MS: u64 = 50;
// E6 parameter range limits (values are clamped to these)
const BACKSPIN_MIN: i32 = -999;
const BACKSPIN_MAX: i32 = 19999;
const BALL_SPEED_MIN: f64 = 0.09;
const BALL_SPEED_MAX: f64 = 249.9;
const SIDESPIN_MIN: i32 = -5999;
const SIDESPIN_MAX: i32 = 5999;

/// E6/TruGolf simulator interface.
///
/// Protocol: TCP port 2483 with 3-step shot sequence:
///   1. SetBallData → 50ms delay
///   2. SetClubData → 50ms delay
///   3. SendShot
///
/// Requires handshake on connect and explicit arming before shots.
pub struct E6Interface {
    address: String,
    port: u16,
    stream: Option<Mutex<TcpStream>>,
    connected: bool,
    armed: bool,
}

// -- E6 JSON message types --

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct E6TypedMessage {
    r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ball_data: Option<E6BallData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    club_data: Option<E6ClubData>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct E6BallData {
    back_spin: i32,
    ball_speed: f64,
    launch_angle: f64,
    launch_direction: f64,
    side_spin: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct E6ClubData {
    club_head_speed: String,
    club_angle_face: String,
    club_angle_path: String,
    #[serde(rename = "ClubHeadSpeedMPH")]
    club_head_speed_mph: String,
}

impl Default for E6ClubData {
    fn default() -> Self {
        Self {
            club_head_speed: "0.0".into(),
            club_angle_face: "0.0".into(),
            club_angle_path: "0.0".into(),
            club_head_speed_mph: "0.0".into(),
        }
    }
}

/// E6 response message types
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct E6Response {
    pub r#type: String,
}

impl E6Interface {
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            port: DEFAULT_PORT,
            stream: None,
            connected: false,
            armed: false,
        }
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Clamp a value to the E6 valid range
    fn clamp_backspin(v: i32) -> i32 {
        v.clamp(BACKSPIN_MIN, BACKSPIN_MAX)
    }

    fn clamp_sidespin(v: i32) -> i32 {
        v.clamp(SIDESPIN_MIN, SIDESPIN_MAX)
    }

    fn clamp_speed(v: f64) -> f64 {
        v.clamp(BALL_SPEED_MIN, BALL_SPEED_MAX)
    }

    async fn send_raw(&self, json: &str) -> Result<(), LaunchTracError> {
        if let Some(stream) = &self.stream {
            let mut stream = stream.lock().await;
            stream
                .write_all(json.as_bytes())
                .await
                .map_err(|e| LaunchTracError::Simulator(format!("E6 send failed: {e}")))?;
            Ok(())
        } else {
            Err(LaunchTracError::Simulator("Not connected to E6".into()))
        }
    }

    async fn send_typed_message(&self, msg: &E6TypedMessage) -> Result<(), LaunchTracError> {
        let json = serde_json::to_string(msg)
            .map_err(|e| LaunchTracError::Serialization(format!("E6 message: {e}")))?;
        tracing::debug!(msg_type = %msg.r#type, bytes = json.len(), "Sending E6 message");
        self.send_raw(&json).await
    }

    /// Send the 3-step shot sequence with required 50ms inter-message delays
    async fn send_shot_sequence(&self, shot: &ShotData) -> Result<(), LaunchTracError> {
        // Step 1: SetBallData
        let ball_msg = E6TypedMessage {
            r#type: "SetBallData".into(),
            ball_data: Some(E6BallData {
                back_spin: Self::clamp_backspin(shot.backspin_rpm),
                ball_speed: Self::clamp_speed(shot.speed_mph),
                launch_angle: shot.vla_deg,
                launch_direction: shot.hla_deg,
                side_spin: Self::clamp_sidespin(shot.sidespin_rpm),
            }),
            club_data: None,
        };
        self.send_typed_message(&ball_msg).await?;

        // 50ms delay (required by E6 protocol)
        tokio::time::sleep(tokio::time::Duration::from_millis(INTER_MESSAGE_DELAY_MS)).await;

        // Step 2: SetClubData
        let club_msg = E6TypedMessage {
            r#type: "SetClubData".into(),
            ball_data: None,
            club_data: Some(E6ClubData::default()),
        };
        self.send_typed_message(&club_msg).await?;

        // 50ms delay
        tokio::time::sleep(tokio::time::Duration::from_millis(INTER_MESSAGE_DELAY_MS)).await;

        // Step 3: SendShot
        let send_msg = E6TypedMessage {
            r#type: "SendShot".into(),
            ball_data: None,
            club_data: None,
        };
        self.send_typed_message(&send_msg).await?;

        Ok(())
    }

    /// Send handshake message on connection
    async fn handshake(&self) -> Result<(), LaunchTracError> {
        self.send_raw(r#"{"Type":"Handshake"}"#).await?;
        tracing::info!("E6 handshake sent");
        Ok(())
    }

    /// Set armed state
    pub fn set_armed(&mut self, armed: bool) {
        self.armed = armed;
        tracing::debug!(armed, "E6 armed state changed");
    }
}

#[async_trait::async_trait]
impl super::SimulatorInterface for E6Interface {
    async fn connect(&mut self) -> Result<(), LaunchTracError> {
        let addr = format!("{}:{}", self.address, self.port);
        tracing::info!(address = %addr, "Connecting to E6/TruGolf");

        let stream = TcpStream::connect(&addr)
            .await
            .map_err(|e| LaunchTracError::Simulator(format!("E6 connect to {addr}: {e}")))?;

        self.stream = Some(Mutex::new(stream));
        self.connected = true;

        // Send handshake
        self.handshake().await?;

        tracing::info!("Connected to E6/TruGolf");
        Ok(())
    }

    async fn send_shot(&mut self, shot: &ShotData) -> Result<(), LaunchTracError> {
        if !self.armed {
            tracing::warn!("E6 not armed, cannot send shot");
            return Err(LaunchTracError::Simulator("E6 not armed".into()));
        }

        self.send_shot_sequence(shot).await?;

        // Disarm after sending shot (E6 protocol requirement)
        self.armed = false;

        tracing::info!(
            shot = shot.shot_number,
            speed = shot.speed_mph,
            vla = shot.vla_deg,
            hla = shot.hla_deg,
            backspin = shot.backspin_rpm,
            sidespin = shot.sidespin_rpm,
            "Shot sent to E6"
        );

        Ok(())
    }

    async fn send_heartbeat(&mut self, _heartbeat: &Heartbeat) -> Result<(), LaunchTracError> {
        // E6 doesn't use the same heartbeat mechanism as GSPro.
        // Connection is maintained via the Ping/Pong response flow.
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn disconnect(&mut self) -> Result<(), LaunchTracError> {
        // Send disconnect message
        if self.connected {
            let _ = self.send_raw(r#"{"Type":"Disconnect"}"#).await;
        }
        self.stream = None;
        self.connected = false;
        self.armed = false;
        tracing::info!("Disconnected from E6");
        Ok(())
    }

    fn name(&self) -> &str {
        "E6/TruGolf"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e6_clamps_backspin() {
        assert_eq!(E6Interface::clamp_backspin(25000), BACKSPIN_MAX);
        assert_eq!(E6Interface::clamp_backspin(-2000), BACKSPIN_MIN);
        assert_eq!(E6Interface::clamp_backspin(3000), 3000);
    }

    #[test]
    fn e6_clamps_sidespin() {
        assert_eq!(E6Interface::clamp_sidespin(7000), SIDESPIN_MAX);
        assert_eq!(E6Interface::clamp_sidespin(-7000), SIDESPIN_MIN);
        assert_eq!(E6Interface::clamp_sidespin(200), 200);
    }

    #[test]
    fn e6_clamps_speed() {
        assert_eq!(E6Interface::clamp_speed(0.01), BALL_SPEED_MIN);
        assert_eq!(E6Interface::clamp_speed(300.0), BALL_SPEED_MAX);
        assert_eq!(E6Interface::clamp_speed(150.0), 150.0);
    }

    #[test]
    fn e6_ball_data_serializes_correctly() {
        let msg = E6TypedMessage {
            r#type: "SetBallData".into(),
            ball_data: Some(E6BallData {
                back_spin: 2800,
                ball_speed: 150.5,
                launch_angle: 12.3,
                launch_direction: -1.5,
                side_spin: -200,
            }),
            club_data: None,
        };

        let json = serde_json::to_string(&msg).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(v["Type"], "SetBallData");
        assert_eq!(v["BallData"]["BackSpin"], 2800);
        assert_eq!(v["BallData"]["BallSpeed"], 150.5);
        assert_eq!(v["BallData"]["LaunchAngle"], 12.3);
        assert_eq!(v["BallData"]["LaunchDirection"], -1.5);
        assert_eq!(v["BallData"]["SideSpin"], -200);
        assert!(v.get("ClubData").is_none());
    }

    #[test]
    fn e6_handshake_format() {
        let json = r#"{"Type":"Handshake"}"#;
        let v: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(v["Type"], "Handshake");
    }

    #[test]
    fn e6_send_shot_format() {
        let json = r#"{"Type":"SendShot"}"#;
        let v: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(v["Type"], "SendShot");
    }
}
