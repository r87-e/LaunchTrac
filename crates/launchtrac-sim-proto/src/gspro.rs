use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use async_trait::async_trait;

use launchtrac_common::error::LaunchTracError;
use launchtrac_common::shot::{Heartbeat, ShotData};

const DEFAULT_PORT: u16 = 921;
const DEVICE_ID: &str = "LaunchTrac LM";
const API_VERSION: &str = "1";
const BUFFER_SIZE: usize = 2000;

/// GSPro simulator interface.
///
/// Protocol: Single JSON message per shot over TCP port 921.
/// Supports heartbeat keep-alive and receives player info responses.
pub struct GsProInterface {
    address: String,
    port: u16,
    stream: Option<Mutex<TcpStream>>,
    shot_number: u32,
    connected: bool,
}

// -- GSPro JSON message types --

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct GsProMessage {
    #[serde(rename = "DeviceID")]
    device_id: String,
    units: String,
    shot_number: u32,
    #[serde(rename = "APIversion")]
    api_version: String,
    ball_data: GsProBallData,
    club_data: GsProClubData,
    shot_data_options: GsProShotOptions,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct GsProBallData {
    speed: String,
    spin_axis: String,
    total_spin: String,
    back_spin: String,
    side_spin: String,
    #[serde(rename = "HLA")]
    hla: String,
    #[serde(rename = "VLA")]
    vla: String,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct GsProClubData {
    speed: String,
    angle_of_attack: String,
    face_to_target: String,
    lie: String,
    loft: String,
    path: String,
    speed_at_impact: String,
    vertical_face_impact: String,
    horizontal_face_impact: String,
    closure_rate: String,
}

impl Default for GsProClubData {
    fn default() -> Self {
        Self {
            speed: "0.0".into(),
            angle_of_attack: "0.0".into(),
            face_to_target: "0.0".into(),
            lie: "0.0".into(),
            loft: "0.0".into(),
            path: "0.0".into(),
            speed_at_impact: "0.0".into(),
            vertical_face_impact: "0.0".into(),
            horizontal_face_impact: "0.0".into(),
            closure_rate: "0.0".into(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct GsProShotOptions {
    contains_ball_data: bool,
    contains_club_data: bool,
    launch_monitor_is_ready: bool,
    launch_monitor_ball_detected: bool,
    is_heart_beat: bool,
}

/// Response from GSPro server
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GsProResponse {
    pub code: u32,
    pub message: String,
    pub player: Option<GsProPlayer>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GsProPlayer {
    pub handed: String, // "RH" or "LH"
    pub club: String,   // "DR", "PT", etc.
}

impl GsProInterface {
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            port: DEFAULT_PORT,
            stream: None,
            shot_number: 0,
            connected: false,
        }
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    fn build_shot_message(&self, shot: &ShotData) -> GsProMessage {
        GsProMessage {
            device_id: DEVICE_ID.into(),
            units: "Yards".into(),
            shot_number: shot.shot_number,
            api_version: API_VERSION.into(),
            ball_data: GsProBallData {
                speed: format!("{:.1}", shot.speed_mph),
                spin_axis: format!("{:.1}", shot.spin_axis_deg),
                total_spin: format!("{:.1}", shot.total_spin_rpm),
                back_spin: format!("{}", shot.backspin_rpm),
                side_spin: format!("{}", shot.sidespin_rpm),
                hla: format!("{:.1}", shot.hla_deg),
                vla: format!("{:.1}", shot.vla_deg),
            },
            club_data: GsProClubData::default(),
            shot_data_options: GsProShotOptions {
                contains_ball_data: true,
                contains_club_data: false,
                launch_monitor_is_ready: true,
                launch_monitor_ball_detected: true,
                is_heart_beat: false,
            },
        }
    }

    fn build_heartbeat_message(&self, heartbeat: &Heartbeat) -> GsProMessage {
        GsProMessage {
            device_id: DEVICE_ID.into(),
            units: "Yards".into(),
            shot_number: self.shot_number,
            api_version: API_VERSION.into(),
            ball_data: GsProBallData {
                speed: "0.0".into(),
                spin_axis: "0.0".into(),
                total_spin: "0.0".into(),
                back_spin: "0".into(),
                side_spin: "0".into(),
                hla: "0.0".into(),
                vla: "0.0".into(),
            },
            club_data: GsProClubData::default(),
            shot_data_options: GsProShotOptions {
                contains_ball_data: false,
                contains_club_data: false,
                launch_monitor_is_ready: heartbeat.ready,
                launch_monitor_ball_detected: heartbeat.ball_detected,
                is_heart_beat: true,
            },
        }
    }

    async fn send_message(&self, msg: &GsProMessage) -> Result<(), LaunchTracError> {
        let json = serde_json::to_string(msg)
            .map_err(|e| LaunchTracError::Serialization(format!("GSPro message: {e}")))?;

        tracing::debug!(bytes = json.len(), "Sending GSPro message");

        if let Some(stream) = &self.stream {
            let mut stream = stream.lock().await;
            stream
                .write_all(json.as_bytes())
                .await
                .map_err(|e| LaunchTracError::Simulator(format!("GSPro send failed: {e}")))?;
        } else {
            return Err(LaunchTracError::Simulator("Not connected to GSPro".into()));
        }

        Ok(())
    }

    /// Read and parse a response from GSPro
    pub async fn read_response(&self) -> Result<Option<GsProResponse>, LaunchTracError> {
        if let Some(stream) = &self.stream {
            let mut stream = stream.lock().await;
            let mut buf = vec![0u8; BUFFER_SIZE];

            match stream.read(&mut buf).await {
                Ok(0) => {
                    tracing::warn!("GSPro connection closed");
                    Ok(None)
                }
                Ok(n) => {
                    let text = String::from_utf8_lossy(&buf[..n]);
                    tracing::debug!(response = %text, "GSPro response");

                    let response: GsProResponse = serde_json::from_str(&text).map_err(|e| {
                        LaunchTracError::Serialization(format!("GSPro response parse: {e}"))
                    })?;

                    Ok(Some(response))
                }
                Err(e) => Err(LaunchTracError::Simulator(format!("GSPro read error: {e}"))),
            }
        } else {
            Err(LaunchTracError::Simulator("Not connected to GSPro".into()))
        }
    }
}

#[async_trait::async_trait]
impl super::SimulatorInterface for GsProInterface {
    async fn connect(&mut self) -> Result<(), LaunchTracError> {
        let addr = format!("{}:{}", self.address, self.port);
        tracing::info!(address = %addr, "Connecting to GSPro");

        let stream = TcpStream::connect(&addr)
            .await
            .map_err(|e| LaunchTracError::Simulator(format!("GSPro connect to {addr}: {e}")))?;

        self.stream = Some(Mutex::new(stream));
        self.connected = true;

        // Send initial heartbeat to establish presence
        self.send_heartbeat(&Heartbeat::default()).await?;

        tracing::info!("Connected to GSPro");
        Ok(())
    }

    async fn send_shot(&mut self, shot: &ShotData) -> Result<(), LaunchTracError> {
        self.shot_number = shot.shot_number;
        let msg = self.build_shot_message(shot);
        self.send_message(&msg).await?;

        tracing::info!(
            shot = shot.shot_number,
            speed = shot.speed_mph,
            vla = shot.vla_deg,
            hla = shot.hla_deg,
            backspin = shot.backspin_rpm,
            sidespin = shot.sidespin_rpm,
            "Shot sent to GSPro"
        );

        Ok(())
    }

    async fn send_heartbeat(&mut self, heartbeat: &Heartbeat) -> Result<(), LaunchTracError> {
        let msg = self.build_heartbeat_message(heartbeat);
        self.send_message(&msg).await
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn disconnect(&mut self) -> Result<(), LaunchTracError> {
        self.stream = None;
        self.connected = false;
        tracing::info!("Disconnected from GSPro");
        Ok(())
    }

    fn name(&self) -> &str {
        "GSPro"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use launchtrac_common::types::ClubType;
    use tokio::net::TcpListener;
    use tokio::io::AsyncBufReadExt;
    use tokio::io::BufReader;
    use std::net::SocketAddr;

    #[tokio::test]
    async fn shot_message_format_matches_gspro_spec() {
        let iface = GsProInterface::new("127.0.0.1");
        let shot = ShotData::new(
            1,
            150.0,
            12.5,
            -1.2,
            2800,
            -200,
            ClubType::Driver,
            0.95,
            250,
        );

        let msg = iface.build_shot_message(&shot);
        let json = serde_json::to_string_pretty(&msg).unwrap();

        // Verify required fields exist
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["DeviceID"], "LaunchTrac LM");
        assert_eq!(v["Units"], "Yards");
        assert_eq!(v["APIversion"], "1");
        assert_eq!(v["BallData"]["Speed"], "150.0");
        assert_eq!(v["BallData"]["BackSpin"], "2800");
        assert_eq!(v["BallData"]["SideSpin"], "-200");
        assert_eq!(v["BallData"]["VLA"], "12.5");
        assert_eq!(v["BallData"]["HLA"], "-1.2");
        assert_eq!(v["ShotDataOptions"]["ContainsBallData"], true);
        assert_eq!(v["ShotDataOptions"]["IsHeartBeat"], false);
    }

    #[test]
    fn heartbeat_message_format() {
        let iface = GsProInterface::new("127.0.0.1");
        let heartbeat = Heartbeat {
            ball_detected: true,
            ready: true,
        };

        let msg = iface.build_heartbeat_message(&heartbeat);
        let json = serde_json::to_string(&msg).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(v["ShotDataOptions"]["IsHeartBeat"], true);
        assert_eq!(v["ShotDataOptions"]["ContainsBallData"], false);
        assert_eq!(v["ShotDataOptions"]["LaunchMonitorBallDetected"], true);
        assert_eq!(v["ShotDataOptions"]["LaunchMonitorIsReady"], true);
    }

    #[tokio::test]
    async fn test_gspro_integration() -> Result<(), LaunchTracError> {
        let listener = TcpListener::bind("127.0.0.1:921").await?;
        let addr = listener.local_addr()?;

        let mut iface = GsProInterface::new(addr.to_string());
        iface.connect().await?;

        let shot = ShotData::new(
            1,
            150.0,
            12.5,
            -1.2,
            2800,
            -200,
            ClubType::Driver,
            0.95,
            250,
        );

        iface.send_shot(&shot).await?;

        let mut stream = listener.accept().await?;
        let mut reader = BufReader::new(stream.0);
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        let response: GsProResponse = serde_json::from_str(&line).unwrap();
        assert_eq!(response.code, 0);

        iface.disconnect().await?;
        Ok(())
    }
}