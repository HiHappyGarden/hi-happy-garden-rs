# hi-happy-garden-rs

# TODO

# Install pkg
```
sudo apt install python3 git tar build-essential cmake gcc-arm-none-eabi libusb-1.0-0 libftdi1 gdb-multiarch libftdi1-2 libhidapi-hidraw0

```

# Picocom
```sh
picocom --omap crcrlf --echo -b 115200 /dev/ttyACM0
```

# Emulator

The firmware also runs on a Linux (or macOS) host, on the Pico 2 W board
emulated at the level of the `hhg_*` C wrappers of `src/pico`: the pico
platform layer, the drivers and the apps are the same code that runs on the
board, on osal-rs `posix` instead of FreeRTOS. The host models live in
`main/src/drivers/emulator` and `src/emulator`.

| Board | Emulator |
|---|---|
| UART0 (log, AT commands) | the terminal: stdout and stdin |
| Buttons, encoder, relays, RGB LED | the control socket (see below) |
| GPIO / PWM / ADC | in-memory pins, inputs idle at their pull, chip at 27 °C |
| DS3231 on I2C0 | register level model, starts at the host UTC time |
| SH1106 display on I2C1 | register level model, shown by the control socket |
| littlefs on flash | the same littlefs on a 256 KB RAM image, optionally kept in a file |
| mbedtls AES, SHA-256 accelerator | RustCrypto `aes` and `sha2` |
| CYW43 WiFi, lwIP | an access point accepting any credentials, DNS and UDP (NTP) on the host network; `--offline`: no access point in range |

The C sources it needs (littlefs, cJSON) are taken from `build/_deps` after a
firmware CMake build, or fetched once with:

```sh
scripts/emulator-deps.sh
```

Run the firmware, with the flash kept in `hhg-flash.bin` across restarts
(Ctrl+C to stop):

```sh
cd main
cargo run --no-default-features --features emulator,encryption --bin hhg-emulator -- --flash hhg-flash.bin
```

While it runs, the board is driven from another terminal through the
control socket (`$XDG_RUNTIME_DIR/hhg-emulator.sock`, `--control <path>` to
change it), with the emulator binary itself as the client:

```sh
cd main
alias hhg='cargo run -q --no-default-features --features emulator,encryption --bin hhg-emulator --'
hhg --send help             # the commands
hhg --send display          # the screen, drawn in the terminal
hhg --send "enc cw 3"       # turn the encoder 3 steps clockwise
hhg --send "enc click"      # press the encoder button
hhg --send "btn long"       # long press on the front panel button
hhg --send status           # relays, RGB LED, CYW43 LED, WiFi
hhg --send "wifi down"      # take the access point away
watch -n 0.5 ./target/debug/hhg-emulator --send display   # live screen
```

Run the firmware test suite, the exit status is 0 only if every test passed
(this is what the `Emulator` GitHub workflow does):

```sh
cd main
cargo run --no-default-features --features emulator,encryption,tests --bin hhg-emulator < /dev/null
```

`secrets.cmake` is read as for the firmware; without it, set the
`HHG_AES_KEY_SALT`, `HHG_AES_IV_SALT`, `HHG_SYSTEM_USER_EMAIL` and
`HHG_SYSTEM_USER_PASSWORD` environment variables.

Real-time behaviour, interrupt timing, the two RP2350 cores, heap and stack
limits, the real radio and the real peripherals still need the board.

# Default Parameters Configuration via CMake

## Overview

You can configure the application's default values via CMake options. These values are integrated into the binary during compilation and used when the configuration file does not exist or is empty.

## Available Parameters

### WiFi Configuration

