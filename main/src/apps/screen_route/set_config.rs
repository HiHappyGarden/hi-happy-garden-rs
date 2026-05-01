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

use crate::apps::display::check::Check;
use crate::apps::display::input::Input;
use crate::apps::display::date::Date;
use crate::apps::display::time::Time;

enum FSMState {
    Serial,
    EnableWifi,
    Ssid,
    Passwd,
    Date,
    Time,
    EnableDst,
}

pub struct ScreenSetConfig {
    fsm_state: FSMState,
    serial: Input,
    enable_wifi: Check,
    ssid: Input,
    passwd: Input,
    auth: bool,
    date: Date,
    time: Time,
    enable_dst: Check,
}