// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2022-2025, Verdant Consultants, LLC. (original PiTrac code)
// Copyright (C) 2026, LaunchTrac contributors
//
// This file is part of LaunchTrac, a derivative work of PiTrac
// (https://github.com/jeshernandez/PiTrac). Both projects are licensed
// under the GNU General Public License v2.0.
//
use launchtrac_common::error::LaunchTracError;

use crate::GpioController;

/// Real GPIO controller for Raspberry Pi (uses lgpio)
///
/// Pin assignments (BCM numbering):
///   GPIO 25 (Pin 22) — Camera 2 external trigger (active HIGH)
///   GPIO 18 (Pin 12) — PWM0 for IR LED strobe
///   GPIO 19 (Pin 35) — PWM1 (spare)
pub struct PiGpio {
    // lgpio handle will go here when building on Pi
    // For now, this is a compile-target stub
}

impl PiGpio {
    pub fn new() -> Result<Self, LaunchTracError> {
        // TODO: Initialize lgpio on Pi target
        // #[cfg(target_arch = "aarch64")]
        // let handle = lgpio::gpiochip_open(0)?;
        Ok(Self {})
    }
}

impl GpioController for PiGpio {
    fn set_pin(&self, pin: u32, value: bool) -> Result<(), LaunchTracError> {
        tracing::debug!(pin, value, "GPIO set_pin");
        // TODO: lgpio::gpio_write(handle, pin, value as i32)
        Ok(())
    }

    fn read_pin(&self, pin: u32) -> Result<bool, LaunchTracError> {
        tracing::debug!(pin, "GPIO read_pin");
        // TODO: lgpio::gpio_read(handle, pin)
        Ok(false)
    }
}
