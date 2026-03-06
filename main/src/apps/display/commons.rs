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
use osal_rs::os::{Mutex, MutexFn};
use osal_rs::utils::Result;

use crate::traits::lcd_display::{LCDDisplayFn, LCDWriteMode};


pub fn clean_context<T>(lcd: &mut Arc<Mutex<T>>) -> Result<()> 
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{
    let mut lcd = lcd.lock().unwrap();
    let (display_width, display_height) = lcd.get_size();
    let y_start = lcd.get_header_height();
    lcd.draw_rect(0, y_start, display_width, display_height - y_start, LCDWriteMode::REMOVE)
}
