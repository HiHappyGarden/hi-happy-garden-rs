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

#![allow(dead_code)]

///! Display signal for display updates and interactions.

use crate::define_signal;

pub enum DisplayFlag {
    None = 0x00,
    ButtonPressed = 0x01,
    ButtonReleased = 0x02,
    EncoderButtonPressed = 0x04,
    EncoderButtonReleased = 0x08,
    EncoderRotatedClockwise = 0x10,
    EncoderRotatedCounterClockwise = 0x20,
    WifiStatusUnknown = 0x40,
    WifiStatusExcellent = 0x80,
    WifiStatusGood = 0xC0,
    WifiStatusFair = 0x01_00,
    WifiStatusWeak = 0x01_40,
    WifiStatusNoSignal = 0x01_80,
    Draw = 0x00_80_00_00, // Special flag to indicate that the display should be redrawn
}

impl From<u32> for DisplayFlag {
    fn from(value: u32) -> Self {
        use DisplayFlag::*;
        match value {
            0x01 => ButtonPressed,
            0x02 => ButtonReleased,
            0x04 => EncoderButtonPressed,
            0x08 => EncoderButtonReleased,
            0x10 => EncoderRotatedClockwise,
            0x20 => EncoderRotatedCounterClockwise,
            0x40 => WifiStatusUnknown,
            0x80 => WifiStatusExcellent,
            0xC0 => WifiStatusGood,
            0x01_00 => WifiStatusFair,
            0x01_40 => WifiStatusWeak,
            0x01_80 => WifiStatusNoSignal,
            0x00_80_00_00 => Draw,
            _ => None, // Default case, can be adjusted as needed
        }
    }
}

impl From<DisplayFlag> for u32 {
    fn from(flag: DisplayFlag) -> Self {
        flag as u32
    }
}

define_signal!(DisplaySignal, DISPLAY_SIGNAL);

