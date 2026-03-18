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
use osal_rs::os::Mutex;

use crate::traits::lcd_display::LCDDisplayFn;

pub struct Input<T>
where
    T: LCDDisplayFn + Sync + Send + Clone + 'static,
{
    lcd: Arc<Mutex<T>>,
}

impl<T> Input<T>
where
    T: LCDDisplayFn + Sync + Send + Clone + 'static,
{
    pub(super) fn new(lcd: Arc<Mutex<T>>) -> Self {
        Self { lcd }
    }

    pub(super) fn draw(
        &mut self,
        signals: &mut EventBits,
        current_date_time: &DateTime,
        text: &impl AsSyncStr,
        date_time: Option<DateTime>,
        callback: Option<fn(Option<DateTime>)>,
    ) -> Result<()> {
        self.0.draw(signals, current_date_time, text, date_time, callback)
    }
}