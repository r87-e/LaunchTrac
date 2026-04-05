use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Json;
use axum::routing::get;
use axum::Router;
use launchtrac_common::config::Config;
use launchtrac_common::shot::ShotData;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Shared state for the web server
struct AppState {
    config: Config,
    last_shot: RwLock<Option<ShotData>>,
    shot_tx: broadcast::Sender<ShotData>,
}

/// Start the embedded web server (Axum).
///
/// Provides:
///   - GET /api/health — system status
///   - GET /api/shot — last shot data
///   - GET /api/config — current configuration
///   - WS  /ws — real-time shot streaming
pub async fn start_server(
    config: Config,
    mut shot_rx: broadcast::Receiver<ShotData>,
) -> anyhow::Result<()> {
    let port = config.network.web_port;
    let (shot_tx, _) = broadcast::channel(32);

    let state = Arc::new(AppState {
        config: config.clone(),
        last_shot: RwLock::new(None),
        shot_tx: shot_tx.clone(),
    });

    // Background task: receive shots and update state
    let state_clone = state.clone();
    tokio::spawn(async move {
        while let Ok(shot) = shot_rx.recv().await {
            *state_clone.last_shot.write().await = Some(shot.clone());
            let _ = state_clone.shot_tx.send(shot);
        }
    });

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/shot", get(get_shot))
        .route("/api/config", get(get_config))
        .route("/ws", get(ws_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    tracing::info!(port, "Web server listening");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "name": "LaunchTrac v2"
    }))
}

async fn get_shot(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let shot = state.last_shot.read().await;
    match &*shot {
        Some(s) => Json(serde_json::to_value(s).unwrap_or_default()),
        None => Json(serde_json::json!({"message": "No shots yet"})),
    }
}

async fn get_config(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(serde_json::to_value(&state.config).unwrap_or_default())
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: Arc<AppState>) {
    tracing::info!("WebSocket client connected");

    let mut rx = state.shot_tx.subscribe();

    while let Ok(shot) = rx.recv().await {
        let json = serde_json::to_string(&shot).unwrap_or_default();
        if socket.send(Message::Text(json.into())).await.is_err() {
            break;
        }
    }

    tracing::info!("WebSocket client disconnected");
}
