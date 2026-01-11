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

use osal_rs::utils::OsalRsBool;

use crate::drivers::platform::GpioPeripheral;


pub trait Relays {
  fn set_relay_state(&self, relay_index: GpioPeripheral, state: bool) -> OsalRsBool;

  fn turn_off_all_relays(&self) {
      self.set_relay_state(GpioPeripheral::Relay1, false);
      self.set_relay_state(GpioPeripheral::Relay2, false);
      self.set_relay_state(GpioPeripheral::Relay3, false);
      self.set_relay_state(GpioPeripheral::Relay4, false);
  }
}