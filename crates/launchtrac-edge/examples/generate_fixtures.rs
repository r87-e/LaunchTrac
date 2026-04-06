/// Synthetic test fixture generator for LaunchTrac v2.
use std::fs;
use std::path::Path;

const WIDTH: usize = 1456;
const HEIGHT: usize = 1088;

fn main() {
    let base = Path::new("tests/fixtures");
    fs::create_dir_all(base).unwrap();

    generate_driver_shot(&base.join("driver_shot_001"));
    generate_iron7_shot(&base.join("iron7_shot_001"));

    // --- New Fixtures ---
    generate_wedge_shot(&base.join("wedge_shot_001"));
    generate_putter_shot(&base.join("putter_shot_001"));
    generate_mishit_top(&base.join("mishit_top_001"));
    generate_sky_ball(&base.join("sky_ball_001"));

    println!("\nAll fixtures generated successfully!");
}

struct FrameSpec {
    ball_cx: f64,
    ball_cy: f64,
    ball_radius: f64,
    brightness: u8,
    time_offset_us: u64,
}

/// Wedge: High launch, high spin, slower horizontal speed
fn generate_wedge_shot(dir: &Path) {
    println!("Generating wedge_shot_001...");
    fs::create_dir_all(dir).unwrap();
    let frames = vec![
        FrameSpec {
            ball_cx: 400.0,
            ball_cy: 800.0,
            ball_radius: 88.0,
            brightness: 220,
            time_offset_us: 0,
        },
        FrameSpec {
            ball_cx: 450.0,
            ball_cy: 550.0,
            ball_radius: 82.0,
            brightness: 200,
            time_offset_us: 1500,
        },
        FrameSpec {
            ball_cx: 520.0,
            ball_cy: 300.0,
            ball_radius: 75.0,
            brightness: 180,
            time_offset_us: 3500,
        },
        FrameSpec {
            ball_cx: 600.0,
            ball_cy: 100.0,
            ball_radius: 68.0,
            brightness: 160,
            time_offset_us: 6000,
        },
    ];
    write_fixture(dir, &frames, "wedge_shot");
}

/// Putter: Very low launch, low speed
fn generate_putter_shot(dir: &Path) {
    println!("Generating putter_shot_001...");
    fs::create_dir_all(dir).unwrap();
    let frames = vec![
        FrameSpec {
            ball_cx: 400.0,
            ball_cy: 850.0,
            ball_radius: 90.0,
            brightness: 210,
            time_offset_us: 0,
        },
        FrameSpec {
            ball_cx: 480.0,
            ball_cy: 845.0,
            ball_radius: 90.0,
            brightness: 210,
            time_offset_us: 2000,
        },
        FrameSpec {
            ball_cx: 580.0,
            ball_cy: 840.0,
            ball_radius: 90.0,
            brightness: 210,
            time_offset_us: 5000,
        },
        FrameSpec {
            ball_cx: 700.0,
            ball_cy: 835.0,
            ball_radius: 90.0,
            brightness: 210,
            time_offset_us: 9000,
        },
    ];
    write_fixture(dir, &frames, "putter_shot");
}

/// Mishit/Top: Ball barely visible at the bottom of the frame
fn generate_mishit_top(dir: &Path) {
    println!("Generating mishit_top_001...");
    fs::create_dir_all(dir).unwrap();
    let frames = vec![
        FrameSpec {
            ball_cx: 400.0,
            ball_cy: 1050.0,
            ball_radius: 85.0,
            brightness: 180,
            time_offset_us: 0,
        },
        FrameSpec {
            ball_cx: 550.0,
            ball_cy: 1040.0,
            ball_radius: 85.0,
            brightness: 180,
            time_offset_us: 1200,
        },
        FrameSpec {
            ball_cx: 750.0,
            ball_cy: 1030.0,
            ball_radius: 85.0,
            brightness: 180,
            time_offset_us: 3000,
        },
    ];
    write_fixture(dir, &frames, "mishit_top");
}

/// Sky Ball: Extreme vertical launch angle (almost straight up)
fn generate_sky_ball(dir: &Path) {
    println!("Generating sky_ball_001...");
    fs::create_dir_all(dir).unwrap();
    let frames = vec![
        FrameSpec {
            ball_cx: 400.0,
            ball_cy: 750.0,
            ball_radius: 85.0,
            brightness: 210,
            time_offset_us: 0,
        },
        FrameSpec {
            ball_cx: 410.0,
            ball_cy: 500.0,
            ball_radius: 80.0,
            brightness: 190,
            time_offset_us: 1000,
        },
        FrameSpec {
            ball_cx: 425.0,
            ball_cy: 200.0,
            ball_radius: 70.0,
            brightness: 170,
            time_offset_us: 2500,
        },
        FrameSpec {
            ball_cx: 435.0,
            ball_cy: -50.0,
            ball_radius: 60.0,
            brightness: 150,
            time_offset_us: 4500,
        },
    ];
    write_fixture(dir, &frames, "sky_ball");
}

// --- Keep existing Driver and Iron functions below ---

