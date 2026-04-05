use axum::extract::State;
use axum::response::Json;
use axum::routing::post;
use axum::Router;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Simple device registry (will use Postgres + JWT in production)
pub struct AuthState {
    devices: RwLock<Vec<DeviceRegistration>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRegistration {
    pub device_id: String,
    pub api_key: String,
    pub hardware_version: String,
    pub firmware_version: String,
    pub registered_at: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub device_id: String,
    pub hardware_version: String,
    pub firmware_version: String,
}

impl AuthState {
    pub fn new() -> Self {
        Self {
            devices: RwLock::new(Vec::new()),
        }
    }
}

impl Default for AuthState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn build_router(state: Arc<AuthState>) -> Router {
    Router::new()
        .route("/api/v1/devices/register", post(register_device))
        .with_state(state)
}

async fn register_device(
    State(state): State<Arc<AuthState>>,
    Json(req): Json<RegisterRequest>,
) -> Json<serde_json::Value> {
    let api_key = format!("ptk_{}", uuid::Uuid::new_v4().to_string().replace('-', ""));

    let registration = DeviceRegistration {
        device_id: req.device_id.clone(),
        api_key: api_key.clone(),
        hardware_version: req.hardware_version,
        firmware_version: req.firmware_version,
        registered_at: chrono::Utc::now().to_rfc3339(),
    };

    state.devices.write().await.push(registration);

    tracing::info!(device_id = %req.device_id, "Device registered");

    Json(serde_json::json!({
        "device_id": req.device_id,
        "api_key": api_key,
        "status": "registered"
    }))
}
