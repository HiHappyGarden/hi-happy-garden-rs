# hi-happy-garden-rs

# TODO

# Default Parameters Configuration via CMake

## Overview

You can configure the application's default values via CMake options. These values are integrated into the binary during compilation and used when the configuration file does not exist or is empty.

## Available Parameters

### WiFi Configuration

- **HHG_DEFAULT_WIFI_SSID**: WiFi network SSID (default: "")
- **HHG_DEFAULT_WIFI_PASSWORD**: WiFi network password (default: "")
- **HHG_DEFAULT_WIFI_HOSTNAME**: Device hostname (default: "hi-happy-garden")
- **HHG_DEFAULT_WIFI_ENABLED**: Enable WiFi at startup (default: OFF)

### General Configuration

- **HHG_DEFAULT_TIMEZONE**: Timezone offset in minutes (default: 0)
- **HHG_DEFAULT_DAYLIGHT_SAVING**: Enable daylight saving time (default: OFF)

## Usage Examples

### Using .env File

You can use a `.env` file to store your configuration and load it before building:

1. Create a `.env` file:
```bash
# .env
WIFI_SSID="MyNetwork"
WIFI_PASSWORD="MyPassword"
WIFI_HOSTNAME="garden-controller"
WIFI_ENABLED=ON
TIMEZONE=60
DAYLIGHT_SAVING=ON
BUILD_TYPE=Release
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
  -DHHG_DEFAULT_TIMEZONE=${TIMEZONE} \
  -DHHG_DEFAULT_DAYLIGHT_SAVING=${DAYLIGHT_SAVING}

# Build
cmake --build build -j$(nproc)
```

### Basic Configuration

```bash
cmake -B build \
  -DHHG_DEFAULT_WIFI_SSID="MyNetwork" \
  -DHHG_DEFAULT_WIFI_PASSWORD="MyPassword" \
  -DHHG_DEFAULT_WIFI_HOSTNAME="garden-controller" \
  -DHHG_DEFAULT_WIFI_ENABLED=ON
```

### Complete Configuration

```bash
cmake -B build \
  -DHHG_DEFAULT_WIFI_SSID="MyNetwork" \
  -DHHG_DEFAULT_WIFI_PASSWORD="MySecurePassword123" \
  -DHHG_DEFAULT_WIFI_HOSTNAME="garden-controller-01" \
  -DHHG_DEFAULT_WIFI_ENABLED=ON \
  -DHHG_DEFAULT_TIMEZONE=60 \
  -DHHG_DEFAULT_DAYLIGHT_SAVING=ON
```

### Production Configuration

```bash
# Central Europe (UTC+1 with daylight saving)
cmake -B build-production \
  -DCMAKE_BUILD_TYPE=Release \
  -DHHG_DEFAULT_WIFI_SSID="ProductionNetwork" \
  -DHHG_DEFAULT_WIFI_PASSWORD="ProdSecurePass456" \
  -DHHG_DEFAULT_WIFI_HOSTNAME="hhg-prod-01" \
  -DHHG_DEFAULT_WIFI_ENABLED=ON \
  -DHHG_DEFAULT_TIMEZONE=60 \
  -DHHG_DEFAULT_DAYLIGHT_SAVING=ON
```

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

## Files Involved

- **CMakeLists.txt**: Defines the options and passes them to cargo
- **main/build.rs**: Build script that generates the defaults.rs file
- **main/src/apps/configuration.rs**: Uses the defaults when necessary

## Options Verification

During CMake configuration, the set values are printed:

```
-- HHG_DEFAULT_WIFI_SSID: MyNetwork
-- HHG_DEFAULT_WIFI_HOSTNAME: garden-controller
-- HHG_DEFAULT_WIFI_ENABLED: ON
-- HHG_DEFAULT_TIMEZONE: 60
```

## Security Notes

⚠️ **WARNING**: WiFi passwords are compiled into the binary. For production environments:
- Use an encrypted configuration file instead of hard-coding credentials
- Consider using a secure provisioning system
- Do not share binaries that contain credentials

## Recommended Workflow

1. **Local development**: Use minimal defaults or no configuration
2. **Testing**: Use a dedicated test WiFi network
3. **Production**: Load configuration from an encrypted file, use CMake defaults only as fallback
