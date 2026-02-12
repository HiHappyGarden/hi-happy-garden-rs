/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
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

use crate::drivers::i2c::I2C;
use crate::drivers::platform::{I2C_BAUDRATE, I2C1_INSTANCE};
use crate::traits::lcd_display::{LCDDisplayFn, LCDWriteMode};
use crate::traits::state::Initializable;
use osal_rs::log_info;
use osal_rs::utils::{Error, Result};
use sh1106_commands::*;
use crate::drivers::plt::ffi::pico_error_codes::PICO_ERROR_GENERIC;

const APP_TAG: &str = "LCDSH1106";
const ASCII_TABLE_START_AT_IDX: u8 = 32;

#[allow(unused)]
pub(super) mod sh1106_commands {
    pub(super) const CONTRAST: u8 = 0x80;
    pub(super) const ENTRTY_DISPLAY_OFF: u8 = 0xA4;
    pub(super) const ENTRTY_DISPLAY_ON: u8 = 0xA5;
    pub(super) const INVERTED_OFF: u8 = 0xA6;
    pub(super) const INVERTED_ON: u8 = 0xA7;
    pub(super) const DISPLAY_OFF: u8 = 0xAE;
    pub(super) const DISPLAY_ON: u8 = 0xAF;
    pub(super) const DISPLAY_OFFSET: u8 = 0xD3;
    pub(super) const COM_PADS: u8 = 0xDA;
    pub(super) const VCOM_SET: u8 = 0xDB;
    pub(super) const DISPLAY_PRESCALER: u8 = 0xD5;
    pub(super) const PRE_CHARGE: u8 = 0xD9;
    pub(super) const LOW_COLUMN: u8 = 0x00;
    pub(super) const HIGH_COLUMN: u8 = 0x10;
    pub(super) const START_LINE: u8 = 0x40;
    pub(super) const MEMORY_MODE: u8 = 0x20;
    pub(super) const MEMORY_MODE_HORIZONTAL: u8 = 0x00;
    pub(super) const MEMORY_MODE_VERTICAL: u8 = 0x01;
    pub(super) const MEMORY_MODE_PAGE: u8 = 0x10;
    pub(super) const COLUMN_ADDR: u8 = 0x21;
    pub(super) const PAGE_ADDR: u8 = 0xB0;
    pub(super) const VERTICAL_FLIP_OFF: u8 = 0xC0;
    pub(super) const VERTICAL_FLIP_ON: u8 = 0xC8;
    pub(super) const COLUMN_REMAP_OFF: u8 = 0xA0;
    pub(super) const COLUMN_REMAP_ON: u8 = 0xA1;
    pub(super) const CHARGE_PUMP: u8 = 0x8B;
    pub(super) const EXTERNAL_VCC: u8 = 0x1;
    pub(super) const SWITCH_CAP_VCC: u8 = 0x2;
    pub(super) const SET_CONTRAST: u8 = 0x81;
    pub(super) const MULTIPLEX: u8 = 0x3F;
}

#[derive(Clone)]
pub struct LCDSH1106 {
    i2c: I2C<{I2C1_INSTANCE}, {I2C_BAUDRATE}>,
    buffer: [u8; (LCDSH1106::WIDTH as usize) * (LCDSH1106::HEIGHT as usize)],
    orientation: bool,
    #[allow(unused)]
    turned_on: bool,
}

unsafe impl Send for LCDSH1106 {}
unsafe impl Sync for LCDSH1106 {}

