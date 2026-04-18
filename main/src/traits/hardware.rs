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


use alloc::sync::Arc;
use osal_rs::os::Mutex;

use crate::traits::relays::Relays as RelaysFn;
use crate::traits::button::OnClickable;
use crate::traits::encoder::OnRotatableAndClickable;
use crate::traits::rtc::RTC;
use crate::traits::rx_tx::{SetOnReceive, SetTransmit};
use crate::traits::wifi::SetOnWifiChangeStatus;

pub trait HardwareFn<'a> : RelaysFn + SetOnWifiChangeStatus<'a> + SetOnReceive<'a> + SetTransmit {

    #![allow(dead_code)]
    const SAMPLES: u8 = 20;

    #[inline(always)]
    fn temperature_conversion(value: u32) -> f32 {
        let voltage = 3.3f32 / (1 << 12) as f32 * value as f32;
        27.0f32 - (voltage - 0.706f32) / 0.001721f32
    }

    fn set_button_handler(&mut self, clicclable: &'a dyn OnClickable);

    fn set_encoder_handler(&mut self, rotate_and_click: &'a dyn OnRotatableAndClickable);

    fn get_temperature(&self) -> f32;

    fn get_unique_id() -> [u8; 8];

    fn get_rtc(&self) -> Arc<Mutex<dyn RTC + 'a>>;
}

