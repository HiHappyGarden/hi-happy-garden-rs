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

use alloc::sync::Arc;
use osal_rs::os::types::EventBits;
use osal_rs::os::{Mutex, MutexFn};
use osal_rs::utils::{AsSyncStr, Result};

use crate::apps::display::commons::clean_context;
use crate::apps::signals::display::DisplayFlag;
use crate::assets::font_8x8::FONT_8X8;
use crate::drivers::date_time::DateTime;
use crate::traits::lcd_display::LCDDisplayFn;

pub(super) struct Check<T> 
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{
    lcd: Arc<Mutex<T>>,
}

impl<T> Check<T> 
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{


    pub(super) fn new(lcd: Arc<Mutex<T>>) -> Self {
        Self {
            lcd,
        }
    }

    pub(super) fn draw(&mut self, signals: &mut EventBits, _date_time: &DateTime, text: &impl AsSyncStr, _check: bool) -> Result<()> {
        clean_context(&mut self.lcd)?;
        
        let mut lcd = self.lcd.lock()?;

        lcd.draw_str(text.as_str(), 2, 30, &FONT_8X8)?;

        *signals |= DisplayFlag::Draw as u32; // Set the flag to indicate that the display should be redrawn 
        Ok(())
    }
}

