# hi-happy-garden-rs

# TODO

# Default Parameters Configuration via CMake

## Overview

You can configure the application's default values via CMake options. These values are integrated into the binary during compilation and used when the configuration file does not exist or is empty.

## Available Parameters

### WiFi Configuration

> **Note**: WiFi credentials (**HHG_DEFAULT_WIFI_SSID** and **HHG_DEFAULT_WIFI_PASSWORD**) should be configured using the `secrets.cmake` file, which is excluded from git. See the `secrets.cmake.example` file for reference.

- **HHG_DEFAULT_WIFI_SSID**: WiFi network SSID (default: "")
- **HHG_DEFAULT_WIFI_PASSWORD**: WiFi network password (default: "")
- **HHG_DEFAULT_WIFI_HOSTNAME**: Device hostname (default: "hi-happy-garden")
- **HHG_DEFAULT_WIFI_ENABLED**: Enable WiFi at startup (default: OFF)

### NTP Configuration

- **HHG_DEFAULT_NTP_SERVER**: NTP server address (default: "0.europe.pool.ntp.org")
- **HHG_DEFAULT_NTP_PORT**: NTP server port (default: 123)
- **HHG_DEFAULT_NTP_MSG_LEN**: NTP message length in bytes (default: 48)

### General Configuration

- **HHG_DEFAULT_TIMEZONE**: Timezone offset in minutes (default: 0)
- **HHG_DEFAULT_DAYLIGHT_SAVING**: Enable daylight saving time (default: OFF)

### AES Encryption Configuration

The filesystem uses AES encryption with keys derived from the hardware's unique ID. You can customize the salt values used in the key derivation process:

- **HHG_AES_KEY_SALT**: Salt for AES key derivation (default: "AES_KEY")
- **HHG_AES_IV_SALT**: Salt for AES IV derivation (default: "AES_IV")

> **How it works**: The encryption key and IV are generated using SHA256-based key derivation:
> - Key (32 bytes): `SHA256(hardware_unique_id || HHG_AES_KEY_SALT)`
> - IV (16 bytes): First 16 bytes of `SHA256(hardware_unique_id || HHG_AES_IV_SALT)`
>
> This ensures that each device has unique encryption keys while allowing customization per deployment for additional security.

> **Security Note**: Changing these salt values will make previously encrypted data unreadable. Use different salts for different deployment environments to prevent cross-device data access.

#### Daylight Saving Time Configuration

When **HHG_DEFAULT_DAYLIGHT_SAVING** is enabled, you can configure the DST transition dates:

- **HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_MONTH**: Month when DST starts (1-12, default: 2)
- **HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_DAY**: Day when DST starts (1-31, default: 31)
- **HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_HOUR**: Hour when DST starts (0-23, default: 2)
- **HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_MONTH**: Month when DST ends (1-12, default: 9)
- **HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_DAY**: Day when DST ends (1-31, default: 31)
- **HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_HOUR**: Hour when DST ends (0-23, default: 3)

## Secrets Configuration

For security reasons, WiFi credentials should not be hardcoded in CMakeLists.txt. Instead, use the `secrets.cmake` file:

1. Copy the example file:
   ```bash
   cp secrets.cmake.example secrets.cmake
   ```

2. Edit `secrets.cmake` with your credentials:
   ```cmake
   set(HHG_DEFAULT_WIFI_SSID "YourSSID")
   set(HHG_DEFAULT_WIFI_PASSWORD "YourPassword")
   
   # Optional: Customize AES encryption salts for enhanced security
   # These are combined with hardware unique_id to derive encryption keys
   set(HHG_AES_KEY_SALT "MyCustomKeySalt2024")
   set(HHG_AES_IV_SALT "MyCustomIVSalt2024")
   ```

3. Include it in your CMakeLists.txt (if not already included):
   ```cmake
   include(secrets.cmake OPTIONAL)
   ```

> **Note**: The `secrets.cmake` file is in `.gitignore` and will not be committed to version control.

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
  -DHHG_DEFAULT_DAYLIGHT_SAVING=ON \
  -DHHG_AES_KEY_SALT="PROD_KEY_2024" \
  -DHHG_AES_IV_SALT="PROD_IV_2024"
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
- **Customizable salts**: Change `HHG_AES_KEY_SALT` and `HHG_AES_IV_SALT` for different deployments
- **Cryptographically secure**: Uses SHA256-based key derivation function (KDF)
- **⚠️ Data compatibility**: Changing salt values makes previously encrypted data unreadable
- **Best practice**: Use different salts for development, testing, and production environments

## Recommended Workflow

1. **Local development**: Use minimal defaults or no configuration
2. **Testing**: Use a dedicated test WiFi network
3. **Production**: Load configuration from an encrypted file, use CMake defaults only as fallback
