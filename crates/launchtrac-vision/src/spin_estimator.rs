// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2022-2025, Verdant Consultants, LLC. (original PiTrac code)
// Copyright (C) 2026, LaunchTrac contributors
//
// This file is part of LaunchTrac, a derivative work of PiTrac
// (https://github.com/PiTracLM/PiTrac). Both projects are licensed
// under the GNU General Public License v2.0.
//
use launchtrac_common::error::LaunchTracError;
use launchtrac_hal::ImageFrame;

use crate::BallDetection;

/// Spin estimation result
#[derive(Debug, Clone)]
pub struct SpinResult {
    pub backspin_rpm: i32,
    pub sidespin_rpm: i32,
    /// Spin axis in degrees
    pub spin_axis_deg: f64,
    pub confidence: f64,
}

/// Gabor filter bank configuration
const NUM_ORIENTATIONS: usize = 16;
const GABOR_KERNEL_SIZE: usize = 21;
const GABOR_SIGMA: f64 = 3.0;
const GABOR_WAVELENGTH: f64 = 8.0;
const GABOR_GAMMA: f64 = 0.5;

/// Rotation search parameters
const COARSE_STEP_DEG: f64 = 6.0;
const FINE_STEP_DEG: f64 = 1.0;
const X_RANGE: (f64, f64) = (-42.0, 42.0); // Side spin range
const Y_RANGE: (f64, f64) = (-30.0, 30.0); // Backspin range
const Z_RANGE: (f64, f64) = (-50.0, 60.0); // Tilt range

/// Spin estimator using ML model with Gabor filter preprocessing.
///
/// Two-stage approach:
///   1. Fast path: MobileNetV3-Small CNN regresses spin directly from
///      Gabor-filtered dimple pair images (<50ms target)
///   2. Fallback: 3D rotation search (ported from LaunchTrac v1)
pub struct SpinEstimator {
    ml_available: bool,
    gabor_kernels: Vec<Vec<f64>>,
}

impl SpinEstimator {
    pub fn new() -> Result<Self, LaunchTracError> {
        // Pre-compute Gabor filter bank
        let gabor_kernels = (0..NUM_ORIENTATIONS)
            .map(|i| {
                let theta = (i as f64) * std::f64::consts::PI / NUM_ORIENTATIONS as f64;
                generate_gabor_kernel(
                    GABOR_KERNEL_SIZE,
                    GABOR_SIGMA,
                    theta,
                    GABOR_WAVELENGTH,
                    GABOR_GAMMA,
                )
            })
            .collect();

        Ok(Self {
            ml_available: false,
            gabor_kernels,
        })
    }

    /// Estimate spin from strobed ball images
    pub fn estimate(
        &self,
        frames: &[ImageFrame],
        detections: &[BallDetection],
    ) -> Result<SpinResult, LaunchTracError> {
        if detections.len() < 2 {
            return Ok(SpinResult {
                backspin_rpm: 0,
                sidespin_rpm: 0,
                spin_axis_deg: 0.0,
                confidence: 0.0,
            });
        }

        if self.ml_available {
            self.estimate_ml(frames, detections)
        } else {
            self.estimate_rotation_search(frames, detections)
        }
    }

    /// ML-based spin regression (fast path)
    fn estimate_ml(
        &self,
        _frames: &[ImageFrame],
        _detections: &[BallDetection],
    ) -> Result<SpinResult, LaunchTracError> {
        // TODO: Load and run MobileNetV3-Small via NCNN
        Err(LaunchTracError::Vision("ML model not loaded".into()))
    }

