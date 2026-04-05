use launchtrac_common::error::LaunchTracError;

use crate::{PulseTiming, PwmStrobe};

/// Hardware PWM strobe controller for Raspberry Pi 5.
///
/// Replaces the old SPI bit-bang approach with native hardware PWM,
/// providing sub-microsecond jitter vs the old ~8.7µs per SPI bit.
///
/// Uses GPIO18 (PWM0) for IR LED control and GPIO25 for camera trigger.
pub struct PiPwmStrobe {
    /// GPIO pin for IR LED PWM (BCM 18)
    led_pin: u32,
    /// GPIO pin for camera external trigger (BCM 25)
    trigger_pin: u32,
}

impl PiPwmStrobe {
    pub const LED_PWM_PIN: u32 = 18;
    pub const CAMERA_TRIGGER_PIN: u32 = 25;

    pub fn new() -> Result<Self, LaunchTracError> {
        Ok(Self {
            led_pin: Self::LED_PWM_PIN,
            trigger_pin: Self::CAMERA_TRIGGER_PIN,
        })
    }

    /// Convert the LaunchTrac v1 strobe pulse vector format (inter-flash delays in ms)
    /// to our PulseTiming format.
    ///
    /// Original format: [0.7, 1.8, 3.0, 2.2, 3.0, 7.1, 4.0, 4.0, 0]
    /// where each value is the delay in ms before the next flash, 0 = end.
    pub fn from_legacy_pulse_vector(delays_ms: &[f64], pulse_width_us: u64) -> Vec<PulseTiming> {
        delays_ms
            .iter()
            .take_while(|&&d| d > 0.0)
            .map(|&delay_ms| PulseTiming {
                delay_us: (delay_ms * 1000.0) as u64,
                duration_us: pulse_width_us,
            })
            .collect()
    }
}

impl PwmStrobe for PiPwmStrobe {
    fn send_pulse_train(&self, pulses: &[PulseTiming]) -> Result<(), LaunchTracError> {
        tracing::debug!(
            pin = self.led_pin,
            pulse_count = pulses.len(),
            "Sending PWM pulse train"
        );

        // TODO: Use lgpio wave functions or direct PWM hardware
        // For each pulse:
        //   1. Wait delay_us
        //   2. Set LED pin HIGH for duration_us
        //   3. Set LED pin LOW
        //
        // On Pi 5, this can use DMA-timed GPIO for precise timing:
        //   lgpio::tx_pulse(handle, led_pin, duration_us, delay_us, ...)

        for (i, pulse) in pulses.iter().enumerate() {
            tracing::trace!(
                pulse_index = i,
                delay_us = pulse.delay_us,
                duration_us = pulse.duration_us,
                "Pulse"
            );
        }

        Ok(())
    }

    fn trigger_camera(&self, duration_us: u64) -> Result<(), LaunchTracError> {
        tracing::debug!(
            pin = self.trigger_pin,
            duration_us,
            "Camera trigger pulse"
        );
        // TODO: GPIO25 HIGH for duration_us, then LOW
        Ok(())
    }
}
