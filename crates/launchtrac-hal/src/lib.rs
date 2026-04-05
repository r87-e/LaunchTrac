pub mod camera;
pub mod gpio;
pub mod mock;
pub mod pwm;

use launchtrac_common::error::LaunchTracError;

/// Timing for a single strobe pulse
#[derive(Debug, Clone)]
pub struct PulseTiming {
    /// Delay before this pulse, in microseconds
    pub delay_us: u64,
    /// Pulse duration, in microseconds
    pub duration_us: u64,
}

/// Raw image frame from a camera
#[derive(Debug, Clone)]
pub struct ImageFrame {
    /// Raw pixel data (grayscale, 8-bit)
    pub data: Vec<u8>,
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// Timestamp when frame was captured
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Frame sequence number
    pub sequence: u64,
}

/// Camera capture interface
pub trait CameraCapture: Send + Sync {
    /// Start continuous capture (high-FPS motion detection mode)
    fn start_preview(&mut self) -> Result<(), LaunchTracError>;

    /// Capture a single still frame
    fn capture_still(&mut self) -> Result<ImageFrame, LaunchTracError>;

    /// Capture a strobed sequence (external trigger + IR flash)
    fn capture_strobed(&mut self) -> Result<Vec<ImageFrame>, LaunchTracError>;

    /// Stop capture
    fn stop(&mut self) -> Result<(), LaunchTracError>;
}

/// GPIO control interface
pub trait GpioController: Send + Sync {
    /// Set a GPIO pin high or low
    fn set_pin(&self, pin: u32, value: bool) -> Result<(), LaunchTracError>;

    /// Read a GPIO pin value
    fn read_pin(&self, pin: u32) -> Result<bool, LaunchTracError>;
}

/// PWM strobe control interface
pub trait PwmStrobe: Send + Sync {
    /// Send a sequence of timed pulses to the IR LEDs.
    /// Each PulseTiming specifies delay-before and pulse-duration.
    fn send_pulse_train(&self, pulses: &[PulseTiming]) -> Result<(), LaunchTracError>;

    /// Send the camera external trigger signal
    fn trigger_camera(&self, duration_us: u64) -> Result<(), LaunchTracError>;
}
