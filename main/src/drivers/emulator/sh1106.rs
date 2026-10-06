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

//! SH1106 132x64 OLED controller on I2C1.
//!
//! Interprets the command stream (page and column addressing, display
//! on/off) and keeps the display RAM, so what the firmware draws can be
//! inspected.

use super::i2c::I2cDevice;

const COLUMNS: usize = 132;
const PAGES: usize = 8;

/// Control byte: D/C# bit set means the bytes that follow are display data
const CONTROL_DATA: u8 = 0x40;

const DISPLAY_OFF: u8 = 0xAE;
const DISPLAY_ON: u8 = 0xAF;

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

pub(super) struct Sh1106 {
    ram: [[u8; COLUMNS]; PAGES],
    page: usize,
    column: usize,
    display_on: bool,
    /// The previous command expects its argument in the next command byte
    awaiting_argument: bool,
}

impl Sh1106 {
    pub(super) const I2C_ADDRESS: u8 = 0x3C;

    pub(super) fn new() -> Self {
        Self {
            ram: [[0u8; COLUMNS]; PAGES],
            page: 0,
            column: 0,
            display_on: false,
            awaiting_argument: false,
        }
    }

    fn command(&mut self, cmd: u8) {
        if self.awaiting_argument {
            self.awaiting_argument = false;
            return;
        }

        match cmd {
            0x00..=0x0F => self.column = (self.column & 0xF0) | (cmd & 0x0F) as usize,
            0x10..=0x1F => self.column = (self.column & 0x0F) | (((cmd & 0x0F) as usize) << 4),
            0xB0..=0xB7 => self.page = (cmd & 0x07) as usize,
            DISPLAY_OFF => self.display_on = false,
            DISPLAY_ON => self.display_on = true,
            _ if TWO_BYTE_COMMANDS.contains(&cmd) => self.awaiting_argument = true,
            _ => {}
        }
    }

    fn data(&mut self, byte: u8) {
        // The column address stops at the last column instead of wrapping
        if self.column < COLUMNS {
            self.ram[self.page][self.column] = byte;
            self.column += 1;
        }
    }
}

impl I2cDevice for Sh1106 {
    fn write(&mut self, data: &[u8]) {
        let Some((&control, payload)) = data.split_first() else {
            return;
        };

        if control & CONTROL_DATA != 0 {
            payload.iter().for_each(|&byte| self.data(byte));
        } else {
            payload.iter().for_each(|&cmd| self.command(cmd));
        }
    }

    fn read(&mut self, buffer: &mut [u8]) {
        let status = if self.display_on { 0 } else { STATUS_DISPLAY_OFF };
        buffer.fill(status);
    }
}
