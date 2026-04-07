# LaunchTrac

An open-source DIY golf launch monitor that measures ball speed, launch angles, and spin — for under $200 in parts.

Built in Rust. No custom PCB. Cloud-connected. Compatible with GSPro and E6/TruGolf simulators.

## What It Does

LaunchTrac uses two global shutter cameras and IR LED strobing to capture a golf ball in flight. It measures:

- **Ball speed** (mph)
- **Vertical launch angle** (degrees)
- **Horizontal launch angle** (degrees)
- **Backspin** (RPM)
- **Sidespin** (RPM)

Results stream in real-time to golf simulators (GSPro, E6 Connect, TruGolf) and a web dashboard.

## Quick Start (No Hardware Needed)

You can run the full pipeline on your laptop using synthetic test data:

```bash
# Clone and build
git clone https://github.com/r87-e/LaunchTrac.git
cd LaunchTrac
cargo build

# Run all tests (33 tests, full vision pipeline)
cargo test --workspace

# Generate synthetic ball image fixtures
cargo run -p launchtrac-edge --example generate_fixtures

# Process a fixture through the full pipeline
cargo run -p launchtrac-edge --example process_fixture -- tests/fixtures/driver_shot_001

# Start the edge binary with web UI on :8080
cargo run -p launchtrac-edge -- --mock --port 8080
```

## Architecture

```
┌─ Raspberry Pi 5 ──────────────────────────────────────────┐
│                                                            │
│  Camera1 → MotionDetector → StrobeController               │
│                                    ↓                       │
│                             ImageProcessor                  │
│                           (YOLO + Spin ML)                  │
│                                    ↓                       │
│                             ResultsRouter ──→ GSPro/E6     │
│                                    ↓                       │
│                             WebServer (:8080)              │
│                                    ↓                       │
│                             CloudUploader ──→ Fly.io       │
└────────────────────────────────────────────────────────────┘
```

Seven actors, zero shared mutable state, typed message channels.

### Crate Map

