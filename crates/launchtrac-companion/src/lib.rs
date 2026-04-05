use launchtrac_common::error::LaunchTracError;
use launchtrac_common::shot::ShotData;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

/// Desktop companion app — bridges cloud relay to local simulator.
///
/// Architecture:
///   1. Connects outbound to Fly.io relay via WebSocket
///   2. Opens local TCP server on port 921 (GSPro) and 2483 (E6)
///   3. When a simulator connects locally, forwards shots from relay
///   4. GSPro/E6 see localhost as if LaunchTrac were on the same network
pub struct CompanionBridge {
    relay_url: String,
    device_id: String,
}

impl CompanionBridge {
    pub fn new(relay_url: impl Into<String>, device_id: impl Into<String>) -> Self {
        Self {
            relay_url: relay_url.into(),
            device_id: device_id.into(),
        }
    }

    /// Start the companion bridge
    pub async fn run(&self) -> Result<(), LaunchTracError> {
        tracing::info!(
            relay = %self.relay_url,
            device = %self.device_id,
            "Starting companion bridge"
        );

        // Start local TCP servers for simulators
        let gspro_handle = tokio::spawn(Self::run_gspro_proxy());
        let e6_handle = tokio::spawn(Self::run_e6_proxy());

        // TODO: Connect to WebSocket relay and forward messages to TCP clients

        tokio::select! {
            r = gspro_handle => tracing::info!("GSPro proxy exited: {:?}", r),
            r = e6_handle => tracing::info!("E6 proxy exited: {:?}", r),
        }

        Ok(())
    }

    async fn run_gspro_proxy() -> Result<(), LaunchTracError> {
        let listener = TcpListener::bind("127.0.0.1:921")
            .await
            .map_err(|e| LaunchTracError::Network(format!("Failed to bind GSPro port 921: {e}")))?;

        tracing::info!("GSPro proxy listening on 127.0.0.1:921");

        loop {
            let (mut socket, addr) = listener
                .accept()
                .await
                .map_err(|e| LaunchTracError::Network(format!("Accept error: {e}")))?;

            tracing::info!(%addr, "GSPro simulator connected");

            // TODO: Forward shots from relay WebSocket to this TCP socket
            // For now, just hold the connection open
            tokio::spawn(async move {
                tokio::signal::ctrl_c().await.ok();
                let _ = socket.shutdown().await;
            });
        }
    }

    async fn run_e6_proxy() -> Result<(), LaunchTracError> {
        let listener = TcpListener::bind("127.0.0.1:2483")
            .await
            .map_err(|e| LaunchTracError::Network(format!("Failed to bind E6 port 2483: {e}")))?;

        tracing::info!("E6 proxy listening on 127.0.0.1:2483");

        loop {
            let (mut socket, addr) = listener
                .accept()
                .await
                .map_err(|e| LaunchTracError::Network(format!("Accept error: {e}")))?;

            tracing::info!(%addr, "E6 simulator connected");

            tokio::spawn(async move {
                tokio::signal::ctrl_c().await.ok();
                let _ = socket.shutdown().await;
            });
        }
    }
}