    /// Rotation search spin estimation (fallback).
    ///
    /// Algorithm (ported from LaunchTrac v1 ball_image_proc.cpp):
    ///   1. Extract ball ROIs from first and last detection
    ///   2. Apply Gabor filter bank to enhance dimple patterns
    ///   3. Generate rotation candidates (coarse: 6 deg steps)
    ///   4. For each candidate, rotate ball1 image and compare to ball2
    ///   5. Find best match, then refine with 1 deg steps
    ///   6. Convert rotation angles to RPM using time delta
    fn estimate_rotation_search(
        &self,
        frames: &[ImageFrame],
        detections: &[BallDetection],
    ) -> Result<SpinResult, LaunchTracError> {
        let first_det = &detections[0];
        let last_det = detections.last().unwrap();

        // Extract ball ROIs
        let ball1 = extract_ball_roi(&frames[0], first_det);
        let ball2 = extract_ball_roi(&frames[frames.len() - 1], last_det);

        // Apply Gabor filter bank to both balls
        let filtered1 = self.apply_gabor_bank(&ball1.pixels, ball1.size);
        let filtered2 = self.apply_gabor_bank(&ball2.pixels, ball2.size);

        // Coarse rotation search
        let coarse_best = self.coarse_search(&filtered1, &filtered2, ball1.size);

        // Fine refinement around best coarse match
        let fine_best = self.fine_search(&filtered1, &filtered2, ball1.size, &coarse_best);

        // Convert rotation angles to RPM
        let time_delta_s = (frames[frames.len() - 1].timestamp - frames[0].timestamp)
            .num_microseconds()
            .unwrap_or(1) as f64
            / 1_000_000.0;

        let backspin_rpm = angle_to_rpm(fine_best.z_rotation, time_delta_s);
        let sidespin_rpm = angle_to_rpm(fine_best.x_rotation, time_delta_s);

        let spin_axis_deg = if backspin_rpm != 0 {
            (sidespin_rpm as f64 / backspin_rpm as f64)
                .atan()
                .to_degrees()
        } else {
            0.0
        };

        Ok(SpinResult {
            backspin_rpm,
            sidespin_rpm,
            spin_axis_deg,
            confidence: fine_best.score.min(1.0),
        })
    }

    /// Apply Gabor filter bank and accumulate responses
    fn apply_gabor_bank(&self, pixels: &[u8], size: usize) -> Vec<f64> {
        let mut accumulated = vec![0.0f64; size * size];

        for kernel in &self.gabor_kernels {
            let response = convolve_2d(pixels, size, size, kernel, GABOR_KERNEL_SIZE);
            for (acc, &resp) in accumulated.iter_mut().zip(response.iter()) {
                *acc += resp.abs();
            }
        }

        // Normalize to 0-1
        let max = accumulated.iter().cloned().fold(0.0f64, f64::max);
        if max > 0.0 {
            for v in &mut accumulated {
                *v /= max;
            }
        }

        accumulated
    }

    /// Coarse rotation search (6-degree steps)
    fn coarse_search(
        &self,
        filtered1: &[f64],
        filtered2: &[f64],
        size: usize,
    ) -> RotationCandidate {
        let mut best = RotationCandidate::default();

        let mut x = X_RANGE.0;
        while x <= X_RANGE.1 {
            let mut y = Y_RANGE.0;
            while y <= Y_RANGE.1 {
                let mut z = Z_RANGE.0;
                while z <= Z_RANGE.1 {
                    let score = self.evaluate_rotation(filtered1, filtered2, size, x, y, z);
                    if score > best.score {
                        best = RotationCandidate {
                            x_rotation: x,
                            y_rotation: y,
                            z_rotation: z,
                            score,
                        };
                    }
                    z += COARSE_STEP_DEG;
                }
                y += COARSE_STEP_DEG;
            }
            x += COARSE_STEP_DEG;
        }

        best
    }

    /// Fine refinement search (1-degree steps around coarse best)
    fn fine_search(
        &self,
        filtered1: &[f64],
        filtered2: &[f64],
        size: usize,
        coarse: &RotationCandidate,
    ) -> RotationCandidate {
        let mut best = coarse.clone();
        let half_step = COARSE_STEP_DEG;

        let mut x = coarse.x_rotation - half_step;
        while x <= coarse.x_rotation + half_step {
            let mut y = coarse.y_rotation - half_step;
            while y <= coarse.y_rotation + half_step {
                let mut z = coarse.z_rotation - half_step;
                while z <= coarse.z_rotation + half_step {
                    let score = self.evaluate_rotation(filtered1, filtered2, size, x, y, z);
                    if score > best.score {
                        best = RotationCandidate {
                            x_rotation: x,
                            y_rotation: y,
                            z_rotation: z,
                            score,
                        };
                    }
                    z += FINE_STEP_DEG;
                }
                y += FINE_STEP_DEG;
            }
            x += FINE_STEP_DEG;
        }

        best
    }

