# Hi Happy Garden RS

Smart irrigator firmware written in Rust for Raspberry Pico 2 W (RP2350).

## Features

- **Multi-zone irrigation control**: Support for up to 4 irrigation zones
- **Scheduling system**: Configurable schedules with day-of-week support
- **WiFi connectivity**: Connect via CYW43 WiFi chip on Pico 2 W
- **LED status indication**: Visual feedback for system state
- **Low power**: Power-save mode for WiFi

## Hardware Requirements

- Raspberry Pico 2 W (RP2350 with CYW43 WiFi)
- Relay module (4-channel recommended)
- Power supply (5V)
- Solenoid valves for irrigation zones

## Pin Configuration

| Pin | Function |
|-----|----------|
| GPIO 2 | Zone 1 Relay |
| GPIO 3 | Zone 2 Relay |
| GPIO 4 | Zone 3 Relay |
| GPIO 5 | Zone 4 Relay |
| GPIO 23 | CYW43 WiFi |
| GPIO 24 | CYW43 WiFi |
| GPIO 25 | CYW43 WiFi |
| GPIO 29 | CYW43 WiFi |

## Building

### Prerequisites

1. Install Rust and cargo:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Add the ARM target:
   ```bash
   rustup target add thumbv8m.main-none-eabihf
   ```

3. Install probe-rs for flashing:
   ```bash
   cargo install probe-rs-tools
   ```

4. Download CYW43 firmware files and place them in `cyw43-firmware/` directory:
   - `43439A0.bin`
   - `43439A0_clm.bin`

### Build Commands

```bash
# Build in debug mode
cargo build

# Build in release mode
cargo build --release

# Flash to device
cargo run --release
```

## Project Structure

```
hi-happy-garden-rs/
├── src/
│   ├── main.rs          # Main application entry point
│   ├── config/          # Configuration management
│   │   └── mod.rs
│   ├── irrigation/      # Irrigation control logic
│   │   └── mod.rs
│   └── wifi/            # WiFi connectivity
│       └── mod.rs
├── cyw43-firmware/      # WiFi firmware files
├── .cargo/
│   └── config.toml      # Cargo configuration
├── memory.x             # Memory layout
├── build.rs             # Build script
├── Cargo.toml           # Dependencies
└── README.md
```

## LED Status Indicators

| Pattern | Meaning |
|---------|---------|
| Slow blink | Initializing |
| Fast blink | Connecting to WiFi |
| Solid on | Ready |
| Medium blink | Irrigating |
| Very fast blink | Error |

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## Credits

Part of the Hi Happy Garden project.
- Original C++ implementation: [hi-happy-garden](https://github.com/HiHappyGarden/hi-happy-garden)
- Author: Antonio Salsi <passy.linux@zresa.it>