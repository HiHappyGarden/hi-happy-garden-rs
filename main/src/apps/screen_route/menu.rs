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
use osal_rs::os::types::EventBits;
use osal_rs::utils::{Bytes, Result};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::display::text::Text;
use crate::apps::screen_route::ScreenId;
use crate::apps::signals::display::DisplayFlag;
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::rtc::RTC;
use crate::traits::screen::{Nav, Screen, ScreenParam, ScreenRoute};

/// Entries of the main menu, in rotation order.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum MenuItem {
    Info,
    DateTime,
    DaylightSavingTime,
    Wifi,
    User,
    Sprinkler,
}

impl MenuItem {
    const ORDER: [MenuItem; 6] = [
        MenuItem::Info,
        MenuItem::DateTime,
        MenuItem::DaylightSavingTime,
        MenuItem::Wifi,
        MenuItem::User,
        MenuItem::Sprinkler,
    ];

    fn label(self) -> &'static str {
        match self {
            MenuItem::Info               => "Info",
            MenuItem::DateTime           => "Date Time",
            MenuItem::DaylightSavingTime => "Daylight Saving Time",
            MenuItem::Wifi               => "Wifi",
            MenuItem::User               => "User",
            MenuItem::Sprinkler          => "Sprinkler",
        }
    }

    fn index(self) -> usize {
        match self {
            MenuItem::Info               => 0,
            MenuItem::DateTime           => 1,
            MenuItem::DaylightSavingTime => 2,
            MenuItem::Wifi               => 3,
            MenuItem::User               => 4,
            MenuItem::Sprinkler          => 5,
        }
    }

    fn next(self) -> Self {
        Self::ORDER[(self.index() + 1) % Self::ORDER.len()]
    }

    fn previous(self) -> Self {
        Self::ORDER[(self.index() + Self::ORDER.len() - 1) % Self::ORDER.len()]
    }
}

impl From<MenuItem> for ScreenId {
    fn from(item: MenuItem) -> Self {
        match item {
            MenuItem::Info               => ScreenId::Info,
            MenuItem::DateTime           => ScreenId::DateTime,
            MenuItem::DaylightSavingTime => ScreenId::DaylightSavingTime,
            MenuItem::Wifi               => ScreenId::Wifi,
            MenuItem::User               => ScreenId::User,
            MenuItem::Sprinkler          => ScreenId::Sprinkler,
        }
    }
}

pub(super) struct ScreenMenu {
    item: MenuItem,
    text: Text,
}

impl ScreenRoute<ScreenId> for ScreenMenu {
    fn id(&self) -> ScreenId {
        ScreenId::Menu
    }

    fn draw(&mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        _status_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>) -> Result<Nav<ScreenId>> {

        self.update_input(display_signal);

        self.text.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str(self.item.label()),
            ScreenParam::<u16>::default(),
            None,
        )?;

        if *display_signal & DisplayFlag::EncoderButtonReleased as u32 != 0 {
            // The router builds the screen, so the menu stays independent from it.
            Ok(Nav::PushId(self.item.into()))
        } else {
            Ok(Nav::Stay)
        }
    }

    fn requires_auth(&self) -> bool {
        false
    }
}

impl ScreenMenu {
    pub(super) fn new() -> Self {
        Self::with_item(MenuItem::Info)
    }

    pub(super) fn with_item(item: MenuItem) -> Self {
        Self {
            item,
            text: Text::new(),
        }
    }

    fn update_input(&mut self, signal: &mut EventBits) {
        if *signal & DisplayFlag::EncoderRotatedClockwise as u32 != 0 {
            self.item = self.item.next();
            *signal |= DisplayFlag::Draw as u32; // Set the flag to indicate that the display should be redrawn
        } else if *signal & DisplayFlag::EncoderRotatedCounterClockwise as u32 != 0 {
            self.item = self.item.previous();
            *signal |= DisplayFlag::Draw as u32; // Set the flag to indicate that the display should be redrawn
        }
    }
}
