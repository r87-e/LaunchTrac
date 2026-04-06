// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2022-2025, Verdant Consultants, LLC. (original PiTrac code)
// Copyright (C) 2026, LaunchTrac contributors
//
// This file is part of LaunchTrac, a derivative work of PiTrac
// (https://github.com/jeshernandez/PiTrac). Both projects are licensed
// under the GNU General Public License v2.0.
//
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

/// Safety limits for IR LED strobe operation.
///
/// These limits prevent hardware damage (burned MOSFETs, overheated gate drivers)
/// and reduce IR eye safety risk. Derived from PiTrac community field experience.
pub struct StrobeSafetyLimits {
    /// Maximum pulse duration in microseconds (default: 20μs)
    pub max_pulse_us: u64,
    /// Minimum delay between pulses in microseconds (default: 500μs)
    pub min_delay_us: u64,
    /// Maximum number of pulses per burst (default: 10)
    pub max_pulses_per_burst: usize,
    /// Minimum cooldown between bursts in milliseconds (default: 100ms)
    pub min_burst_cooldown_ms: u64,
    /// Maximum duty cycle as a fraction (default: 0.02 = 2%)
    pub max_duty_cycle: f64,
}

impl Default for StrobeSafetyLimits {
    fn default() -> Self {
        Self {
            max_pulse_us: 20,
            min_delay_us: 500,
            max_pulses_per_burst: 10,
            min_burst_cooldown_ms: 100,
            max_duty_cycle: 0.02,
        }
    }
}

impl StrobeSafetyLimits {
    /// Validate a pulse train against safety limits.
    /// Returns an error describing the violation if any limit is exceeded.
    pub fn validate(&self, pulses: &[PulseTiming]) -> Result<(), LaunchTracError> {
        if pulses.len() > self.max_pulses_per_burst {
            return Err(LaunchTracError::Hardware(format!(
                "Pulse train has {} pulses, max allowed is {}",
                pulses.len(),
                self.max_pulses_per_burst
            )));
        }

        let mut total_on_us: u64 = 0;
        let mut total_period_us: u64 = 0;

        for (i, pulse) in pulses.iter().enumerate() {
            if pulse.duration_us > self.max_pulse_us {
                return Err(LaunchTracError::Hardware(format!(
                    "Pulse {} duration {}μs exceeds max {}μs",
                    i, pulse.duration_us, self.max_pulse_us
                )));
            }
            if pulse.delay_us < self.min_delay_us && i > 0 {
                return Err(LaunchTracError::Hardware(format!(
                    "Pulse {} delay {}μs below min {}μs",
                    i, pulse.delay_us, self.min_delay_us
                )));
            }
            total_on_us += pulse.duration_us;
            total_period_us += pulse.delay_us + pulse.duration_us;
        }

        if total_period_us > 0 {
            let duty_cycle = total_on_us as f64 / total_period_us as f64;
            if duty_cycle > self.max_duty_cycle {
                return Err(LaunchTracError::Hardware(format!(
                    "Duty cycle {:.1}% exceeds max {:.1}%",
                    duty_cycle * 100.0,
                    self.max_duty_cycle * 100.0
                )));
            }
        }

        Ok(())
    }
}

/// PWM strobe control interface
pub trait PwmStrobe: Send + Sync {
    /// Send a sequence of timed pulses to the IR LEDs.
    /// Each PulseTiming specifies delay-before and pulse-duration.
    ///
    /// Implementations MUST validate pulses against StrobeSafetyLimits
    /// before firing. Continuous or unvalidated strobe operation can
    /// burn MOSFETs and gate drivers.
    fn send_pulse_train(&self, pulses: &[PulseTiming]) -> Result<(), LaunchTracError>;

    /// Send the camera external trigger signal
    fn trigger_camera(&self, duration_us: u64) -> Result<(), LaunchTracError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_pulse_train_passes() {
        let limits = StrobeSafetyLimits::default();
        let pulses = vec![
            PulseTiming {
                delay_us: 700,
                duration_us: 10,
            },
            PulseTiming {
                delay_us: 1800,
                duration_us: 10,
            },
            PulseTiming {
                delay_us: 1800,
                duration_us: 10,
            },
        ];
        assert!(limits.validate(&pulses).is_ok());
    }

    #[test]
    fn rejects_pulse_too_long() {
        let limits = StrobeSafetyLimits::default();
        let pulses = vec![PulseTiming {
            delay_us: 700,
            duration_us: 50,
        }];
        let err = limits.validate(&pulses).unwrap_err();
        assert!(err.to_string().contains("exceeds max"));
    }

    #[test]
    fn rejects_too_many_pulses() {
        let limits = StrobeSafetyLimits::default();
        let pulses: Vec<PulseTiming> = (0..15)
            .map(|_| PulseTiming {
                delay_us: 1000,
                duration_us: 10,
            })
            .collect();
        let err = limits.validate(&pulses).unwrap_err();
        assert!(err.to_string().contains("max allowed"));
    }

    #[test]
    fn rejects_delay_too_short() {
        let limits = StrobeSafetyLimits::default();
        let pulses = vec![
            PulseTiming {
                delay_us: 700,
                duration_us: 10,
            },
            PulseTiming {
                delay_us: 100,
                duration_us: 10,
            },
        ];
        let err = limits.validate(&pulses).unwrap_err();
        assert!(err.to_string().contains("below min"));
    }

    #[test]
    fn rejects_excessive_duty_cycle() {
        let limits = StrobeSafetyLimits::default();
        let pulses = vec![
            PulseTiming {
                delay_us: 500,
                duration_us: 20,
            },
            PulseTiming {
                delay_us: 500,
                duration_us: 20,
            },
        ];
        // duty = 40/1040 = 3.8%, exceeds 2% default
        let err = limits.validate(&pulses).unwrap_err();
        assert!(err.to_string().contains("Duty cycle"));
    }

    #[test]
    fn empty_pulse_train_passes() {
        let limits = StrobeSafetyLimits::default();
        assert!(limits.validate(&[]).is_ok());
    }
}
