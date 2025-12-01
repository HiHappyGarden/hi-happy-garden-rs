/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2025 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

//! Hi Happy Garden - Smart Irrigator Firmware for Raspberry Pico 2 W
//!
//! This firmware provides automated watering management with the following features:
//! - Multiple zone irrigation control
//! - Scheduling system
//! - WiFi connectivity
//! - LED status indication

#![no_std]
#![no_main]

mod config;
mod irrigation;
mod wifi;

use cyw43_pio::PioSpi;
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIO0};
use embassy_rp::pio::Pio;
use embassy_time::{Duration, Timer};
use fixed::types::U24F8;
use panic_probe as _;
use static_cell::StaticCell;

use crate::config::AppConfig;
use crate::irrigation::{IrrigationController, Zone, ZoneId};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => embassy_rp::pio::InterruptHandler<PIO0>;
});

/// WiFi firmware for CYW43
#[link_section = ".rodata.cyw43_fw"]
static CYW43_FW: &[u8] = include_bytes!("../cyw43-firmware/43439A0.bin");

/// WiFi CLM (Country Locale Matrix) for CYW43
#[link_section = ".rodata.cyw43_clm"]
static CYW43_CLM: &[u8] = include_bytes!("../cyw43-firmware/43439A0_clm.bin");

/// Static cell for the network stack
static STATE: StaticCell<cyw43::State> = StaticCell::new();

/// Application state machine states
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum AppState {
    /// Initial state, checking configuration
    Init,
    /// Connecting to WiFi
    ConnectingWifi,
    /// Ready for operation
    Ready,
    /// Executing irrigation
    Irrigating,
    /// Error state
    Error,
}

/// Main entry point for the firmware
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hi Happy Garden - Starting...");

    let p = embassy_rp::init(Default::default());

    // Initialize the onboard LED (GPIO 25 on Pico, connected via CYW43 on Pico W)
    info!("Initializing LED...");

    // Setup PIO for CYW43 WiFi chip
    let mut pio = Pio::new(p.PIO0, Irqs);

    let cs = Output::new(p.PIN_25, Level::High);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        U24F8::from_num(embassy_rp::clocks::clk_sys_freq()),
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );

    let pwr = Output::new(p.PIN_23, Level::Low);

    let state = STATE.init(cyw43::State::new());
    let (_net_device, mut control, runner) = cyw43::new(state, pwr, spi, CYW43_FW).await;
    // Note: _net_device will be used when implementing TCP/UDP networking features

    // Spawn the WiFi driver task
    spawner.must_spawn(wifi_task(runner));

    // Initialize the WiFi chip
    control.init(CYW43_CLM).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    info!("WiFi initialized");

    // Load configuration
    let config = AppConfig::default();
    info!("Configuration loaded");

    // Initialize irrigation controller with relay outputs
    // Using GPIO pins for relay control (example pins, adjust for your hardware)
    let zone1_pin = Output::new(p.PIN_2, Level::Low);
    let zone2_pin = Output::new(p.PIN_3, Level::Low);
    let zone3_pin = Output::new(p.PIN_4, Level::Low);
    let zone4_pin = Output::new(p.PIN_5, Level::Low);

    let zones = [
        Zone::new(ZoneId::Zone1, zone1_pin),
        Zone::new(ZoneId::Zone2, zone2_pin),
        Zone::new(ZoneId::Zone3, zone3_pin),
        Zone::new(ZoneId::Zone4, zone4_pin),
    ];

    let mut irrigation = IrrigationController::new(zones);
    info!("Irrigation controller initialized");

    // Main application state machine
    let mut state = AppState::Init;
    let mut led_state = false;

    loop {
        match state {
            AppState::Init => {
                info!("State: Init");
                // Blink LED slowly during init
                control.gpio_set(0, led_state).await;
                led_state = !led_state;
                Timer::after(Duration::from_millis(500)).await;

                // Transition to WiFi connection if enabled
                if config.wifi_enabled {
                    state = AppState::ConnectingWifi;
                } else {
                    state = AppState::Ready;
                }
            }
            AppState::ConnectingWifi => {
                info!("State: ConnectingWifi");
                // Fast blink during connection
                control.gpio_set(0, led_state).await;
                led_state = !led_state;
                Timer::after(Duration::from_millis(100)).await;

                // Try to connect to WiFi
                match wifi::connect(&mut control, &config).await {
                    Ok(()) => {
                        info!("WiFi connected");
                        state = AppState::Ready;
                    }
                    Err(e) => {
                        warn!("WiFi connection failed: {:?}", e);
                        Timer::after(Duration::from_secs(5)).await;
                    }
                }
            }
            AppState::Ready => {
                // Solid LED when ready
                control.gpio_set(0, true).await;

                // Check schedules and execute irrigation if needed
                if let Some(zone_id) = irrigation.check_schedules() {
                    info!("Starting irrigation for zone {:?}", zone_id);
                    irrigation.start_zone(zone_id);
                    state = AppState::Irrigating;
                }

                Timer::after(Duration::from_secs(1)).await;
            }
            AppState::Irrigating => {
                // Slow blink during irrigation
                control.gpio_set(0, led_state).await;
                led_state = !led_state;
                Timer::after(Duration::from_millis(250)).await;

                // Check if irrigation is complete
                if !irrigation.is_any_active() {
                    info!("Irrigation complete");
                    state = AppState::Ready;
                }

                // Update irrigation timers
                irrigation.update();
            }
            AppState::Error => {
                // Fast blink on error
                control.gpio_set(0, led_state).await;
                led_state = !led_state;
                Timer::after(Duration::from_millis(50)).await;
            }
        }
    }
}

/// WiFi driver task
#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}
