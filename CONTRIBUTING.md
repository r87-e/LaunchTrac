# Contributing to LaunchTrac

Thanks for your interest in LaunchTrac! Here's how to get started.

## Prerequisites

- Rust 1.82+ (`rustup update`)
- Node.js 22+ (for the dashboard)

## Getting Started

```bash
git clone https://github.com/r87-e/LaunchTrac.git
cd LaunchTrac

# Build everything
cargo build

# Run all tests (no hardware needed)
cargo test --workspace

# Run the edge binary in mock mode (simulated cameras)
cargo run -p launchtrac-edge -- --mock --port 8080

# Generate synthetic ball image fixtures
cargo run -p launchtrac-edge --example generate_fixtures

# Process a fixture through the full vision pipeline
cargo run -p launchtrac-edge --example process_fixture -- tests/fixtures/driver_shot_001
```

## Dashboard

```bash
cd dashboard
npm install
npm run dev
```

Opens a dev server at http://localhost:5173.

## Before Submitting a PR

```bash
cargo fmt --all          # Format code
cargo clippy --workspace # Check for lint issues
cargo test --workspace   # Run all tests
```

CI runs these checks automatically on every pull request.

## Where to Help

Check the [issues](https://github.com/r87-e/LaunchTrac/issues) page for open tasks. Good areas to contribute:

- **Vision pipeline** — improve ball detection accuracy, edge cases
- **Spin estimation** — train and improve the ML spin model
- **Simulator protocols** — test with GSPro, E6, TruGolf
- **Dashboard** — improve the SvelteKit web UI
- **Hardware abstraction** — add support for new cameras or SBCs
- **Documentation** — setup guides, wiring diagrams, troubleshooting

## Project Structure

```
LaunchTrac/
  crates/              # Rust workspace (11 crates)
  dashboard/           # SvelteKit web UI
  hardware/            # BOM, wiring diagrams
  docs/                # Architecture decisions
  tests/               # Integration tests + image fixtures
```

See the [README](README.md) for the full crate map.

## Code Style

- Run `cargo fmt` before committing
- Follow existing patterns in the codebase
- Keep PRs focused — one feature or fix per PR

## License

By contributing, you agree that your contributions will be licensed under GPL-2.0.
