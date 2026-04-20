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

pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}


pub trait RgbLed {
    fn set(&self, red: u8, green: u8, blue: u8);    

    #[inline]
    fn set_color(&self, color: &Color) {
        self.set(color.red, color.green, color.blue);
    }

    #[inline]
    fn set_red(&self, red: u8) {
        self.set(red, 0, 0);
    }

    #[inline]
    fn set_green(&self, green: u8) {
        self.set(0, green, 0);
    }

    #[inline]
    fn set_blue(&self, blue: u8) {
        self.set(0, 0, blue);
    }

    #[inline]
    fn turn_off(&self) {
        self.set(0, 0, 0);
    }
}
