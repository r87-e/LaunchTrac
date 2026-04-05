# ADR-002: Eliminate Custom PCB

## Status
Accepted

## Context
LaunchTrac v1 requires a custom V3 Connector Board with a boost converter (MC33063A), MOSFET drivers, gate driver IC, ADC for current feedback, and SPI-based strobe timing. This is the biggest barrier to entry for DIY builders.

## Decision
Replace the custom PCB with an off-the-shelf Meanwell LDD-700H constant-current LED driver, driven directly by the Pi 5's hardware PWM.

## Rationale
- **LDD-700H**: $8, accepts 3.3V PWM input directly, self-regulates current (no ADC feedback needed)
- **Hardware PWM**: Pi 5 has dedicated PWM hardware with sub-microsecond jitter, better than the v1 SPI bit-bang approach (~8.7us per bit)
- **No soldering**: Only wire connections needed (LED driver to LEDs, GPIO to PWM input)
- **Cost reduction**: ~$55 saved in PCB components and manufacturing

## Consequences
- Cannot dynamically adjust LED current (fixed by LDD-700H rating selection)
- Lose the hardware duty-cycle limiter (must enforce in software)
- Simpler but less flexible than the custom board
