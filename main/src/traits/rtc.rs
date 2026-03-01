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

use osal_rs::utils::Result;

use crate::drivers::date_time::DateTime;

 #[allow(dead_code)]
 pub trait RTC: Send + Sync {
   fn set_timestamp(&self, timestamp: i64) -> Result<()>;

   fn get_timestamp(&self) -> Result<i64>;

   fn is_to_synch(&self) -> bool;

   fn timestamp_to_datetime(&self, locale: bool) -> Result<DateTime>  {
         let timestamp = self.get_timestamp().unwrap_or(0);
         DateTime::from_timestamp_locale(timestamp, locale)
   }
   
 }