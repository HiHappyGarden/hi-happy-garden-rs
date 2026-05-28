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

use osal_rs::utils::Bytes;

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::drivers::wifi::Auth;
use crate::traits::screen::{ScreenSelections, screen_selections_new};

pub(super) fn auth_as_bytes(auth: Auth) -> Bytes<DISPLAY_INPUT_MAX_SIZE> {
    match auth {
        Auth::Open => Bytes::from_str("OPEN"),
        Auth::Wpa => Bytes::from_str("WPA"),
        Auth::Wpa2 => Bytes::from_str("WPA2"),
        Auth::Wpa2Mixed => Bytes::from_str("WPA2 MIXED"),
        Auth::Wpa3 => Bytes::from_str("WPA3"),
        Auth::Wpa2Wpa3 => Bytes::from_str("WPA3/WPA2"),
        _ => Bytes::default(),
    }
}

pub(super) fn fill_auth_selections(selected: Auth) -> ScreenSelections {
    let mut selections = screen_selections_new();
    selections[0] = (auth_as_bytes(Auth::Open), selected == Auth::Open);
    selections[1] = (auth_as_bytes(Auth::Wpa), selected == Auth::Wpa);
    selections[2] = (auth_as_bytes(Auth::Wpa2), selected == Auth::Wpa2);
    selections[3] = (auth_as_bytes(Auth::Wpa2Mixed), selected == Auth::Wpa2Mixed);
    selections[4] = (auth_as_bytes(Auth::Wpa3), selected == Auth::Wpa3);
    selections[5] = (auth_as_bytes(Auth::Wpa2Wpa3), selected == Auth::Wpa2Wpa3);
    selections
}

pub(super) fn selected_auth_from_selections(selections: &ScreenSelections) -> Auth {
    selections
        .iter()
        .find(|entry| entry.1)
        .map(|entry| match entry.0.as_str() {
            "OPEN" => Auth::Open,
            "WPA" => Auth::Wpa,
            "WPA2" => Auth::Wpa2,
            "WPA2 MIXED" => Auth::Wpa2Mixed,
            "WPA3" => Auth::Wpa3,
            "WPA3/WPA2" => Auth::Wpa2Wpa3,
            _ => Auth::Open,
        })
        .unwrap_or(Auth::Open)
}