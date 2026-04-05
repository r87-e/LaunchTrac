# LaunchTrac v2 Bill of Materials

Total estimated cost: **~$195 USD** (no custom PCB required)

## Compute

| Item | Qty | Est. Price | Notes |
|------|-----|-----------|-------|
| Raspberry Pi 5 (8GB) | 1 | $80 | 4GB works but 8GB recommended |
| MicroSD Card (64GB) | 1 | $10 | Class 10 / A2 recommended |
| Pi 5 Active Cooler | 1 | $5 | Required for sustained processing |
| USB-C PD Power Supply (27W) | 1 | $12 | Official Pi 5 PSU recommended |

## Cameras & Optics

| Item | Qty | Est. Price | Notes |
|------|-----|-----------|-------|
| InnoMaker IMX296 Global Shutter Camera | 2 | $30 ea ($60) | Monochrome, CSI-2 interface |
| M12 6mm Lens | 2 | $5 ea ($10) | Alternative: 3.6mm for wider FOV |
| IR Longpass Filter (>700nm, 1"x1") | 1 | $8 | Optional but recommended |

## IR Strobing

| Item | Qty | Est. Price | Notes |
|------|-----|-----------|-------|
| Meanwell LDD-700H LED Driver | 1 | $8 | Constant-current, PWM input, 9-56V |
| Vishay VSMA1085400 IR LED (880nm) | 6 | $2 ea ($12) | Wire in series |
| 24V DC Power Supply (30W) | 1 | $15 | Powers LEDs + Pi via buck converter |

## Wiring & Connectors

| Item | Qty | Est. Price | Notes |
|------|-----|-----------|-------|
| JST connectors, wire, heatshrink | 1 kit | $10 | 22AWG for signals, 18AWG for power |
| GPIO jumper wires | 5 | included | For PWM + trigger connections |

## Enclosure

| Item | Qty | Est. Price | Notes |
|------|-----|-----------|-------|
| 3D Printed Enclosure (V2) | 1 | Self-printed | ~200g PLA/PETG |
| M3 bolts/nuts assortment | 1 kit | $5 | For mounting cameras and boards |

## Wiring Diagram

```
24V PSU ──→ Meanwell LDD-700H ──→ 6x IR LEDs (series)
                    ↑ PWM input
                    │
              Pi GPIO 18 (PWM0)

Pi GPIO 25 ──→ Camera 2 External Trigger

Camera 1 ──CSI──→ Pi CAM0 port
Camera 2 ──CSI──→ Pi CAM1 port
```

## Key Differences from LaunchTrac v1

| Aspect | v1 | v2 |
|--------|----|----|
| Custom PCB | Required (V3 connector board) | Eliminated |
| LED driving | SPI bit-bang + boost converter | Hardware PWM + LDD-700H |
| Current feedback | MCP3202 ADC | Not needed (constant-current driver) |
| Soldering | SMD + through-hole | Minimal wire connections only |
| Total cost | ~$250 | ~$195 |