fn generate_driver_shot(dir: &Path) {
    println!("Generating driver_shot_001...");
    fs::create_dir_all(dir).unwrap();
    let frames = vec![
        FrameSpec {
            ball_cx: 400.0,
            ball_cy: 700.0,
            ball_radius: 85.0,
            brightness: 210,
            time_offset_us: 0,
        },
        FrameSpec {
            ball_cx: 520.0,
            ball_cy: 620.0,
            ball_radius: 82.0,
            brightness: 200,
            time_offset_us: 700,
        },
        FrameSpec {
            ball_cx: 680.0,
            ball_cy: 520.0,
            ball_radius: 78.0,
            brightness: 195,
            time_offset_us: 2500,
        },
        FrameSpec {
            ball_cx: 900.0,
            ball_cy: 400.0,
            ball_radius: 74.0,
            brightness: 185,
            time_offset_us: 5500,
        },
        FrameSpec {
            ball_cx: 1100.0,
            ball_cy: 310.0,
            ball_radius: 70.0,
            brightness: 175,
            time_offset_us: 7700,
        },
    ];
    write_fixture(dir, &frames, "driver_shot");
}

fn generate_iron7_shot(dir: &Path) {
    println!("Generating iron7_shot_001...");
    fs::create_dir_all(dir).unwrap();
    let frames = vec![
        FrameSpec {
            ball_cx: 500.0,
            ball_cy: 750.0,
            ball_radius: 86.0,
            brightness: 215,
            time_offset_us: 0,
        },
        FrameSpec {
            ball_cx: 580.0,
            ball_cy: 630.0,
            ball_radius: 83.0,
            brightness: 205,
            time_offset_us: 900,
        },
        FrameSpec {
            ball_cx: 680.0,
            ball_cy: 490.0,
            ball_radius: 79.0,
            brightness: 195,
            time_offset_us: 2700,
        },
        FrameSpec {
            ball_cx: 800.0,
            ball_cy: 340.0,
            ball_radius: 75.0,
            brightness: 185,
            time_offset_us: 5700,
        },
    ];
    write_fixture(dir, &frames, "iron7_shot");
}

fn write_fixture(dir: &Path, frames: &[FrameSpec], name: &str) {
    let meta = serde_json::json!({
        "name": name,
        "width": WIDTH,
        "height": HEIGHT,
        "frame_count": frames.len(),
        "frames": frames.iter().enumerate().map(|(i, f)| serde_json::json!({
            "index": i,
            "ball_cx": f.ball_cx,
            "ball_cy": f.ball_cy,
            "ball_radius": f.ball_radius,
            "time_offset_us": f.time_offset_us
        })).collect::<Vec<_>>()
    });
    fs::write(
        dir.join("metadata.json"),
        serde_json::to_string_pretty(&meta).unwrap(),
    )
    .unwrap();

    for (i, spec) in frames.iter().enumerate() {
        let image = render_frame(spec);
        let path = dir.join(format!("frame_{:03}.raw", i));
        fs::write(&path, &image).unwrap();
    }
}

fn render_frame(spec: &FrameSpec) -> Vec<u8> {
    let mut data = vec![15u8; WIDTH * HEIGHT];
    for y in (HEIGHT * 3 / 4)..HEIGHT {
        for x in 0..WIDTH {
            data[y * WIDTH + x] = 25 + ((x * 7 + y * 13) % 15) as u8;
        }
    }
    let (cx, cy, r) = (spec.ball_cx, spec.ball_cy, spec.ball_radius);
    let y0 = (cy - r * 1.3).max(0.0) as usize;
    let y1 = (cy + r * 1.3).min(HEIGHT as f64) as usize;
    let x0 = (cx - r * 1.3).max(0.0) as usize;
    let x1 = (cx + r * 1.3).min(WIDTH as f64) as usize;

    for y in y0..y1 {
        for x in x0..x1 {
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < r {
                let n = dist / r;
                let base = spec.brightness as f64 * (1.0 - 0.35 * n * n);
                let dimple = ((x as f64 * 0.8).sin() * (y as f64 * 0.8).cos() * 6.0) as f64;
                data[y * WIDTH + x] = (base + dimple).clamp(0.0, 255.0) as u8;
            } else if dist < r * 1.1 {
                let edge = 1.0 - (dist - r) / (r * 0.1);
                let val = (spec.brightness as f64 * 0.25 * edge).clamp(0.0, 255.0) as u8;
                data[y * WIDTH + x] = data[y * WIDTH + x].max(val);
            }
        }
    }
    // Specular IR hotspot
    let hx = cx - r * 0.2;
    let hy = cy - r * 0.3;
    let hr = r * 0.18;
    let hy0 = (hy - hr).max(0.0) as usize;
    let hy1 = (hy + hr).min(HEIGHT as f64) as usize;
    let hx0 = (hx - hr).max(0.0) as usize;
    let hx1 = (hx + hr).min(WIDTH as f64) as usize;
    for y in hy0..hy1 {
        for x in hx0..hx1 {
            let d = (((x as f64 - hx).powi(2)) + ((y as f64 - hy).powi(2))).sqrt();
            if d < hr {
                let f = 1.0 - d / hr;
                data[y * WIDTH + x] = data[y * WIDTH + x].saturating_add((60.0 * f * f) as u8);
            }
        }
    }
    data
}