| Crate | Purpose |
|-------|---------|
| `launchtrac-common` | Shared types, config (30 params vs PiTrac v1's 280+), errors |
| `launchtrac-hal` | Hardware abstraction — GPIO, PWM, cameras + MockHardware for testing |
| `launchtrac-vision` | Ball detection (YOLO/Hough), trajectory math, Gabor filter, spin estimation |
| `launchtrac-sim-proto` | GSPro (port 921) and E6/TruGolf (port 2483) protocol implementations |
| `launchtrac-edge` | Main binary — actor pipeline, embedded web server, CLI |
| `launchtrac-companion` | Desktop app — bridges cloud relay to local simulator TCP ports |
| `launchtrac-api` | Cloud API gateway with WebSocket streaming |
| `launchtrac-shot` | Shot storage and analytics service |
| `launchtrac-auth` | Device registration and authentication |
| `launchtrac-ota` | Over-the-air firmware updates |
| `launchtrac-relay` | WebSocket bridge for remote simulator connections |

## Hardware (~$195, No Custom PCB)

| Component | Price |
|-----------|-------|
| Raspberry Pi 5 (8GB) | $80 |
| 2x InnoMaker IMX296 Global Shutter Camera | $60 |
| 2x M12 6mm Lens | $10 |
| Meanwell LDD-700H LED Driver | $8 |
| 6x IR LEDs (880nm) in series | $12 |
| 24V DC Power Supply | $15 |
| Wiring + connectors | $10 |
| 3D printed enclosure | Self-printed |

**No soldering a custom PCB.** The Meanwell LDD-700H is a constant-current LED driver that accepts 3.3V PWM directly from the Pi's GPIO. It replaces the entire custom connector board from PiTrac v1.

See [hardware/bom/](hardware/bom/) for the full bill of materials with wiring diagram.

## Simulator Compatibility

| Simulator | Protocol | Port | Status |
|-----------|----------|------|--------|
| GSPro | TCP JSON | 921 | Working |
| E6 Connect | TCP JSON (3-step) | 2483 | Working |
| TruGolf | TCP JSON (3-step) | 2483 | Working |

## Dashboard

SvelteKit + Tailwind web dashboard with:
- Live session view with real-time shot data via WebSocket
- Shot history with sorting and filtering
- Analytics: dispersion plot, speed/spin/angle distributions
- Device setup wizard (connect, calibrate, configure simulators)

```bash
cd dashboard
npm install
npm run dev
```

## Cloud Services (Fly.io)

Optional cloud backend for:
- Shot history and analytics across sessions
- OTA firmware updates (delta updates, signed binaries)
- Simulator relay (Pi and simulator on different networks)
- Community ML model improvement (opt-in image sharing)

```bash
# Local development
docker compose up -d  # Postgres + NATS
```

## Development

### Prerequisites

- Rust 1.82+ (`rustup update`)
- Node.js 22+ (for dashboard)
- Docker (for cloud services local dev)

### Commands

```bash
just build          # Build all crates
just test           # Run all tests
just lint           # Clippy + format check
just run-mock       # Run edge in mock mode
just dashboard      # Start dashboard dev server
just cloud-up       # Start local Postgres + NATS
just build-pi       # Cross-compile for Raspberry Pi 5
```

### Testing Without Hardware

The `MockHardware` system replays captured image sequences through the full pipeline. Generate synthetic fixtures or use real images captured from a PiTrac v1:

```bash
cargo run -p launchtrac-edge --example generate_fixtures
cargo run -p launchtrac-edge --example process_fixture -- tests/fixtures/driver_shot_001
```

### Project Structure

```
LaunchTrac/
  crates/              # Rust workspace (11 crates)
  dashboard/           # SvelteKit web UI
  ml/                  # ML model training + exported models
  hardware/            # BOM, wiring diagrams, 3D parts
  docs/                # Architecture decisions, guides
  tests/               # Integration tests + image fixtures
  sql/                 # Database schema
  .github/workflows/   # CI/CD pipeline
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

**Ways to help right now:**
- Test the pipeline with real IR ball images from PiTrac v1 hardware
- Improve ball detection accuracy on edge cases
- Train the ML spin estimation model
- Build out the cloud services
- Improve the dashboard

## Status

This is an early-stage rewrite. The pipeline works end-to-end on synthetic data. Real hardware integration is next.

- [x] Rust workspace + crate architecture
- [x] Ball detection (Hough fallback, YOLO stub)
- [x] Trajectory calculation (3D pinhole model)
- [x] Spin estimation (Gabor filter + rotation search)
- [x] GSPro protocol (port 921)
- [x] E6/TruGolf protocol (port 2483)
- [x] MockHardware + synthetic fixtures
- [x] Web dashboard (SvelteKit)
- [x] Cloud service stubs (API, shots, auth, OTA, relay)
- [x] CI/CD pipeline (GitHub Actions)
- [ ] YOLO ball detector (NCNN model)
- [ ] ML spin estimator (MobileNetV3)
- [ ] Real camera integration (libcamera-rs)
- [ ] Hardware PWM strobe (lgpio)
- [ ] Auto-calibration
- [ ] Cloud deployment (Fly.io)
- [ ] Desktop companion app
- [ ] OTA update pipeline

## Acknowledgments

Inspired by [PiTrac](https://github.com/PiTracLM/PiTrac), the world's first fully open-source DIY golf launch monitor.

## Attribution

LaunchTrac is a derivative work of [PiTrac](https://github.com/PiTracLM/PiTrac), the world's first fully open-source DIY golf launch monitor, created by James Pilgrim and contributors at Verdant Consultants, LLC. See [NOTICE.md](NOTICE.md) for details.

## License

Copyright (C) 2022-2025, Verdant Consultants, LLC. (original PiTrac code)
Copyright (C) 2026, LaunchTrac contributors

Licensed under the GNU General Public License v2.0 (GPL-2.0-only). See [LICENSE](LICENSE).
