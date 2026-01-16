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

#![allow(unused)]

use crate::drivers::i2c::I2C;
use crate::traits::lcd_display::{LCDDisplayFn, LCDWriteMode};
use crate::traits::state::Initializable;
use osal_rs::log_info;
use osal_rs::utils::{Error, Result};
use sh1106_commands::*;
use crate::drivers::plt::ffi::pico_error_codes::PICO_ERROR_GENERIC;

const APP_TAG: &str = "LCDSH1106";
const ASCII_TABLE_START_AT_IDX: u8 = 32;

pub(super) mod sh1106_commands {
    pub(super) const CONTRAST: u8 = 0x80;
    pub(super) const DISPLAY_ALL_ON_RESUME: u8 = 0xA4;
    pub(super) const DISPLAY_ALL_ON: u8 = 0xA5;
    pub(super) const INVERTED_OFF: u8 = 0xA6;
    pub(super) const INVERTED_ON: u8 = 0xA7;
    pub(super) const DISPLAY_OFF: u8 = 0xAE;
    pub(super) const DISPLAY_ON: u8 = 0xAF;
    pub(super) const DISPLAY_OFFSET: u8 = 0xD3;
    pub(super) const COM_PADS: u8 = 0xDA;
    pub(super) const VCOM_DETECT: u8 = 0xDB;
    pub(super) const DISPLAY_CLOCK_DIV: u8 = 0xD5;
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
    i2c: I2C<{LCDSH1106::I2C_ADDRESS}>,
    buffer: [u8; (LCDSH1106::WIDTH as usize) * (LCDSH1106::HEIGHT as usize)],
    orientation: bool,
    turned_on: bool,
}

unsafe impl Send for LCDSH1106 {}
unsafe impl Sync for LCDSH1106 {}

impl Initializable  for LCDSH1106 {
    fn init(&mut self) -> Result<()> {

        self.i2c.init()?;

        log_info!(APP_TAG, "Init LCDSH1106");

        let init_sequence: [u8; 25] = [
            DISPLAY_OFF,
            DISPLAY_CLOCK_DIV,
            CONTRAST,
            MULTIPLEX,
            MULTIPLEX,
            DISPLAY_OFFSET,
            0x00,
            START_LINE | 0x00,
            CHARGE_PUMP,
            0x14,
            MEMORY_MODE,
            MEMORY_MODE_HORIZONTAL,
            COLUMN_REMAP_ON,
            VERTICAL_FLIP_OFF,
            COM_PADS,
            0x12,
            CONTRAST,
            0xFF,
            PRE_CHARGE,
            0xF1,
            VCOM_DETECT,
            0x40,
            DISPLAY_ALL_ON_RESUME,
            INVERTED_OFF,
            DISPLAY_ON,
        ];

        self.send_cmds(&init_sequence);

        self.clear();

        self.draw()?;
        
        Ok(())
    }
}

impl LCDDisplayFn for LCDSH1106 {
    fn draw(&mut self) -> Result<()> {
        self.send_cmd(LOW_COLUMN);
        self.send_cmd(HIGH_COLUMN);

        for page in 0..LCDSH1106::HEIGHT{
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
        let idx : usize = (page * LCDSH1106::WIDTH) as usize + x as usize;

        match write_mode {
            ADD => self.buffer[idx] |= (1 << bit),
            REMOVE =>  self.buffer[idx] &= !(1 << bit),
            INVERT => self.buffer[idx] ^= (1 << bit),
        }
        Ok(())
    }

    fn draw_bitmap_image(&mut self, x: u8, y: u8, width: u8, height: u8, image: &[u8], write_mode: LCDWriteMode) -> Result<()> {
        if(image.len() ==0 || width * height != image.len() as u8)
        {
            return Err(Error::InvalidType);
        }

        let mut idx = 0usize;


        for h in 0..height
        {
            for w in 0..width
            {
                if(image[idx] != 0)
                {
                    self.draw_pixel(x + w, y + h, LCDWriteMode::ADD);
                }
                else
                {
                    self.draw_pixel(x + w, y + h, LCDWriteMode::REMOVE);
                }
                idx += 1;
            }
        }
        Ok(())
    }

    fn draw_rect(&mut self, x: u8, y: u8, width: u8, height: u8, write_mode: LCDWriteMode) -> Result<()> {
        for w in 0..width
        {
            for h in 0..height
            {
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

        if((font.len() - 2) % width as usize != 0) {
            return Err(Error::InvalidType);
        }

        let c_offset = ( (c as u8 - ASCII_TABLE_START_AT_IDX) * (width * height) ) + 2;

        let mut w = 0;
        let mut h = 0;
        for idx in 0..single_font_size
        {
            if(w >= width)
            {
                h += 1;
                w = 0;
            }

            if(h == height)
            {
                break;
            }

            if(w < width)
            {
                for bit in 0..8
                {
                    if (font[c_offset as usize + idx] & (1 << bit) > 0)
                    {
                        self.draw_pixel(x + w, y + (h * 8) + bit, LCDWriteMode::ADD)?;
                    }
                    else
                    {
                        self.draw_pixel(x + w, y + (h * 8) + bit, LCDWriteMode::REMOVE)?;
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


    pub fn new(i2c: I2C<{LCDSH1106::I2C_ADDRESS}>) -> Self {
        Self { 
            i2c,
            buffer: [0u8; (LCDSH1106::WIDTH as usize) * (LCDSH1106::HEIGHT as usize)],
            orientation: true,
            turned_on: true,
        }
    }

    fn send_cmd(&self, cmd: u8) -> Result<()>{
        let data = [0x00, cmd]; // Control byte 0x00 for commands
        if self.i2c.write(&data) == PICO_ERROR_GENERIC as i32 {
            Err(Error::WriteError)
        } else {
            Ok(())
        }
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
        
        if self.i2c.write(&buffer) == PICO_ERROR_GENERIC as i32 {
            Err(Error::WriteError)
        } else {
            Ok(())
        }
    }
}