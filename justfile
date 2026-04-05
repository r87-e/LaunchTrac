# LaunchTrac v2 task runner

# Build all crates
build:
    cargo build

# Build release
release:
    cargo build --release

# Run all tests
test:
    cargo test --workspace

# Run clippy lints
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Format code
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Run edge in mock mode (replays test fixtures)
run-mock *ARGS:
    cargo run -p launchtrac-edge -- --mock {{ARGS}}

# Run edge with specific fixture
run-fixture FIXTURE:
    cargo run -p launchtrac-edge -- --mock --fixture {{FIXTURE}}

# Start cloud services locally
cloud-up:
    docker compose up -d

# Stop cloud services
cloud-down:
    docker compose down

# Start dashboard dev server
dashboard:
    cd dashboard && npm run dev

# Run security audit
audit:
    cargo audit

# Generate SBOM
sbom:
    cargo sbom > sbom.json

# Cross-compile for Raspberry Pi 5 (aarch64)
build-pi:
    cross build --release --target aarch64-unknown-linux-gnu -p launchtrac-edge
