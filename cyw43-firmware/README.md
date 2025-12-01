# CYW43 Firmware Files

This directory should contain the WiFi firmware files for the CYW43 chip on the Raspberry Pico 2 W.

## Required Files

1. `43439A0.bin` - Main firmware blob
2. `43439A0_clm.bin` - Country Locale Matrix

## How to Obtain

These files can be obtained from the official Raspberry Pi Pico SDK or the Embassy project:

### Option 1: From Pico SDK
```bash
git clone https://github.com/raspberrypi/pico-sdk
cp pico-sdk/lib/cyw43-driver/firmware/43439A0.bin .
cp pico-sdk/lib/cyw43-driver/firmware/43439A0_clm.bin .
```

### Option 2: From Embassy
```bash
git clone https://github.com/embassy-rs/embassy
cp embassy/cyw43-firmware/*.bin .
```

## License

The firmware files are proprietary and subject to Infineon/Cypress licensing terms.
