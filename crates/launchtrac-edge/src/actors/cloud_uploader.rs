/// Cloud uploader actor -- sends shot data to LaunchTrac cloud for storage and analytics.
///
/// Features:
///   - Buffered uploads (batch shots when offline)
///   - Automatic reconnection
///   - Optional raw image upload for community ML training
///
/// Communicates with the Fly.io Shot Service via HTTPS + WebSocket.
pub async fn run(_cloud_token: String) -> anyhow::Result<()> {
    tracing::info!("Cloud uploader actor starting (not yet implemented)");

    // TODO: Implementation phases:
    // 1. Establish WebSocket connection to wss://api.launchtrac.dev/ws
    // 2. Authenticate with device token
    // 3. Listen for shot data on channel
    // 4. Serialize and send each shot
    // 5. Buffer locally if offline, replay when reconnected
    // 6. Optionally upload raw images if user opted in

    // For now, this actor is a no-op
    tokio::signal::ctrl_c().await?;

    tracing::info!("Cloud uploader actor shutting down");
    Ok(())
}
