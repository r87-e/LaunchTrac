use launchtrac_hal::mock::MockHardware;
use launchtrac_hal::{CameraCapture, ImageFrame};
use tokio::sync::mpsc;

/// Camera 1 actor — watches the tee for ball presence and motion.
///
/// In real mode: uses libcamera in high-FPS cropped video mode.
/// In mock mode: replays frames from a fixture directory.
pub async fn run(
    mock: bool,
    fixture_path: Option<String>,
    frame_tx: mpsc::Sender<ImageFrame>,
) -> anyhow::Result<()> {
    tracing::info!(mock, "Camera1 actor starting");

    let mut camera: Box<dyn CameraCapture> = if mock {
        let hw = if let Some(ref path) = fixture_path {
            MockHardware::from_fixture(path)?
        } else {
            MockHardware::new()
        };
        Box::new(hw)
    } else {
        // TODO: Real camera initialization
        // Box::new(LibCamera::new(0)?)
        Box::new(MockHardware::new()) // Fallback to mock for now
    };

    camera.start_preview()?;

    loop {
        match camera.capture_still() {
            Ok(frame) => {
                if frame_tx.send(frame).await.is_err() {
                    tracing::info!("Frame receiver dropped, shutting down Camera1");
                    break;
                }
            }
            Err(e) => {
                tracing::warn!("Camera1 capture error: {e}");
            }
        }

        // In real mode, this runs at camera FPS.
        // In mock mode, simulate ~30fps.
        if mock {
            tokio::time::sleep(tokio::time::Duration::from_millis(33)).await;
        }
    }

    camera.stop()?;
    Ok(())
}