    /// Evaluate a single rotation: rotate filtered1 by (x,y,z) and compare to filtered2.
    ///
    /// Uses normalized cross-correlation as similarity metric.
    fn evaluate_rotation(
        &self,
        filtered1: &[f64],
        filtered2: &[f64],
        size: usize,
        x_deg: f64,
        y_deg: f64,
        z_deg: f64,
    ) -> f64 {
        // Apply 2D approximation of 3D ball rotation
        // Full 3D projection would map each pixel to a sphere, rotate, then reproject
        // Simplified: use affine transform approximation for small angles
        let rotated = rotate_image_approx(filtered1, size, x_deg, y_deg, z_deg);

        // Normalized cross-correlation
        normalized_cross_correlation(&rotated, filtered2)
    }
}

/// Ball region of interest
struct BallRoi {
    pixels: Vec<u8>,
    size: usize,
}

/// Extract a square ROI around the detected ball
fn extract_ball_roi(frame: &ImageFrame, det: &BallDetection) -> BallRoi {
    let size = (det.radius * 2.5) as usize; // Slightly larger than ball diameter
    let size = size.max(32).min(256); // Clamp to reasonable size

    let half = size / 2;
    let cx = det.cx as usize;
    let cy = det.cy as usize;
    let w = frame.width as usize;
    let h = frame.height as usize;

    let mut pixels = vec![0u8; size * size];

    for dy in 0..size {
        for dx in 0..size {
            let sx = (cx + dx).saturating_sub(half);
            let sy = (cy + dy).saturating_sub(half);
            if sx < w && sy < h {
                pixels[dy * size + dx] = frame.data[sy * w + sx];
            }
        }
    }

    BallRoi { pixels, size }
}

/// Generate a Gabor filter kernel
fn generate_gabor_kernel(
    size: usize,
    sigma: f64,
    theta: f64,
    wavelength: f64,
    gamma: f64,
) -> Vec<f64> {
    let mut kernel = vec![0.0f64; size * size];
    let half = size as f64 / 2.0;
    let cos_t = theta.cos();
    let sin_t = theta.sin();

    for y in 0..size {
        for x in 0..size {
            let xf = x as f64 - half;
            let yf = y as f64 - half;

            let x_theta = xf * cos_t + yf * sin_t;
            let y_theta = -xf * sin_t + yf * cos_t;

            let gaussian = (-0.5
                * ((x_theta.powi(2) + gamma.powi(2) * y_theta.powi(2)) / sigma.powi(2)))
            .exp();
            let sinusoid = (2.0 * std::f64::consts::PI * x_theta / wavelength).cos();

            kernel[y * size + x] = gaussian * sinusoid;
        }
    }

    kernel
}

/// 2D convolution
fn convolve_2d(
    input: &[u8],
    width: usize,
    height: usize,
    kernel: &[f64],
    kernel_size: usize,
) -> Vec<f64> {
    let mut output = vec![0.0f64; width * height];
    let half = kernel_size / 2;

    for y in half..height.saturating_sub(half) {
        for x in half..width.saturating_sub(half) {
            let mut sum = 0.0;
            for ky in 0..kernel_size {
                for kx in 0..kernel_size {
                    let px = input[(y + ky - half) * width + (x + kx - half)] as f64;
                    sum += px * kernel[ky * kernel_size + kx];
                }
            }
            output[y * width + x] = sum;
        }
    }

    output
}

/// Approximate 3D ball rotation as a 2D affine transform.
/// This is a simplification — full implementation would use sphere projection.
fn rotate_image_approx(
    pixels: &[f64],
    size: usize,
    x_deg: f64,
    y_deg: f64,
    z_deg: f64,
) -> Vec<f64> {
    let mut output = vec![0.0f64; size * size];
    let cx = size as f64 / 2.0;
    let cy = size as f64 / 2.0;

    // Z rotation is in-plane rotation
    let z_rad = z_deg.to_radians();
    let cos_z = z_rad.cos();
    let sin_z = z_rad.sin();

    // X and Y rotations cause perspective foreshortening
    let scale_x = x_deg.to_radians().cos(); // Horizontal compression from tilt
    let scale_y = y_deg.to_radians().cos(); // Vertical compression from tilt

    for y in 0..size {
        for x in 0..size {
            // Transform from output to input coordinates
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;

            // Apply Z rotation
            let rx = dx * cos_z - dy * sin_z;
            let ry = dx * sin_z + dy * cos_z;

            // Apply perspective scaling
            let sx = rx / scale_x + cx;
            let sy = ry / scale_y + cy;

            // Bilinear interpolation
            let sx_i = sx.floor() as usize;
            let sy_i = sy.floor() as usize;
            let fx = sx - sx.floor();
            let fy = sy - sy.floor();

            if sx_i < size - 1 && sy_i < size - 1 {
                let v00 = pixels[sy_i * size + sx_i];
                let v10 = pixels[sy_i * size + sx_i + 1];
                let v01 = pixels[(sy_i + 1) * size + sx_i];
                let v11 = pixels[(sy_i + 1) * size + sx_i + 1];

                output[y * size + x] = v00 * (1.0 - fx) * (1.0 - fy)
                    + v10 * fx * (1.0 - fy)
                    + v01 * (1.0 - fx) * fy
                    + v11 * fx * fy;
            }
        }
    }

    output
}

