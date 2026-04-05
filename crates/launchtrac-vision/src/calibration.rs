use launchtrac_common::error::LaunchTracError;

/// Camera calibration data for converting pixel coordinates to real-world 3D.
///
/// Auto-calibration approach:
///   1. Ship pre-computed intrinsic matrices for supported lenses (6mm, 3.6mm)
///   2. Extrinsic calibration: user places golf ball (known diameter 42.67mm)
///      at marked position, presses button, system auto-detects and computes transform
///   3. No checkerboard needed
#[derive(Debug, Clone)]
pub struct CameraCalibration {
    /// 3x3 intrinsic camera matrix (flattened row-major)
    pub intrinsic_matrix: [f64; 9],

    /// Distortion coefficients (k1, k2, p1, p2, k3)
    pub distortion_coeffs: [f64; 5],

    /// Focal length in mm
    pub focal_length_mm: f64,

    /// Expected ball radius in pixels at 40cm distance
    pub ball_radius_at_40cm: f64,
}

impl CameraCalibration {
    /// Pre-computed calibration for InnoMaker IMX296 + 6mm M12 lens
    pub fn imx296_6mm() -> Self {
        Self {
            intrinsic_matrix: [
                1200.0, 0.0, 728.0, // fx, 0, cx
                0.0, 1200.0, 544.0, // 0, fy, cy
                0.0, 0.0, 1.0, // 0, 0, 1
            ],
            distortion_coeffs: [0.0, 0.0, 0.0, 0.0, 0.0], // Will be refined per-unit
            focal_length_mm: 6.0,
            ball_radius_at_40cm: 87.0,
        }
    }

    /// Pre-computed calibration for InnoMaker IMX296 + 3.6mm M12 lens
    pub fn imx296_3_6mm() -> Self {
        Self {
            intrinsic_matrix: [720.0, 0.0, 728.0, 0.0, 720.0, 544.0, 0.0, 0.0, 1.0],
            distortion_coeffs: [0.0, 0.0, 0.0, 0.0, 0.0],
            focal_length_mm: 3.6,
            ball_radius_at_40cm: 45.0,
        }
    }

    /// Run auto-calibration using a golf ball at known position
    pub fn auto_calibrate(
        &mut self,
        _ball_center_px: (f64, f64),
        _ball_radius_px: f64,
        _known_distance_cm: f64,
    ) -> Result<(), LaunchTracError> {
        // TODO: Using known ball diameter (42.67mm) and detected radius,
        // refine focal length and distortion coefficients
        tracing::info!("Auto-calibration not yet implemented");
        Ok(())
    }

    /// Convert pixel coordinates to real-world 3D position (meters)
    pub fn pixel_to_world(&self, _px: f64, _py: f64, _radius_px: f64) -> (f64, f64, f64) {
        // TODO: Use intrinsic matrix + known ball size to triangulate
        // distance = (focal_length * real_diameter) / (pixel_diameter * sensor_pixel_size)
        (0.0, 0.0, 0.0)
    }
}