> **Note**: WiFi credentials (**HHG_DEFAULT_WIFI_SSID** and **HHG_DEFAULT_WIFI_PASSWORD**) should be configured using the `secrets.cmake` file, which is excluded from git. See [Secrets Configuration](#secrets-configuration).

- **HHG_DEFAULT_WIFI_SSID**: WiFi network SSID (default: "")
- **HHG_DEFAULT_WIFI_PASSWORD**: WiFi network password (default: "")
- **HHG_DEFAULT_WIFI_HOSTNAME**: Device hostname (default: "hi-happy-garden")
- **HHG_DEFAULT_WIFI_ENABLED**: Enable WiFi at startup (default: ON)

### NTP Configuration

- **HHG_DEFAULT_NTP_SERVER**: NTP server address (default: "0.europe.pool.ntp.org")
- **HHG_DEFAULT_NTP_PORT**: NTP server port (default: 123)
- **HHG_DEFAULT_NTP_MSG_LEN**: NTP message length in bytes (default: 48)

### General Configuration

- **HHG_DEFAULT_TIMEZONE**: Timezone offset in minutes (default: 60, i.e. UTC+1)
- **HHG_DEFAULT_DAYLIGHT_SAVING_ENABLED**: Enable daylight saving time (default: OFF)

### System User Configuration

> **Required**: System user credentials must be defined in `secrets.cmake`. They have no default and CMake configuration fails if they are missing or empty.

The system user is stored at position 0 of the session user list and is loaded from the config file at startup. It is initialised from these CMake values only when the config file does not yet exist on the device.

- **HHG_SYSTEM_USER_EMAIL**: Email address of the system user (required, `secrets.cmake` only)
- **HHG_SYSTEM_USER_PASSWORD**: Plain-text password of the system user — it is hashed with SHA256 before being stored (required, `secrets.cmake` only)

### AES Encryption Configuration

The filesystem uses AES encryption with keys derived from the hardware's unique ID. The salt values used in the key derivation process must be defined in `secrets.cmake`: they have no default and CMake configuration fails if they are missing or empty.

- **HHG_AES_KEY_SALT**: Salt for AES key derivation (required, `secrets.cmake` only)
- **HHG_AES_IV_SALT**: Salt for AES IV derivation (required, `secrets.cmake` only)

> **How it works**: The encryption key and IV are generated using SHA256-based key derivation:
> - Key (32 bytes): `SHA256(hardware_unique_id || HHG_AES_KEY_SALT)`
> - IV (16 bytes): First 16 bytes of `SHA256(hardware_unique_id || HHG_AES_IV_SALT)`
>
> This ensures that each device has unique encryption keys while allowing customization per deployment for additional security.

> **Security Note**: Changing these salt values will make previously encrypted data unreadable. Use different salts for different deployment environments to prevent cross-device data access.

#### Daylight Saving Time Configuration

When **HHG_DEFAULT_DAYLIGHT_SAVING_ENABLED** is enabled, you can configure the DST transition dates:

- **HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_MONTH**: Month when DST starts (1-12, default: 2)
- **HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_DAY**: Day when DST starts (1-31, default: 31)
- **HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_HOUR**: Hour when DST starts (0-23, default: 2)
- **HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_MONTH**: Month when DST ends (1-12, default: 9)
- **HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_DAY**: Day when DST ends (1-31, default: 31)
- **HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_HOUR**: Hour when DST ends (0-23, default: 3)

## Secrets Configuration

Sensitive values are not hardcoded in CMakeLists.txt: they are read from the `secrets.cmake` file in the project root, which is **mandatory**. CMake configuration stops with an error if the file does not exist.

1. Copy the example file:
   ```bash
   cp secrets.cmake.example secrets.cmake
   ```

2. Edit `secrets.cmake` with your values:
   ```cmake
   # Required: configuration fails if any of these is missing or empty
   set(HHG_AES_KEY_SALT "MyCustomKeySalt2024")
   set(HHG_AES_IV_SALT "MyCustomIVSalt2024")
   set(HHG_SYSTEM_USER_EMAIL "admin@hhg.local")
   set(HHG_SYSTEM_USER_PASSWORD "change_me")

   # Optional
   set(HHG_DEFAULT_WIFI_SSID "YourSSID")
   set(HHG_DEFAULT_WIFI_PASSWORD "YourPassword")
   set(HHG_DEFAULT_WIFI_AUTH "3")   # 0=Open,1=Web,2=WPA,3=WPA2,4=WPA2-Mixed,5=WPA3,6=WPA2-WPA3
   set(HHG_DEFAULT_WIFI_ENABLED ON)
   set(HHG_DEFAULT_DAYLIGHT_SAVING_ENABLED ON)
   ```

Rules enforced by CMakeLists.txt:

| Variable | In `secrets.cmake` |
|---|---|
| `HHG_AES_KEY_SALT`, `HHG_AES_IV_SALT`, `HHG_SYSTEM_USER_EMAIL`, `HHG_SYSTEM_USER_PASSWORD` | **Required**, no default, cannot be passed with `-D` |
| `HHG_DEFAULT_WIFI_SSID`, `HHG_DEFAULT_WIFI_PASSWORD`, `HHG_DEFAULT_WIFI_AUTH`, `HHG_DEFAULT_WIFI_ENABLED`, `HHG_DEFAULT_DAYLIGHT_SAVING_ENABLED` | Optional |
| Any other variable | Ignored, with a CMake warning |

The file is included in a function scope, so only the variables above leave it. Use plain `set(VAR value)`: a `set(... CACHE ... FORCE)` would write the cache directly and bypass these checks.

> **Note**: The `secrets.cmake` file is excluded from git (`*.cmake` rule in `.gitignore`) and will not be committed to version control.

## Usage Examples

### Using .env File

You can use a `.env` file to store the non-secret configuration and load it before building (`secrets.cmake` is still required):

1. Create a `.env` file:
```bash
# .env
WIFI_SSID="MyNetwork"
WIFI_PASSWORD="MyPassword"
WIFI_HOSTNAME="garden-controller"
WIFI_ENABLED=ON
NTP_SERVER="0.europe.pool.ntp.org"
NTP_PORT=123
NTP_MSG_LEN=48
TIMEZONE=60
DAYLIGHT_SAVING=ON
BUILD_TYPE=Release
...
```

2. Load the configuration:
```bash
# Load environment variables
source .env

# Configure with CMake
cmake -B build \
  -DCMAKE_BUILD_TYPE=${BUILD_TYPE:-Release} \
  -DHHG_DEFAULT_WIFI_SSID="${WIFI_SSID}" \
  -DHHG_DEFAULT_WIFI_PASSWORD="${WIFI_PASSWORD}" \
  -DHHG_DEFAULT_WIFI_HOSTNAME="${WIFI_HOSTNAME}" \
  -DHHG_DEFAULT_WIFI_ENABLED=${WIFI_ENABLED} \
  -DHHG_DEFAULT_NTP_SERVER="${NTP_SERVER}" \
  -DHHG_DEFAULT_NTP_PORT=${NTP_PORT} \
  -DHHG_DEFAULT_NTP_MSG_LEN=${NTP_MSG_LEN} \
  -DHHG_DEFAULT_TIMEZONE=${TIMEZONE} \
  -DHHG_DEFAULT_DAYLIGHT_SAVING=${DAYLIGHT_SAVING}

# Build
cmake --build build -j$(nproc)
```

### Basic Configuration

```bash
cmake -B build \
  -DHHG_DEFAULT_WIFI_HOSTNAME="garden-controller" \
  -DHHG_DEFAULT_WIFI_ENABLED=ON
```

### Complete Configuration

```bash
cmake -B build \
  -DHHG_DEFAULT_WIFI_HOSTNAME="garden-controller-01" \
  -DHHG_DEFAULT_WIFI_ENABLED=ON \
  -DHHG_DEFAULT_NTP_SERVER="pool.ntp.org" \
  -DHHG_DEFAULT_NTP_PORT=123 \
  -DHHG_DEFAULT_NTP_MSG_LEN=48 \
  -DHHG_DEFAULT_TIMEZONE=60 \
  -DHHG_DEFAULT_DAYLIGHT_SAVING=ON
```

### Production Configuration

```bash
# Central Europe (UTC+1 with daylight saving)
cmake -B build-production \
  -DCMAKE_BUILD_TYPE=Release \
  -DHHG_DEFAULT_WIFI_HOSTNAME="hhg-prod-01" \
  -DHHG_DEFAULT_WIFI_ENABLED=ON \
  -DHHG_DEFAULT_NTP_SERVER="0.europe.pool.ntp.org" \
  -DHHG_DEFAULT_NTP_PORT=123 \
  -DHHG_DEFAULT_NTP_MSG_LEN=48 \
  -DHHG_DEFAULT_TIMEZONE=60 \
  -DHHG_DEFAULT_DAYLIGHT_SAVING=ON
```

> AES salts and system user credentials are not passed on the command line: set production values in `secrets.cmake`.

### Debug Only (without WiFi)

```bash
cmake -B build-debug \
  -DCMAKE_BUILD_TYPE=Debug \
  -DHHG_DEFAULT_WIFI_ENABLED=OFF \
  -DHHG_TESTS=ON
```

## How It Works

1. **Build Time**: CMake passes the options as environment variables to the `cargo build` process
2. **Compilation Time**: The `build.rs` file reads these environment variables and generates a `defaults.rs` file with Rust constants
3. **Runtime**: When `Config::load()` is called:
   - If the configuration file exists and is not empty, it is loaded
   - Otherwise, the compiled default values are used
   - The system user (position 0) is initialised from `HHG_SYSTEM_USER_EMAIL` / `HHG_SYSTEM_USER_PASSWORD` only on first boot (empty config file); the password is stored as a SHA256 hash

## Files Involved

- **CMakeLists.txt**: Defines the options, loads `secrets.cmake` and passes them to cargo
- **secrets.cmake** (not in git, copy of `secrets.cmake.example`): Required secrets and optional overrides
- **main/build.rs**: Build script that generates the defaults.rs file
- **main/src/apps/configuration.rs**: Uses the defaults when necessary

## Options Verification

During CMake configuration, the set values are printed:

```
-- HHG_DEFAULT_WIFI_ENABLED: ON
-- HHG_DEFAULT_TIMEZONE: 60
```

## Security Notes

⚠️ **WARNING**: WiFi passwords are compiled into the binary. For production environments:
- Use an encrypted configuration file instead of hard-coding credentials
- Consider using a secure provisioning system
- Do not share binaries that contain credentials

### AES Encryption Security

- **Unique per device**: Encryption keys are derived from each device's hardware unique ID
- **Customizable salts**: `HHG_AES_KEY_SALT` and `HHG_AES_IV_SALT` are required in `secrets.cmake`, use different values for different deployments
- **Cryptographically secure**: Uses SHA256-based key derivation function (KDF)
- **⚠️ Data compatibility**: Changing salt values makes previously encrypted data unreadable
- **Best practice**: Use different salts for development, testing, and production environments

## Recommended Workflow

1. **Local development**: Use minimal defaults or no configuration
2. **Testing**: Use a dedicated test WiFi network
3. **Production**: Load configuration from an encrypted file, use CMake defaults only as fallback

## License

This project is licensed under the GNU General Public License v2.0 or later (GPL-2.0-or-later) - see the [LICENSE](LICENSE) file for details.

### Sub-projects

This repository includes several sub-projects, each with its own license:

- **[at-parser-rs](at-parser-rs/)** - AT command parser library  
  Licensed under LGPL-2.1-or-later

- **[cjson-bindings](cjson-bindings/)** - Safe Rust bindings for cJSON  
  Licensed under LGPL-2.1-or-later

- **[osal-rs](osal-rs/)** - Operating System Abstraction Layer  
  Licensed under LGPL-2.1-or-later

See the LICENSE file in each sub-project directory for complete license information.
