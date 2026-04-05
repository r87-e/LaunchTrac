/// Synthetic test fixture generator for LaunchTrac v2.
///
/// Generates grayscale image sequences simulating a golf ball
/// flying through the camera's field of view under IR strobe.
///
/// Run: cargo run -p launchtrac-edge --example generate_fixtures
use std::fs;
use std::path::Path;

const WIDTH: usize = 1456;
const HEIGHT: usize = 1088;

fn main() {
    let base = Path::new("tests/fixtures");
    fs::create_dir_all(base).unwrap();

    generate_driver_shot(&base.join("driver_shot_001"));
    generate_iron7_shot(&base.join("iron7_shot_001"));
    generate_wedge_shot(&base.join("wedge_shot_001"));
    generate_putter_shot(&base.join("putter_shot_001"));
    generate_mishit_shot(&base.join("mishit_shot_001"));
    generate_skyball_shot(&base.join("skyball_shot_001"));

    println!("\nAll fixtures generated!");
}

struct FrameSpec {
    ball_cx: f64,
    ball_cy: f64,
    ball_radius: f64,
    brightness: u8,
    time_offset_us: u64,
}

/// Driver shot: ~150mph, ~12deg VLA, slight draw
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

/// Iron 7: ~120mph, ~20deg VLA, straighter
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

fn generate_wedge_shot(dir: &Path) {
    println!("Generating wedge_shot_001...");
    fs::create_dir_all(dir).unwrap();

    let frames = vec![
        FrameSpec {
            ball_cx: 500.0,
            ball_cy: 800.0,
            ball_radius: 88.0,
            brightness: 220,
            time_offset_us: 0,
        },
        FrameSpec {
            ball_cx: 550.0,
            ball_cy: 650.0,
            ball_radius: 85.0,
            brightness: 210,
            time_offset_us: 800,
        },
        FrameSpec {
            ball_cx: 650.0,
            ball_cy: 480.0,
            ball_radius: 82.0,
            brightness: 200,
            time_offset_us: 2600,
        },
    ];

    write_fixture(dir, &frames, "wedge_shot");
}

fn generate_putter_shot(dir: &Path) {
    println!("Generating putter_shot_001...");
    fs::create_dir_all(dir).unwrap();

    let frames = vec![
        FrameSpec {
            ball_cx: 450.0,
            ball_cy: 780.0,
            ball_radius: 43.0,
            brightness: 100,
            time_offset_us: 0,
        },
        FrameSpec {
            ball_cx: 460.0,
            ball_cy: 760.0,
            ball_radius: 42.0,
            brightness: 95,
            time_offset_us: 100,
        },
        FrameSpec {
            ball_cx: 470.0,
            ball_cy: 740.0,
            ball_radius: 41.0,
            brightness: 90,
            time_offset_us: 200,
        },
    ];

    write_fixture(dir, &frames, "putter_shot");
}

fn generate_mishit_shot(dir: &Path) {
    println!("Generating mishit_shot_001...");
    fs::create_dir_all(dir).unwrap();

    let frames = vec![
        FrameSpec {
            ball_cx: 300.0,
            ball_cy: 750.0,
            ball_radius: 30.0,
            brightness: 50,
            time_offset_us: 0,
        },
        FrameSpec {
            ball_cx: 310.0,
            ball_cy: 730.0,
            ball_radius: 31.0,
            brightness: 55,
            time_offset_us: 100,
        },
    ];

    write_fixture(dir, &frames, "mishit_shot");
}

fn generate_skyball_shot(dir: &Path) {
    println!("Generating skyball_shot_001...");
    fs::create_dir_all(dir).unwrap();

    let frames = vec![
        FrameSpec {
            ball_cx: 500.0,
            ball_cy: 900.0,
            ball_radius: 90.0,
            brightness: 230,
            time_offset_us: 0,
        },
        FrameSpec {
            ball_cx: 520.0,
            ball_cy: 800.0,
            ball_radius: 88.0,
            brightness: 220,
            time_offset_us: 700,
        },
        FrameSpec {
            ball_cx: 550.0,
            ball_cy: 680.0,
            ball_radius: 85.0,
            brightness: 210,
            time_offset_us: 2500,
        },
    ];

    write_fixture(dir, &frames, "skyball_shot");
}

fn write_fixture(dir: &Path, frames: &[FrameSpec], name: &str) {
    // Metadata JSON
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
        println!(
            "  frame_{:03}.raw  ball@({:.0},{:.0}) r={:.0}  [{:.1} MB]",
            i,
            spec.ball_cx,
            spec.ball_cy,
            spec.ball_radius,
            image.len() as f64 / 1_000_000.0
        );
    }
}

fn render_frame(spec: &FrameSpec) -> Vec<u8> {
    let mut data = vec![15u8; WIDTH * HEIGHT];

    // Hitting mat (bottom quarter - brighter, textured)
    for y in (HEIGHT * 3 / 4)..HEIGHT {
        for x in 0..WIDTH {
            data[y * WIDTH + x] = 25 + ((x * 7 + y * 13) % 15) as u8;
        }
    }

    let (cx, cy, r) = (spec.ball_cx, spec.ball_cy, spec.ball_radius);

    // Ball body: Gaussian brightness profile
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
                // Dimple-like texture
                let dimple = ((x as f64 * 0.8).sin() * (y as f64 * 0.8).cos() * 6.0) as f64;
                data[y * WIDTH + x] = (base + dimple).clamp(0.0, 255.0) as u8;
            } else if dist < r * 1.1 {
                // Soft edge
                let edge = 1.0 - (dist - r) / (r * 0.1);
                let val = (spec.brightness as f64 * 0.25 * edge).clamp(0.0, 255.0) as u8;
                data[y * WIDTH + x] = data[y * WIDTH + x].max(val);
            }
        }
    }

    // Specular IR hotspot (top-left of ball)
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