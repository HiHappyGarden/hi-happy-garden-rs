/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
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

use osal_rs::utils::Result;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Source {
    Uart,
    Mqtt,
    Display
}


/// Trait for receiving data callbacks
/// 
/// The `source` parameter accepts any string reference with lifetime 'a,
/// making the trait more flexible while maintaining safety
pub trait OnReceive : Send + Sync {
    fn on_receive(&self, source: Source, data: &[u8]) -> Result<()>;
}


pub trait SetOnReceive<'a> {
    fn set_on_receive(&mut self, on_receive: &'a dyn OnReceive);
}

pub trait SetTransmit {
    fn transmit(&self, data: &[u8]) -> usize;
}