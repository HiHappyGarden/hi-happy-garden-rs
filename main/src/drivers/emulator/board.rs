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

//! What is wired to the board pins, seen from outside the chip.
//!
//! The firmware owns its pin map (`pico::gpio::GPIO_CONFIGS`) and does not
//! expose the pin numbers; this is the copy the control channel needs to
//! press a button or read a relay. `emulator::tests` drives every pin here
//! and checks the firmware sees it on the expected peripheral, so the two
//! cannot drift apart unnoticed.

/// Front panel button, pull-up, active low
pub(super) const BTN: u32 = 18;

/// Encoder push button, pull-up, active low
pub(super) const ENCODER_BTN: u32 = 19;

/// Encoder phase read as the CCW signal, pull-down
pub(super) const ENCODER_CCW: u32 = 20;

/// Encoder phase read as the CW signal, pull-down
pub(super) const ENCODER_CW: u32 = 21;

/// RGB LED channels, PWM
pub(super) const LED_RED: u32 = 13;
pub(super) const LED_GREEN: u32 = 14;
pub(super) const LED_BLUE: u32 = 15;

/// Valve relays, in zone order
pub(super) const RELAYS: [u32; 4] = [6, 7, 8, 9];

/// The LED on the CYW43 radio: WL_GPIO0, not an RP2350 pin
pub(super) const CYW43_LED: u32 = 0;
