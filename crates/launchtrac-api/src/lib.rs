// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2022-2025, Verdant Consultants, LLC. (original PiTrac code)
// Copyright (C) 2026, LaunchTrac contributors
//
// This file is part of LaunchTrac, a derivative work of PiTrac
// (https://github.com/jeshernandez/PiTrac). Both projects are licensed
// under the GNU General Public License v2.0.
//
use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Json;
use axum::routing::{get, post};
use std::sync::Arc;
use tokio::sync::broadcast;

use launchtrac_common::shot::ShotData;

/// Shared API gateway state
pub struct ApiState {
    pub shot_tx: broadcast::Sender<ShotData>,
}

/// Build the API gateway router
pub fn build_router(state: Arc<ApiState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/shots", post(receive_shot))
        .route("/ws", get(ws_upgrade))
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "launchtrac-api-gateway",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn receive_shot(
    State(state): State<Arc<ApiState>>,
    Json(shot): Json<ShotData>,
) -> Json<serde_json::Value> {
    tracing::info!(shot_id = %shot.id, "Received shot from edge device");
    let _ = state.shot_tx.send(shot);
    Json(serde_json::json!({"status": "ok"}))
}

async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ApiState>>,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: Arc<ApiState>) {
    let mut rx = state.shot_tx.subscribe();
    while let Ok(shot) = rx.recv().await {
        let json = serde_json::to_string(&shot).unwrap_or_default();
        if socket.send(Message::Text(json.into())).await.is_err() {
            break;
        }
    }
}
