/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, see <https://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

//! Emulator only tests: the models agree with the firmware on how the board
//! is wired, the control channel acts on what the firmware sees, and the
//! network path works end to end on the host network.
//!
//! Run after the driver tests, on the board `Hardware::init` brought up.

use alloc::format;
use core::time::Duration;
use std::net::UdpSocket;
use std::time::{SystemTime, UNIX_EPOCH};

use osal_rs::log_debug;
use osal_rs::os::System;
use osal_rs::utils::{Bytes, Error, OsalRsBool, Result};

use super::{board, control, cyw43, gpio, sh1106};
use crate::drivers::gpio::Gpio;
use crate::drivers::network::{IP4Addr, Network};
use crate::drivers::platform::GpioPeripheral;
use crate::drivers::relays::Relays;
use crate::tests::{TestStats, hardware, run_tests, test_assert, test_assert_eq};
use crate::traits::lcd_display::{LCDDisplayFn, LCDWriteMode};
use crate::traits::relays::Relays as RelaysFn;

const TAG: &str = "EmulatorTests";

/// Seconds between the NTP epoch (1900) and the Unix one
const NTP_DELTA: u64 = 2_208_988_800;

fn test_input_pin_map() -> Result<()> {
    let gpio = Gpio::shared();

    // Pull-up buttons: driven low they read 0, released the pull-up wins
    for (peripheral, pin) in [(GpioPeripheral::Btn, board::BTN), (GpioPeripheral::EncoderBtn, board::ENCODER_BTN)] {
        gpio::drive_input(pin, Some(false));
        let pressed = gpio.read(&peripheral)?;
        gpio::drive_input(pin, None);
        test_assert_eq!(pressed, 0, "{peripheral:?} not on GPIO {pin}");
        test_assert_eq!(gpio.read(&peripheral)?, 1, "{peripheral:?} not released");
        // Past the button debounce before the next edge
        System::delay_with_to_tick(Duration::from_millis(60));
    }

    // Pull-down encoder phases: driven high they read 1
    for (peripheral, pin) in [(GpioPeripheral::EncoderCCW, board::ENCODER_CCW), (GpioPeripheral::EncoderCW, board::ENCODER_CW)] {
        gpio::drive_input(pin, Some(true));
        let high = gpio.read(&peripheral)?;
        gpio::drive_input(pin, None);
        test_assert_eq!(high, 1, "{peripheral:?} not on GPIO {pin}");
        test_assert_eq!(gpio.read(&peripheral)?, 0, "{peripheral:?} not released");
        System::delay_with_to_tick(Duration::from_millis(10));
    }
    Ok(())
}

fn test_output_pin_map() -> Result<()> {
    let relays = Relays::shared();
    let peripherals = [GpioPeripheral::Relay0, GpioPeripheral::Relay1, GpioPeripheral::Relay2, GpioPeripheral::Relay3];
    for (peripheral, pin) in peripherals.into_iter().zip(board::RELAYS) {
        test_assert_eq!(relays.set_relay_state(peripheral, true), OsalRsBool::True);
        let on = gpio::output_level(pin);
        relays.set_relay_state(peripheral, false);
        test_assert_eq!(on, Some(true), "{peripheral:?} not on GPIO {pin}");
        test_assert_eq!(gpio::output_level(pin), Some(false));
    }

    let gpio_driver = Gpio::shared();
    for (peripheral, pin) in [(GpioPeripheral::LedRed, board::LED_RED), (GpioPeripheral::LedGreen, board::LED_GREEN), (GpioPeripheral::LedBlue, board::LED_BLUE)] {
        test_assert_eq!(gpio_driver.set_pwm(&peripheral, 255), OsalRsBool::True);
        let level = gpio::pwm_level(pin);
        gpio_driver.set_pwm(&peripheral, 0);
        test_assert_eq!(level, 255, "{peripheral:?} not on GPIO {pin}");
    }

    test_assert_eq!(gpio_driver.write(&GpioPeripheral::Cyw43Led, 1), OsalRsBool::True);
    let led = gpio::wl_gpio(board::CYW43_LED);
    gpio_driver.write(&GpioPeripheral::Cyw43Led, 0);
    test_assert!(led, "Cyw43Led not on WL_GPIO{}", board::CYW43_LED);
    Ok(())
}

fn test_control_commands() -> Result<()> {
    let ok = |command: &str| control::execute(command).map_err(|e| Error::UnhandledOwned(format!("{command}: {e}")));

    test_assert!(ok("help")?.contains("enc cw|ccw"));

    ok("btn press")?;
    test_assert_eq!(Gpio::shared().read(&GpioPeripheral::Btn)?, 0);
    ok("btn release")?;
    test_assert_eq!(Gpio::shared().read(&GpioPeripheral::Btn)?, 1);

    // Two edges per detent: a full turn of steps lands back on the rest state
    ok("enc cw 2")?;
    test_assert_eq!((gpio::level(board::ENCODER_CCW), gpio::level(board::ENCODER_CW)), (false, false));
    ok("enc ccw")?;
    test_assert_eq!((gpio::level(board::ENCODER_CCW), gpio::level(board::ENCODER_CW)), (true, true));
    ok("enc ccw")?;

    test_assert!(ok("status")?.contains("relays     1:off 2:off 3:off 4:off"));

    for bad in ["", "nope", "btn twist", "enc cw 0", "enc cw 1000", "gpio x high"] {
        test_assert!(control::execute(bad).is_err(), "{bad:?} accepted");
    }
    Ok(())
}

