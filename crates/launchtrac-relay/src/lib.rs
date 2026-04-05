use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Json;
use axum::routing::get;
use axum::Router;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Simulator relay service.
///
/// Bridges LaunchTrac edge devices and simulator PCs when they're on different networks.
///
/// Flow:
///   1. Edge device connects via WebSocket, identifies with device_id
///   2. Companion app connects via WebSocket, identifies with same device_id
///   3. Relay forwards shot data from edge to companion
///   4. Companion translates WebSocket -> TCP for GSPro/E6
pub struct RelayState {
    /// Active device channels: device_id -> broadcast sender
    channels: RwLock<HashMap<String, broadcast::Sender<String>>>,
}

impl RelayState {
    pub fn new() -> Self {
        Self {
            channels: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for RelayState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn build_router(state: Arc<RelayState>) -> Router {
    Router::new()
        .route("/relay/health", get(health))
        .route("/relay/device/{device_id}", get(device_ws))
        .route("/relay/companion/{device_id}", get(companion_ws))
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"service": "launchtrac-relay", "status": "ok"}))
}

async fn device_ws(
    ws: WebSocketUpgrade,
    axum::extract::Path(device_id): axum::extract::Path<String>,
    State(state): State<Arc<RelayState>>,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| handle_device(socket, device_id, state))
}

async fn companion_ws(
    ws: WebSocketUpgrade,
    axum::extract::Path(device_id): axum::extract::Path<String>,
    State(state): State<Arc<RelayState>>,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| handle_companion(socket, device_id, state))
}

/// Handle edge device connection: receives shots and broadcasts to channel
async fn handle_device(mut socket: WebSocket, device_id: String, state: Arc<RelayState>) {
    tracing::info!(device_id = %device_id, "Edge device connected to relay");

    // Create or get channel for this device
    let tx = {
        let mut channels = state.channels.write().await;
        channels
            .entry(device_id.clone())
            .or_insert_with(|| broadcast::channel(64).0)
            .clone()
    };

    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(text) = msg {
            let _ = tx.send(text.to_string());
        }
    }

    tracing::info!(device_id = %device_id, "Edge device disconnected from relay");
}

/// Handle companion app connection: subscribes to device channel and forwards shots
async fn handle_companion(mut socket: WebSocket, device_id: String, state: Arc<RelayState>) {
    tracing::info!(device_id = %device_id, "Companion app connected to relay");

    let rx = {
        let channels = state.channels.read().await;
        channels.get(&device_id).map(|tx| tx.subscribe())
    };

    if let Some(mut rx) = rx {
        while let Ok(msg) = rx.recv().await {
            if socket.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    } else {
        let _ = socket
            .send(Message::Text(
                r#"{"error": "No device found with that ID"}"#.into(),
            ))
            .await;
    }

    tracing::info!(device_id = %device_id, "Companion app disconnected from relay");
}
