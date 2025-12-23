# co2zy

An ESP32-C6 based air quality monitor that displays real-time environmental data on an OLED screen and indicates air quality status via an RGB LED.

## Components

### Sensors
- **AHT20**: Measures temperature and humidity via I2C
- **ENS160**: Measures air quality including eCO₂ (ppm), TVOC (ppb), and Air Quality Index (AQI)

### Display
- **SSD1306**: 128x64 OLED display showing all measurements

### LED Indicator
- **RGB LED**: Color-coded air quality indicator using RMT (Remote Control) protocol

## Hardware Connections

| Component | ESP32-C6 Pin |
|-----------|-------------|
| I2C SCL   | GPIO18      |
| I2C SDA   | GPIO19      |
| RGB LED   | GPIO8       |

## Building

### Option 1: Using Nix (Recommended)

The project includes a Nix flake that provides all development dependencies including the Rust nightly toolchain, espflash, and ESP development tools.

```bash
nix develop
```

This will drop you into a shell with:
- Nightly Rust toolchain with ESP32-C6 target
- `espflash` for flashing the device
- `espup` for ESP development tools
- Additional build tools (lld, llvm, just)

### Option 2: Manual Setup

Install the required tools manually:
- Rust nightly toolchain (configured via `rust-toolchain.toml`)
- `espflash` tool for flashing the device
- ESP32-C6 target: `rustup target add esp32c6-unknown-none-elf`

### Build and Flash

To build in release mode and flash directly to the ESP32-C6 device:

```bash
cargo run --release
```

This will compile the code with optimizations and use `espflash` to write the firmware to the connected device.

## Display

The OLED shows the following information:
- Air quality status (Excellent/Good/Moderate/Poor/Unhealthy)
- eCO₂ in ppm (alternates with TVOC every 5 seconds)
- Temperature in Celsius
- Relative humidity in percentage

## LED Colors

The RGB LED indicates air quality status:
- **Green (120° hue)**: Excellent air quality
- **Green-yellow (90° hue)**: Good air quality
- **Yellow-orange (45° hue)**: Moderate air quality
- **Red-orange (15° hue)**: Poor air quality
- **Red (0° hue)**: Unhealthy air quality
- **Blue-cyan (180° hue)**: Computing/unknown air quality

## Features

- Async/await using Embassy executor
- Shared I2C bus for multiple sensors
- defmt logging for debugging
- Optimized release builds with LTO and size optimization
