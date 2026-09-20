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

mod info;
mod menu;
mod set_config;



use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::sync::Arc;


use crate::apps::config::Config;
use crate::apps::signals::status::StatusFlag;
use crate::apps::screen_route::set_config::ScreenSetConfig;
use crate::apps::screen_route::menu::ScreenMenu;
use crate::traits::screen::{Nav, ScreenRoute as ScreenRouteFn};
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::rtc::RTC;
use osal_rs::os::Mutex;
use osal_rs::os::types::EventBits;
use osal_rs::utils::Result;

mod commons;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(in crate::apps) enum ScreenId {
    SetConfig,
    Login,
    Menu,
    Info,
    DateTime,
    DaylightSavingTime,
    Wifi,
    User,
    Sprinkler,
    SprinklerSchedule,
    SprinklerZone,
}


 pub(in crate::apps) struct ScreenRoute {
    config: &'static mut Config,
    stack: Vec<Box<dyn ScreenRouteFn<ScreenId>>>,
    check_staus_counter: u8,
    has_local_user: bool,
}

impl ScreenRoute {
    const CHECK_STATUS_THRESHOLD: u8 = 5;

    pub(super) fn new() -> Self {
        Self {
            config: Config::shared(),
            stack: Vec::new(),
            check_staus_counter: 0,
            has_local_user: false,
        }
    }

    fn handle_init(&mut self, status_signal: &mut EventBits) {
        if StatusFlag::CheckConfig.check_signal(*status_signal) {
            self.check_staus_counter += 1;
            if self.check_staus_counter >= Self::CHECK_STATUS_THRESHOLD {
                self.stack.push(Box::new(ScreenSetConfig::new()));
                self.check_staus_counter = 0;
            }
        } else if StatusFlag::Ready.check_signal(*status_signal) {
            self.check_staus_counter += 1;
            if self.check_staus_counter >= Self::CHECK_STATUS_THRESHOLD {
                self.stack.push(Box::new(ScreenMenu::new()));
                self.check_staus_counter = 0;
            }
        } else {
            self.check_staus_counter = 0;
        }
    }

    pub(super) fn draw(&mut self, 
        lcd: &mut dyn LCDDisplayFn, 
        display_signal: &mut EventBits, 
        status_signal: &mut EventBits, 
        rtc: &Arc<Mutex<dyn RTC + 'static>>) -> Result<()> {
        
        if self.stack.is_empty() {
            self.handle_init(status_signal);
        }
            

        let top = self.stack.last_mut().ok_or("Screen stack is empty").map_err(|e| osal_rs::utils::Error::UnhandledOwned(e.to_string()))?;
        match top.draw(lcd, display_signal, status_signal, rtc)? {
            Nav::Stay => {}
            Nav::Push(_s) => { }
            Nav::PushId(_id) => { }
            Nav::Pop => { }
            Nav::PopTo(_id) => { }
            Nav::Replace(_s) => { }
        }
        Ok(())
    }

}