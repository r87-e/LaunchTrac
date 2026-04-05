use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::LaunchTracError;

/// LaunchTrac configuration — ~30 user-facing parameters instead of 280+.
///
/// Loaded from layered TOML:
///   1. /etc/launchtrac/defaults.toml (shipped with binary)
///   2. ~/.launchtrac/config.toml (user overrides)
///   3. ~/.launchtrac/calibration.toml (auto-generated)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub hardware: HardwareConfig,
    pub cameras: CameraConfig,
    pub network: NetworkConfig,
    pub preferences: PreferencesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HardwareConfig {
    /// Enclosure version (2 or 3)
    pub enclosure_version: u8,

    /// LED driver type: "ldd-700h" (default) or "custom-pcb" (legacy)
    pub led_driver: String,

    /// Number of IR LEDs in series
    pub led_count: u8,
}

impl Default for HardwareConfig {
    fn default() -> Self {
        Self {
            enclosure_version: 2,
            led_driver: "ldd-700h".to_string(),
            led_count: 6,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CameraConfig {
    /// Camera 1 (tee watcher) position relative to ball, in cm
    pub cam1_offset_x_cm: f64,
    pub cam1_offset_y_cm: f64,
    pub cam1_offset_z_cm: f64,

    /// Camera 2 (flight capture) position relative to ball, in cm
    pub cam2_offset_x_cm: f64,
    pub cam2_offset_y_cm: f64,
    pub cam2_offset_z_cm: f64,

    /// Lens type: "6mm" or "3.6mm"
    pub lens_type: String,

    /// Camera model: "imx296" (default) or "pigs"
    pub camera_model: String,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            cam1_offset_x_cm: 0.0,
            cam1_offset_y_cm: 30.0,
            cam1_offset_z_cm: 0.0,
            cam2_offset_x_cm: 20.0,
            cam2_offset_y_cm: 30.0,
            cam2_offset_z_cm: 0.0,
            lens_type: "6mm".to_string(),
            camera_model: "imx296".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    /// GSPro simulator IP address (empty = disabled)
    pub gspro_address: String,

    /// E6/TruGolf simulator IP address (empty = disabled)
    pub e6_address: String,

    /// Cloud API token (empty = offline mode)
    pub cloud_token: String,

    /// Local web UI port
    pub web_port: u16,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            gspro_address: String::new(),
            e6_address: String::new(),
            cloud_token: String::new(),
            web_port: 8080,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PreferencesConfig {
    /// Measurement units: "imperial" or "metric"
    pub units: String,

    /// Enable putting mode (slower ball speeds, longer strobe intervals)
    pub putting_mode: bool,

    /// Log level: "error", "warn", "info", "debug", "trace"
    pub log_level: String,

    /// Save debug images for each shot
    pub save_debug_images: bool,

    /// Player handedness: "right" or "left"
    pub handedness: String,
}

impl Default for PreferencesConfig {
    fn default() -> Self {
        Self {
            units: "imperial".to_string(),
            putting_mode: false,
            log_level: "info".to_string(),
            save_debug_images: false,
            handedness: "right".to_string(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hardware: HardwareConfig::default(),
            cameras: CameraConfig::default(),
            network: NetworkConfig::default(),
            preferences: PreferencesConfig::default(),
        }
    }
}

impl Config {
    /// Load config from layered TOML files.
    /// Defaults → user overrides → calibration data.
    pub fn load() -> Result<Self, LaunchTracError> {
        let mut config = Config::default();

        // Layer 1: System defaults
        let defaults_path = PathBuf::from("/etc/launchtrac/defaults.toml");
        if defaults_path.exists() {
            config.merge_from_file(&defaults_path)?;
        }

        // Layer 2: User overrides
        let user_config = Self::user_config_path();
        if user_config.exists() {
            config.merge_from_file(&user_config)?;
        }

        // Layer 3: Auto-calibration data
        let calibration = Self::calibration_path();
        if calibration.exists() {
            config.merge_from_file(&calibration)?;
        }

        Ok(config)
    }

    /// Save user config to ~/.launchtrac/config.toml
    pub fn save_user_config(&self) -> Result<(), LaunchTracError> {
        let path = Self::user_config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| LaunchTracError::Config(format!("Failed to serialize config: {e}")))?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    fn merge_from_file(&mut self, path: &Path) -> Result<(), LaunchTracError> {
        let content = std::fs::read_to_string(path)?;
        let overrides: Config = toml::from_str(&content)
            .map_err(|e| LaunchTracError::Config(format!("Failed to parse {}: {e}", path.display())))?;
        // For now, full replacement per section if present in file.
        // A more sophisticated merge could be added later.
        *self = overrides;
        Ok(())
    }

    fn user_config_path() -> PathBuf {
        dirs_or_default("config.toml")
    }

    fn calibration_path() -> PathBuf {
        dirs_or_default("calibration.toml")
    }
}

fn dirs_or_default(filename: &str) -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        home.join(".launchtrac").join(filename)
    } else {
        PathBuf::from(format!("/tmp/launchtrac/{filename}"))
    }
}
