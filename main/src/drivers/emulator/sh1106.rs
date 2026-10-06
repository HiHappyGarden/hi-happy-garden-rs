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

//! SH1106 132x64 OLED controller on I2C1, with the 128x64 panel it drives.
//!
//! Interprets the command stream (page and column addressing, segment
//! remap, COM scan direction, display on/off) and keeps the display RAM, so
//! [`render_text`] and [`render_pbm`] show what the panel would.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;
use std::sync::Mutex;

use super::i2c::I2cDevice;
use super::lock;

/// Display RAM columns of the controller
const RAM_COLUMNS: usize = 132;
const PAGES: usize = 8;

/// The panel shows the 128 RAM columns centred in the 132 of the controller
pub(super) const PANEL_WIDTH: usize = 128;
pub(super) const PANEL_HEIGHT: usize = PAGES * 8;
const PANEL_FIRST_COLUMN: usize = (RAM_COLUMNS - PANEL_WIDTH) / 2;

/// Control byte: D/C# bit set means the bytes that follow are display data
const CONTROL_DATA: u8 = 0x40;

const DISPLAY_OFF: u8 = 0xAE;
const DISPLAY_ON: u8 = 0xAF;
const SEGMENT_REMAP_OFF: u8 = 0xA0;
const SEGMENT_REMAP_ON: u8 = 0xA1;
const NORMAL_DISPLAY: u8 = 0xA6;
const INVERSE_DISPLAY: u8 = 0xA7;
const COM_SCAN_INCREMENT: u8 = 0xC0;
const COM_SCAN_DECREMENT: u8 = 0xC8;

/// Status byte read back: bit 6 set while the display is off
const STATUS_DISPLAY_OFF: u8 = 0x40;

/// Commands whose second byte is an argument, not a new command
const TWO_BYTE_COMMANDS: [u8; 8] = [
    0x81, // contrast
    0xA8, // multiplex ratio
    0xAD, // DC-DC control
    0xD3, // display offset
    0xD5, // clock divide
    0xD9, // pre-charge period
    0xDA, // COM pins
    0xDB, // VCOMH deselect level
];

struct Controller {
    ram: [[u8; RAM_COLUMNS]; PAGES],
    page: usize,
    column: usize,
    display_on: bool,
    inverse: bool,
    segment_remap: bool,
    com_scan_decrement: bool,
    /// The previous command expects its argument in the next command byte
    awaiting_argument: bool,
}

impl Controller {
    /// Power-on state of the SH1106
    const RESET: Controller = Controller {
        ram: [[0u8; RAM_COLUMNS]; PAGES],
        page: 0,
        column: 0,
        display_on: false,
        inverse: false,
        segment_remap: false,
        com_scan_decrement: false,
        awaiting_argument: false,
    };

    fn command(&mut self, cmd: u8) {
        if self.awaiting_argument {
            self.awaiting_argument = false;
            return;
        }

        match cmd {
            0x00..=0x0F => self.column = ((self.column & 0xF0) | (cmd & 0x0F) as usize) % RAM_COLUMNS,
            0x10..=0x1F => self.column = ((self.column & 0x0F) | (((cmd & 0x0F) as usize) << 4)) % RAM_COLUMNS,
            0xB0..=0xB7 => self.page = (cmd & 0x07) as usize,
            DISPLAY_OFF => self.display_on = false,
            DISPLAY_ON => self.display_on = true,
            SEGMENT_REMAP_OFF => self.segment_remap = false,
            SEGMENT_REMAP_ON => self.segment_remap = true,
            NORMAL_DISPLAY => self.inverse = false,
            INVERSE_DISPLAY => self.inverse = true,
            COM_SCAN_INCREMENT => self.com_scan_decrement = false,
            COM_SCAN_DECREMENT => self.com_scan_decrement = true,
            _ if TWO_BYTE_COMMANDS.contains(&cmd) => self.awaiting_argument = true,
            _ => {}
        }
    }

