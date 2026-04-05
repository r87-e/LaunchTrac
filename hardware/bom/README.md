# LaunchTrac Bill of Materials

## Recommended Build (~$195 USD, no custom PCB)

### Compute

| Item | Qty | Est. Price | Notes |
|------|-----|-----------|-------|
| Raspberry Pi 5 (8GB) | 1 | $80 | 4GB works but 8GB recommended |
| MicroSD Card (64GB) | 1 | $10 | Class 10 / A2 recommended |
| Pi 5 Active Cooler | 1 | $5 | Required for sustained processing |
| USB-C PD Power Supply (27W) | 1 | $12 | Official Pi 5 PSU recommended |

### Cameras & Optics

| Item | Qty | Est. Price | Notes |
|------|-----|-----------|-------|
| InnoMaker IMX296 Global Shutter Camera | 2 | $30 ea ($60) | Monochrome, CSI-2 interface |
| M12 6mm Lens | 2 | $5 ea ($10) | Alternative: 3.6mm for wider FOV |
| IR Longpass Filter (>700nm, 1"x1") | 1 | $8 | Optional but recommended |

### IR Strobing

| Item | Qty | Est. Price | Notes |
|------|-----|-----------|-------|
| D4184 MOSFET Module (logic-level) | 1 | $2 | Screw terminals, 3.3V GPIO compatible, nanosecond switching |
| Current-limiting resistor (24Ω, 5W) | 1 | $1 | Sets LED current to ~700mA. R = (Vsupply - Vf_total) / 0.7A |
| Vishay VSMA1085400 IR LED (880nm) | 6 | $2 ea ($12) | Wire in series |
| 24V DC Power Supply (30W) | 1 | $15 | Powers LEDs + Pi via buck converter |

### Wiring & Connectors

| Item | Qty | Est. Price | Notes |
|------|-----|-----------|-------|
| JST connectors, wire, heatshrink | 1 kit | $10 | 22AWG for signals, 18AWG for power |
| GPIO jumper wires | 5 | included | For PWM + trigger connections |

### Enclosure

| Item | Qty | Est. Price | Notes |
|------|-----|-----------|-------|
| 3D Printed Enclosure | 1 | Self-printed | ~200g PLA/PETG |
| M3 bolts/nuts assortment | 1 kit | $5 | For mounting cameras and boards |

## Wiring Diagram

```
24V PSU ──> 24Ω Resistor ──> 6x IR LEDs (series) ──> D4184 MOSFET ──> GND
                                                         ^ gate
                                                         |
                                                   GPIO 18 (PWM0)

GPIO 25 ──> Camera 2 External Trigger

Camera 1 ──CSI──> CAM0 port
Camera 2 ──CSI──> CAM1 port
```

## Alternative Hardware

LaunchTrac uses a hardware abstraction layer (HAL) with trait-based interfaces for cameras, GPIO, and PWM. The recommended build uses a Raspberry Pi 5, but the software can be adapted to other platforms.

### What the hardware must provide

1. **Global shutter camera(s)** — rolling shutter sensors produce motion distortion at golf ball speeds (150+ mph). The IMX296 is one of the most affordable global shutter sensors available.
2. **GPIO with PWM** — microsecond-precision pulse timing for IR LED strobes (20us pulses with 700-1800us delays).
3. **External trigger sync** — camera and strobes must fire in lockstep.

### Possible alternative compute boards

| Board | Pros | Cons | Est. Price |
|-------|------|------|-----------|
| **Raspberry Pi 5** (recommended) | Cheapest, best community support, dual CSI | Slower ML inference than GPU boards | $80 |
| **NVIDIA Jetson Nano/Orin Nano** | CUDA for faster ML inference | Different CSI connector (may need adapter), higher cost | $150-250 |
| **Orange Pi 5 / Rock Pi** | GPIO + CSI, lower cost than Jetson | Smaller community, driver support varies | $60-100 |

### Possible alternative cameras

| Camera | Interface | Pros | Cons | Est. Price |
|--------|-----------|------|------|-----------|
| **InnoMaker IMX296** (recommended) | CSI-2 | Cheapest global shutter, great Pi support | Monochrome only | $30 |
| **FLIR Blackfly S** | USB3 | Higher resolution, industrial quality | Expensive, needs USB3 host | $300+ |
| **Basler ace 2** | USB3/GigE | High frame rate, excellent SDK | Expensive | $400+ |

To add support for a new platform, implement the `CameraCapture`, `GpioController`, and `PwmStrobe` traits in `launchtrac-hal`. See `launchtrac-hal/src/mock.rs` for an example implementation.