impl Initializable  for LCDSH1106 {
    fn init(&mut self) -> Result<()> {

        log_info!(APP_TAG, "Init LCDSH1106");

        let init_sequence: [u8; 25] = [
            DISPLAY_OFF,                // Turn off display during initialization
            DISPLAY_PRESCALER,          // Set display clock divide ratio/oscillator frequency
            CONTRAST,                   // Set contrast control register (command)
            MULTIPLEX,                  // Set multiplex ratio (command)
            0x3F,                       // Set multiplex ratio value (1-64)
            DISPLAY_OFFSET,             // Set display offset (command)
            0x00,                       // Display offset value: no offset
            START_LINE | 0x00,          // Set display start line to 0
            CHARGE_PUMP,                // Charge pump setting (command)
            0x14,                       // Enable charge pump
            MEMORY_MODE,                // Set memory addressing mode (command)
            MEMORY_MODE_HORIZONTAL,     // Use horizontal addressing mode
            COLUMN_REMAP_OFF,           // No column remap - origin at top-left
            VERTICAL_FLIP_OFF,          // No vertical flip - origin at top-left
            COM_PADS,                   // Set COM pins hardware configuration (command)
            0x12,                       // Alternative COM pin configuration
            CONTRAST,                   // Set contrast control (command)
            0xFF,                       // Maximum contrast value
            PRE_CHARGE,                 // Set pre-charge period (command)
            0xF1,                       // Pre-charge period value
            VCOM_SET,                   // Set VCOMH deselect level (command)
            0x40,                       // VCOMH deselect level value
            ENTRTY_DISPLAY_OFF,         // Resume to RAM content display
            INVERTED_OFF,               // Set normal display (not inverted)
            DISPLAY_ON,                 // Turn on display
        ];

        let _ = self.send_cmds(&init_sequence);

        self.clear();

        self.draw()?;
        
        Ok(())
    }
}

impl LCDDisplayFn for LCDSH1106 {
    fn draw(&mut self) -> Result<()> {
        let _ = self.send_cmd(LOW_COLUMN);
        let _ = self.send_cmd(HIGH_COLUMN);

        for page in 0..LCDSH1106::HEIGHT {
            self.send_cmd_with_data(PAGE_ADDR, page)?;

            let page = page as usize;
            let start = page * LCDSH1106::WIDTH as usize;
            let end = start + LCDSH1106::WIDTH as usize;
 
            self.send_data(&self.buffer[start .. end])?;
        }
        Ok(())
    }

    fn clear(&mut self) {
        for byte in self.buffer.iter_mut() {
            *byte = 0x00;
        }
    }

    fn draw_pixel(&mut self, x: u8, y: u8, write_mode: LCDWriteMode)  -> Result<()>  {
        use LCDWriteMode::*;
        if (x >= LCDSH1106::WIDTH) || (y >= (LCDSH1106::HEIGHT * 8) ) {
            return Err(Error::OutOfIndex);
        }
        
        let page = y / 8;
        let bit = y % 8;
        let idx = (page as usize * LCDSH1106::WIDTH as usize) + x as usize;

        match write_mode {
            ADD => self.buffer[idx] |= 1 << bit,
            REMOVE =>  self.buffer[idx] &= !(1 << bit),
            INVERT => self.buffer[idx] ^= 1 << bit,
        }
        Ok(())
    }

    fn draw_bitmap_image(&mut self, x: u8, y: u8, width: u8, height: u8, image: &[u8], _write_mode: LCDWriteMode) -> Result<()> {
        if image.len() ==0 || width * height != image.len() as u8 {
            return Err(Error::InvalidType);
        }

        let mut idx = 0usize;


        for h in 0..height {
            for w in 0..width {
                let x = x + w;
                let y = y + (height - 1 - h);
                if image[idx] != 0 {
                    let _ = self.draw_pixel(x, y, LCDWriteMode::ADD);
                } else {
                    let _ = self.draw_pixel(x, y, LCDWriteMode::REMOVE);
                }
                idx += 1;
            }
        }
        Ok(())
    }

    fn draw_rect(&mut self, x: u8, y: u8, width: u8, height: u8, write_mode: LCDWriteMode) -> Result<()> {
        for w in 0..width {
            for h in 0..height {
                self.draw_pixel(x + w, y + h, write_mode)?;
            }
        }
        Ok(())
    }

