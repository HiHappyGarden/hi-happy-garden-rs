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
use alloc::boxed::Box;

use crate::traits::button::{ButtonCallback, ButtonState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncoderDirection {
    Clockwise,
    CounterClockwise,
}

pub type EncoderCallback = dyn Fn(EncoderDirection, i32) + Send + Sync;

pub trait OnRotatable {
    fn set_on_rotate(&mut self, callback: Box<EncoderCallback>);
    fn get_position(&self) -> i32;
    fn set_on_click(&mut self, callback: Box<ButtonCallback>);
    fn get_state(&self) -> ButtonState;
}


 