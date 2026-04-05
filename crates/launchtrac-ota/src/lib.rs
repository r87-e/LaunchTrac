use axum::Router;
use axum::extract::{Path, State};
use axum::response::Json;
use axum::routing::get;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// OTA update service state
pub struct OtaState {
    pub releases: Vec<FirmwareRelease>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareRelease {
    pub version: String,
    pub channel: String, // "stable", "beta", "nightly"
    pub checksum_sha256: String,
    pub download_url: String,
    pub size_bytes: u64,
    pub release_notes: String,
    pub released_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCheckParams {
    pub current_version: String,
    pub channel: String,
}

impl OtaState {
    pub fn new() -> Self {
        Self {
            releases: vec![FirmwareRelease {
                version: "0.1.0".into(),
                channel: "stable".into(),
                checksum_sha256: "placeholder".into(),
                download_url: "https://releases.launchtrac.dev/v0.1.0/launchtrac-aarch64".into(),
                size_bytes: 15_000_000,
                release_notes: "Initial release".into(),
                released_at: "2026-04-05T00:00:00Z".into(),
            }],
        }
    }
}

impl Default for OtaState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn build_router(state: Arc<OtaState>) -> Router {
    Router::new()
        .route("/api/v1/ota/check/{channel}", get(check_update))
        .route("/api/v1/ota/releases", get(list_releases))
        .with_state(state)
}

async fn check_update(
    State(state): State<Arc<OtaState>>,
    Path(channel): Path<String>,
) -> Json<serde_json::Value> {
    let latest = state
        .releases
        .iter()
        .filter(|r| r.channel == channel)
        .last();

    match latest {
        Some(release) => Json(serde_json::json!({
            "update_available": true,
            "version": release.version,
            "download_url": release.download_url,
            "checksum_sha256": release.checksum_sha256,
            "size_bytes": release.size_bytes,
            "release_notes": release.release_notes
        })),
        None => Json(serde_json::json!({
            "update_available": false
        })),
    }
}

async fn list_releases(State(state): State<Arc<OtaState>>) -> Json<Vec<FirmwareRelease>> {
    Json(state.releases.clone())
}