    fn data(&mut self, byte: u8) {
        self.ram[self.page][self.column] = byte;
        // Past the last column the address wraps to 0: the firmware sets the
        // column once per frame and writes each page of 132 bytes right after
        // the previous one, which only works on the board because of this
        self.column = (self.column + 1) % RAM_COLUMNS;
    }

    /// Whether the panel pixel at (`x`, `y`), from its top-left corner, is lit.
    ///
    /// The panel is mounted so that segment remap and decrementing COM scan
    /// (what the firmware sets at init) give an upright image: turning
    /// either off mirrors it on that axis.
    fn pixel(&self, x: usize, y: usize) -> bool {
        if !self.display_on {
            return false;
        }
        let column = if self.segment_remap {
            PANEL_FIRST_COLUMN + x
        } else {
            RAM_COLUMNS - 1 - PANEL_FIRST_COLUMN - x
        };
        let row = if self.com_scan_decrement { y } else { PANEL_HEIGHT - 1 - y };
        let lit = self.ram[row / 8][column] & (1 << (row % 8)) != 0;
        lit != self.inverse
    }
}

static CONTROLLER: Mutex<Controller> = Mutex::new(Controller::RESET);

/// The SH1106 as seen on the bus; its state is [`CONTROLLER`].
pub(super) struct Sh1106;

impl Sh1106 {
    pub(super) const I2C_ADDRESS: u8 = 0x3C;

    pub(super) fn new() -> Self {
        Self
    }
}

impl I2cDevice for Sh1106 {
    fn write(&mut self, data: &[u8]) {
        let Some((&control, payload)) = data.split_first() else {
            return;
        };

        let mut controller = lock(&CONTROLLER);
        if control & CONTROL_DATA != 0 {
            payload.iter().for_each(|&byte| controller.data(byte));
        } else {
            payload.iter().for_each(|&cmd| controller.command(cmd));
        }
    }

    fn read(&mut self, buffer: &mut [u8]) {
        let status = if lock(&CONTROLLER).display_on { 0 } else { STATUS_DISPLAY_OFF };
        buffer.fill(status);
    }
}

/// The panel in the terminal: two pixel rows per character with half blocks,
/// framed so the edges of the screen are visible.
pub(super) fn render_text() -> String {
    let controller = lock(&CONTROLLER);
    let mut out = String::new();

    let title = if controller.display_on { " SH1106 on " } else { " SH1106 off " };
    let _ = writeln!(out, "+{title:-^width$}+", width = PANEL_WIDTH);
    for y in (0..PANEL_HEIGHT).step_by(2) {
        out.push('|');
        for x in 0..PANEL_WIDTH {
            out.push(match (controller.pixel(x, y), controller.pixel(x, y + 1)) {
                (true, true) => '█',
                (true, false) => '▀',
                (false, true) => '▄',
                (false, false) => ' ',
            });
        }
        out.push_str("|\n");
    }
    let _ = write!(out, "+{}+", "-".repeat(PANEL_WIDTH));
    out
}

/// The panel as a binary PBM (P4) image, lit pixels black.
pub(super) fn render_pbm() -> Vec<u8> {
    let controller = lock(&CONTROLLER);
    let mut out = alloc::format!("P4\n{PANEL_WIDTH} {PANEL_HEIGHT}\n").into_bytes();
    for y in 0..PANEL_HEIGHT {
        for byte_x in (0..PANEL_WIDTH).step_by(8) {
            let byte = (0..8).fold(0u8, |byte, bit| {
                byte | if controller.pixel(byte_x + bit, y) { 0x80 >> bit } else { 0 }
            });
            out.push(byte);
        }
    }
    out
}

/// Number of lit panel pixels, a cheap "something is drawn" check.
pub(super) fn lit_pixels() -> usize {
    let controller = lock(&CONTROLLER);
    (0..PANEL_HEIGHT)
        .flat_map(|y| (0..PANEL_WIDTH).map(move |x| (x, y)))
        .filter(|&(x, y)| controller.pixel(x, y))
        .count()
}
