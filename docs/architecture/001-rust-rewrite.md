# ADR-001: Rewrite in Rust

## Status
Accepted

## Context
LaunchTrac v1 is a 44K-line C++ monolith with manual memory management, 280+ configuration parameters, and tight hardware coupling that makes testing impossible without physical hardware.

## Decision
Rewrite the entire system in Rust with a modular crate architecture.

## Rationale
- **Memory safety**: C++ uses raw `new`/`delete` and has no bounds checking on image buffers
- **Single binary**: Rust produces a single statically-linked binary for the Pi (no Python runtime, no Boost .so files)
- **Testability**: Trait-based hardware abstraction enables MockHardware for development without a Pi
- **Ecosystem**: `tokio` for async, `axum` for web, `serde` for JSON, `ncnn-rs` for ML inference
- **Same performance**: Zero-cost abstractions match C++ speed

## Consequences
- Learning curve for contributors who know C++ but not Rust
- Need to port proven algorithms (Gabor filter, rotation search) faithfully
- OpenCV bindings are less mature than C++ native, but sufficient