fn test_display_render() -> Result<()> {
    let mut display = hardware().get_lcd_display();
    display.clear();
    // Buffer column 2 is the first visible one: the controller RAM is 132
    // columns wide, the panel 128
    display.draw_pixel(2, 0, LCDWriteMode::ADD)?;
    display.draw_pixel(129, 63, LCDWriteMode::ADD)?;
    display.draw()?;

    let lit = sh1106::lit_pixels();
    let text = sh1106::render_text();
    let pbm = sh1106::render_pbm();

    display.clear();
    display.draw()?;

    log_debug!(TAG, "display:\n{text}");
    test_assert_eq!(lit, 2);
    // The pixels land in the panel corners, upright as the firmware sets it up
    let rows: alloc::vec::Vec<&str> = text.lines().collect();
    test_assert_eq!(rows.len(), sh1106::PANEL_HEIGHT / 2 + 2);
    test_assert!(rows[1].chars().nth(1) == Some('▀'), "top-left pixel missing: {:?}", rows[1]);
    test_assert!(rows[rows.len() - 2].chars().nth(sh1106::PANEL_WIDTH) == Some('▄'), "bottom-right pixel missing");
    test_assert!(pbm.starts_with(b"P4\n128 64\n"));
    test_assert_eq!(pbm.len(), b"P4\n128 64\n".len() + sh1106::PANEL_WIDTH / 8 * sh1106::PANEL_HEIGHT);
    test_assert_eq!(sh1106::lit_pixels(), 0);
    Ok(())
}

/// Associates the emulated radio, as the WiFi driver does on connect.
fn join() -> Result<()> {
    unsafe {
        super::cyw43::hhg_cyw43_arch_enable_sta_mode();
        let ret = super::cyw43::hhg_cyw43_arch_wifi_connect(core::ptr::null(), core::ptr::null(), 0);
        if ret != 0 {
            return Err(Error::ReturnWithCode(ret));
        }
    }
    Ok(())
}

fn leave() {
    unsafe { super::cyw43::hhg_cyw43_arch_disable_sta_mode() };
}

/// A one shot NTP server on localhost answering with `unix_time`.
fn fake_ntp_server(unix_time: u64) -> Result<u16> {
    let socket = UdpSocket::bind("127.0.0.1:0").map_err(|e| Error::UnhandledOwned(format!("{e}")))?;
    let port = socket.local_addr().map_err(|e| Error::UnhandledOwned(format!("{e}")))?.port();
    let _ = socket.set_read_timeout(Some(std::time::Duration::from_secs(5)));

    std::thread::spawn(move || {
        let mut request = [0u8; 48];
        let Ok((_, client)) = socket.recv_from(&mut request) else {
            return;
        };
        let mut reply = [0u8; 48];
        reply[0] = 0x24; // LI 0, version 4, mode 4 (server)
        reply[1] = 1; // stratum 1
        reply[40..44].copy_from_slice(&((unix_time + NTP_DELTA) as u32).to_be_bytes());
        let _ = socket.send_to(&reply, client);
    });
    Ok(port)
}

fn test_network_offline() -> Result<()> {
    cyw43::set_online(false);
    let joined = join();
    let link = Network::is_link_up();
    let dns = Network::dns_resolve_addrress(&Bytes::from_str("localhost")).is_ok();
    leave();
    cyw43::set_online(true);

    test_assert!(joined.is_err(), "joined with no access point");
    test_assert!(!link);
    test_assert!(!dns, "DNS answered with no link");
    Ok(())
}

fn test_network_ntp() -> Result<()> {
    join()?;
    let ret = (|| {
        test_assert!(Network::is_link_up());
        test_assert!(Network::dhcp_supplied_address());
        test_assert!(Network::dhcp_get_binary_ip_address() != 0);

        let ip = Network::dns_resolve_addrress(&Bytes::from_str("localhost"))?;
        let ip = unsafe { &*(ip as *const dyn crate::traits::network::IpAddress as *const IP4Addr) };
        // lwIP layout: network byte order in memory
        test_assert_eq!(ip.addr, u32::from_ne_bytes([127, 0, 0, 1]));

        let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let port = fake_ntp_server(now)?;
        let ip = Network::dns_resolve_addrress(&Bytes::from_str("localhost"))?;
        let timestamp = Network::ntp_request(ip, port, 48)?;
        log_debug!(TAG, "NTP from localhost:{port}: {timestamp}");
        test_assert_eq!(timestamp, now as i64);
        Ok(())
    })();
    leave();
    ret
}

pub(crate) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_input_pin_map,
        test_output_pin_map,
        test_control_commands,
        test_display_render,
        test_network_offline,
        test_network_ntp,
    );
}
