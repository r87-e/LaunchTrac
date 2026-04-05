# ADR-003: Actor-Based Pipeline Architecture

## Status
Accepted

## Context
LaunchTrac v1 uses a monolithic finite state machine (gs_fsm.cpp) with 6 states and an event queue, where all processing happens in a single thread with global static state.

## Decision
Decompose the FSM into 7 independent actors communicating via typed tokio::mpsc channels.

## Actors
1. **Camera1** - Tee watcher (high-FPS preview)
2. **MotionDetector** - Frame differencing to detect ball movement
3. **StrobeController** - IR LED pulse train + Camera 2 trigger
4. **ImageProcessor** - Vision pipeline (YOLO detection + spin estimation)
5. **ResultsRouter** - Fan-out to simulators, web, cloud
6. **CloudUploader** - Async upload to Fly.io
7. **WebServer** - Embedded Axum serving dashboard + WebSocket

## Rationale
- **Zero shared mutable state**: Each actor owns its data
- **Natural parallelism**: Camera capture and image processing run concurrently
- **Testability**: Each actor can be unit tested with mock inputs
- **Failure isolation**: One actor crashing doesn't bring down the system

## Consequences
- Message passing adds small latency vs direct function calls (~microseconds)
- Debugging message flow requires structured logging (tracing crate)
- Must carefully design message types to avoid excessive cloning