    fn draw_char(&mut self, c: char ,  x: u8, y: u8, font: &[u8]) -> Result<()> {
        if font.len() == 0 {
            return Err(Error::Empty)
        }

        let width = font[0];
        let height = (font[1] / 8) + (font[1] % 8 != 0) as u8;
        let single_font_size = (width * height) as usize;

        if (font.len() - 2) % width as usize != 0 {
            return Err(Error::InvalidType);
        }

        let c_offset = ( (c as usize - ASCII_TABLE_START_AT_IDX as usize) * (width * height) as usize ) + 2;

        let mut w = 0;
        let mut h = 0;
        for idx in 0..single_font_size
        {
            if w >= width {
                h += 1;
                w = 0;
            }

            if h == height {
                break;
            }

            if w < width {
                for bit in 0..8 {
                    let x = x + w;
                    let y = y + (h * 8) + (7 - bit);
                    if font[c_offset as usize + idx] & (1 << bit) > 0 {
                        self.draw_pixel(x, y, LCDWriteMode::ADD)?;
                    } else {
                        self.draw_pixel(x, y, LCDWriteMode::REMOVE)?;
                    }
                }
                w += 1;
            }
        }

        Ok(())
    }

    fn draw_str(&mut self, str: &str, x: u8, y: u8, font: &[u8]) -> Result<()> {
        if str.is_empty() || font.is_empty() {
            return Err(Error::Empty);
        }

        let width = font[0];

        for (i, c) in str.chars().enumerate() {
            self.draw_char(c, x + (width as u8 * i as u8), y, font)?;
        }
        Ok(())
    }

    fn invert_orientation(&mut self) -> Result<()> {
        self.orientation = !self.orientation;
        if self.orientation {
            self.send_cmd(VERTICAL_FLIP_ON)?;
            self.send_cmd(COLUMN_REMAP_OFF)?;
        } else {
            self.send_cmd(VERTICAL_FLIP_OFF)?;
            self.send_cmd(COLUMN_REMAP_ON)?;
        }
        Ok(())
    }

    fn set_contrast(&self, contrast: u8) -> Result<()> {
        self.send_cmd(SET_CONTRAST)?;
        self.send_cmd(contrast)?;
        Ok(())
    }

    fn turn_off(&mut self) -> Result<()> {
        if !self.turned_on {
            return Ok(());
        }
        self.turned_on = false;
        self.send_cmd(DISPLAY_OFF)?;
        Ok(())
    }

    fn turn_on(&mut self) -> Result<()> {
        if self.turned_on {
            return Ok(());
        }
        self.turned_on = true;
        self.send_cmd(DISPLAY_ON)?;
        Ok(())
    }
}

impl LCDSH1106 {

    pub const I2C_ADDRESS: u8 = 0x3C;
    pub const WIDTH: u8 = 132;
    pub const HEIGHT: u8 = 8; // in pages (8 pixels each)


    pub fn new(i2c: I2C<{I2C1_INSTANCE}, {I2C_BAUDRATE}>) -> Self {
        Self { 
            i2c,
            buffer: [0u8; (LCDSH1106::WIDTH as usize) * (LCDSH1106::HEIGHT as usize)],
            orientation: true,
            turned_on: true,
        }
    }

    fn send_cmd(&self, cmd: u8) -> Result<()>{
        let data = [0x00, cmd]; // Control byte 0x00 for commands
        self.i2c.write(&data)
    }

    fn send_cmd_with_data(&self, cmd: u8, data: u8) -> Result<()> {
        self.send_cmd(cmd | data)
    }

    fn send_cmds(&self, cmds: &[u8]) -> Result<()> {
        for cmd in cmds {
            self.send_cmd(*cmd)?;
        }
        Ok(())
    }

    fn send_data(&self, data: &[u8]) -> Result<()> {
        let mut buffer = [0u8; LCDSH1106::WIDTH as usize + 1];
        buffer[0] = START_LINE;

        let len = data.len().min(LCDSH1106::WIDTH as usize);
        buffer[1..=len].copy_from_slice(&data[..len]);
        
        self.i2c.write(&buffer)
    }
}