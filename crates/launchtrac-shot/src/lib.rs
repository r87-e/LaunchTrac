use axum::Router;
use axum::extract::State;
use axum::response::Json;
use axum::routing::{get, post};
use std::sync::Arc;
use tokio::sync::RwLock;

use launchtrac_common::shot::ShotData;

/// In-memory shot store (will be replaced with Postgres)
pub struct ShotStore {
    shots: RwLock<Vec<ShotData>>,
}

impl ShotStore {
    pub fn new() -> Self {
        Self {
            shots: RwLock::new(Vec::new()),
        }
    }
}

impl Default for ShotStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Build the shot service router
pub fn build_router(store: Arc<ShotStore>) -> Router {
    Router::new()
        .route("/api/v1/shots", get(list_shots).post(create_shot))
        .route("/api/v1/shots/latest", get(latest_shot))
        .route("/api/v1/sessions/stats", get(session_stats))
        .with_state(store)
}

async fn list_shots(State(store): State<Arc<ShotStore>>) -> Json<serde_json::Value> {
    let shots = store.shots.read().await;
    let recent: Vec<_> = shots.iter().rev().take(100).collect();
    Json(serde_json::to_value(&recent).unwrap_or_default())
}

async fn create_shot(
    State(store): State<Arc<ShotStore>>,
    Json(shot): Json<ShotData>,
) -> Json<serde_json::Value> {
    let id = shot.id;
    store.shots.write().await.push(shot);
    Json(serde_json::json!({"id": id, "status": "created"}))
}

async fn latest_shot(State(store): State<Arc<ShotStore>>) -> Json<serde_json::Value> {
    let shots = store.shots.read().await;
    match shots.last() {
        Some(s) => Json(serde_json::to_value(s).unwrap_or_default()),
        None => Json(serde_json::json!({"message": "No shots"})),
    }
}

async fn session_stats(State(store): State<Arc<ShotStore>>) -> Json<serde_json::Value> {
    let shots = store.shots.read().await;
    if shots.is_empty() {
        return Json(serde_json::json!({"shot_count": 0}));
    }

    let count = shots.len() as f64;
    let avg_speed = shots.iter().map(|s| s.speed_mph).sum::<f64>() / count;
    let avg_vla = shots.iter().map(|s| s.vla_deg).sum::<f64>() / count;
    let avg_backspin = shots.iter().map(|s| s.backspin_rpm as f64).sum::<f64>() / count;

    Json(serde_json::json!({
        "shot_count": shots.len(),
        "avg_speed_mph": (avg_speed * 10.0).round() / 10.0,
        "avg_vla_deg": (avg_vla * 10.0).round() / 10.0,
        "avg_backspin_rpm": avg_backspin.round() as i32
    }))
}
