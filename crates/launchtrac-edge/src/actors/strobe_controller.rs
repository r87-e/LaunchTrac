use launchtrac_common::config::Config;
use launchtrac_hal::mock::MockHardware;
use launchtrac_hal::{CameraCapture, ImageFrame, PulseTiming, PwmStrobe};
use tokio::sync::mpsc;

/// Default strobe pulse vector for driver shots (inter-flash delays in ms).
/// Ported from LaunchTrac v1 kStrobePulseVectorDriver.
const DRIVER_PULSE_DELAYS_MS: &[f64] = &[0.7, 1.8, 3.0, 2.2, 3.0, 7.1, 4.0, 4.0];

/// Default strobe pulse vector for putting.
const PUTTER_PULSE_DELAYS_MS: &[f64] = &[
    2.5, 5.0, 8.0, 10.5, 8.5, 21.0, 21.0, 21.0, 21.0, 21.0, 21.0, 21.0,
];

/// Pulse width in microseconds (each IR flash duration)
const PULSE_WIDTH_US: u64 = 20;

/// Camera shutter hold time in microseconds
const SHUTTER_DURATION_US: u64 = 100_000; // 100ms

/// Strobe controller actor — orchestrates IR LED strobing and Camera 2 capture.
///
/// When triggered by motion detection:
///   1. Arm Camera 2 external trigger (GPIO25)
///   2. Send IR LED pulse train via hardware PWM
///   3. Camera 2 captures strobed ball images
///   4. Forward captured frames to the image processor
pub async fn run(
    config: Config,
    mock: bool,
    mut motion_rx: mpsc::Receiver<()>,
    strobed_tx: mpsc::Sender<Vec<ImageFrame>>,
) -> anyhow::Result<()> {
    tracing::info!("Strobe controller actor starting");

    let putting_mode = config.preferences.putting_mode;

    // Select pulse vector based on mode
    let pulse_delays = if putting_mode {
        PUTTER_PULSE_DELAYS_MS
    } else {
        DRIVER_PULSE_DELAYS_MS
    };

    let pulses: Vec<PulseTiming> = pulse_delays
        .iter()
        .map(|&delay_ms| PulseTiming {
            delay_us: (delay_ms * 1000.0) as u64,
            duration_us: PULSE_WIDTH_US,
        })
        .collect();

    // Initialize hardware
    let (strobe, mut camera2): (Box<dyn PwmStrobe>, Box<dyn CameraCapture>) = if mock {
        let mock_strobe = MockHardware::new();
        let mock_cam = MockHardware::new();
        (Box::new(mock_strobe), Box::new(mock_cam))
    } else {
        // TODO: Real hardware
        let mock_strobe = MockHardware::new();
        let mock_cam = MockHardware::new();
        (Box::new(mock_strobe), Box::new(mock_cam))
    };

    while let Some(()) = motion_rx.recv().await {
        tracing::info!("Motion trigger received — starting strobe sequence");

        // 1. Trigger camera shutter open
        if let Err(e) = strobe.trigger_camera(SHUTTER_DURATION_US) {
            tracing::error!("Camera trigger failed: {e}");
            continue;
        }

        // 2. Send IR pulse train (concurrent with open shutter)
        if let Err(e) = strobe.send_pulse_train(&pulses) {
            tracing::error!("Strobe pulse train failed: {e}");
            continue;
        }

        // 3. Capture strobed frames from Camera 2
        match camera2.capture_strobed() {
            Ok(frames) => {
                tracing::info!(frame_count = frames.len(), "Strobed capture complete");
                if strobed_tx.send(frames).await.is_err() {
                    break;
                }
            }
            Err(e) => {
                tracing::error!("Strobed capture failed: {e}");
            }
        }
    }

    tracing::info!("Strobe controller actor shutting down");
    Ok(())
}
