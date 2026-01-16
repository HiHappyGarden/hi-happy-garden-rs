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

#![allow(dead_code)]

use osal_rs::utils::Result;


#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum LCDWriteMode
{
    /// sets pixel on regardless of its state
    ADD = 0,
    /// sets pixel off regardless of its state
    REMOVE = 1,
    /// inverts pixel, so 1->0 or 0->1
    INVERT = 2,
}


/// Trait for LCD display operations
pub trait LCDDisplayFn : Sync + Send {
    /// Draw the buffer to the display
    fn draw(&mut self) -> Result<()>;
    
    /// Clear the display buffer
    fn clear(&mut self);
    
    /// Draw a single pixel at the specified position
    fn draw_pixel(&mut self, x: u8, y: u8, write_mode: LCDWriteMode) -> Result<()>;
    
    /// Draw a bitmap image
    fn draw_bitmap_image(&mut self, x: u8, y: u8, width: u8, height: u8, image: &[u8], write_mode: LCDWriteMode) -> Result<()>;
    
    /// Draw a rectangle
    fn draw_rect(&mut self, x: u8, y: u8, width: u8, height: u8, write_mode: LCDWriteMode) -> Result<()>;
    
    /// Draw a single character
    fn draw_char(&mut self, c: char, x: u8, y: u8, font: &[u8]) -> Result<()>;
    
    /// Draw a string
    fn draw_str(&mut self, str: &str, x: u8, y: u8, font: &[u8], font_size: u32) -> Result<()>;
    
    /// Invert the display orientation
    fn invert_orientation(&mut self) -> Result<()>;
    
    /// Set the display contrast
    fn set_contrast(&self, contrast: u8) -> Result<()>;
    
    /// Turn off the display
    fn turn_off(&mut self) -> Result<()>;
    
    /// Turn on the display
    fn turn_on(&mut self) -> Result<()>;
}