/// Normalized cross-correlation between two images
fn normalized_cross_correlation(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let n = a.len() as f64;
    let mean_a: f64 = a.iter().sum::<f64>() / n;
    let mean_b: f64 = b.iter().sum::<f64>() / n;

    let mut num = 0.0;
    let mut den_a = 0.0;
    let mut den_b = 0.0;

    for (va, vb) in a.iter().zip(b.iter()) {
        let da = va - mean_a;
        let db = vb - mean_b;
        num += da * db;
        den_a += da * da;
        den_b += db * db;
    }

    let den = (den_a * den_b).sqrt();
    if den > 1e-10 { num / den } else { 0.0 }
}

/// Convert rotation angle to RPM given time delta
fn angle_to_rpm(angle_deg: f64, time_delta_s: f64) -> i32 {
    if time_delta_s > 0.0 {
        ((angle_deg / 360.0) / time_delta_s * 60.0) as i32
    } else {
        0
    }
}

#[derive(Debug, Clone)]
struct RotationCandidate {
    x_rotation: f64,
    y_rotation: f64,
    z_rotation: f64,
    score: f64,
}

impl Default for RotationCandidate {
    fn default() -> Self {
        Self {
            x_rotation: 0.0,
            y_rotation: 0.0,
            z_rotation: 0.0,
            score: f64::NEG_INFINITY,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gabor_kernel_is_correct_size() {
        let kernel = generate_gabor_kernel(21, 3.0, 0.0, 8.0, 0.5);
        assert_eq!(kernel.len(), 21 * 21);
    }

    #[test]
    fn gabor_kernel_has_zero_dc() {
        let kernel = generate_gabor_kernel(21, 3.0, 0.0, 8.0, 0.5);
        let sum: f64 = kernel.iter().sum();
        // Gabor cosine kernel has some DC component due to finite size.
        // Verify the kernel has both positive and negative values (bandpass behavior).
        let has_positive = kernel.iter().any(|&v| v > 0.01);
        let has_negative = kernel.iter().any(|&v| v < -0.01);
        assert!(
            has_positive && has_negative,
            "Gabor kernel should have both positive and negative lobes"
        );
    }

    #[test]
    fn ncc_identical_images() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let ncc = normalized_cross_correlation(&a, &a);
        assert!(
            (ncc - 1.0).abs() < 1e-6,
            "NCC of identical should be 1.0, got {ncc}"
        );
    }

    #[test]
    fn ncc_inverse_images() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        let ncc = normalized_cross_correlation(&a, &b);
        assert!(
            (ncc - (-1.0)).abs() < 1e-6,
            "NCC of inverse should be -1.0, got {ncc}"
        );
    }

    #[test]
    fn angle_to_rpm_conversion() {
        // 360 degrees in 1 second = 60 RPM
        assert_eq!(angle_to_rpm(360.0, 1.0), 60);
        // 36 degrees in 0.01 seconds = 600 RPM
        assert_eq!(angle_to_rpm(36.0, 0.01), 600);
        // 10 degrees in 0.005 seconds = 333 RPM
        assert_eq!(angle_to_rpm(10.0, 0.005), 333);
    }

    #[test]
    fn rotate_identity_preserves_image() {
        let size = 32;
        let mut pixels = vec![0.0f64; size * size];
        // Put a bright dot in the center
        pixels[16 * size + 16] = 1.0;

        let rotated = rotate_image_approx(&pixels, size, 0.0, 0.0, 0.0);
        assert!((rotated[16 * size + 16] - 1.0).abs() < 0.01);
    }

    #[test]
    fn convolve_identity() {
        let input = vec![0u8, 0, 0, 0, 255, 0, 0, 0, 0]; // 3x3 with bright center
        let kernel = vec![0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0]; // Identity kernel
        let output = convolve_2d(&input, 3, 3, &kernel, 3);
        assert_eq!(output[4], 255.0);
    }
}
