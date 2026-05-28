/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
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

#[macro_use]
pub(super) mod commons;
pub(super) mod check;
mod date_time_editor;
pub(super) mod date;
mod header;
pub(super) mod input;
pub(super) mod number;
pub(super) mod select;
pub(super) mod text;
pub(super) mod time;

use alloc::sync::Arc;
use osal_rs::log_info;
use osal_rs::os::{EventGroup, Mutex, MutexFn, Thread, ThreadFn};
use osal_rs::os::types::StackType;
use osal_rs::utils::{Bytes, Error};

use crate::apps::display::header::Header;
use crate::apps::display::commons::MAX_SIZE;

use crate::apps::display::text::Text;
use crate::apps::screen_route::SCREEN_ROUTE;
use crate::apps::signals::display::{DisplayFlag::{*}, DisplaySignal};
use crate::apps::signals::error::{ErrorSignal, ErrorFlag};
use crate::apps::signals::status::StatusSignal;
use crate::drivers::platform::ThreadPriority;

use crate::traits::button::{ButtonState::{self, *}, OnClickable};
use crate::traits::encoder::{EncoderDirection::{self, *}, OnRotatableAndClickable};
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::rx_tx::{OnReceive, SetOnReceive, SetTransmit};
use crate::traits::screen::{Screen, ScreenParam, ScreenRoute};
use crate::traits::signal::Signal;
use crate::traits::state::Initializable;
use crate::traits::rtc::RTC;


const APP_TAG: &str = "AppDisplay";
const THREAD_NAME: &str = "app_display_trd";
const STACK_SIZE: StackType = 1_024 * 7;
const TICK_INTERVAL_MS: u16 = 100;

pub const DISPLAY_INPUT_MAX_SIZE: usize = MAX_SIZE;

static mut ON_RECEIVE: Option<&'static dyn OnReceive> = Option::None;

pub(super) struct Display<T>
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{
    rtc: Arc<Mutex<dyn RTC>>,
    lcd: Arc<Mutex<T>>,
    wifi_enabled: Arc<bool>,
    thread: Thread
}

impl<T> Initializable for Display<T>
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{
    fn init(&mut self) -> osal_rs::utils::Result<()> {
        log_info!(APP_TAG, "Init app display");


        DisplaySignal::init()?;
        let lcd = Arc::clone(&self.lcd);
        let rtc = Arc::clone(&self.rtc);

        let wifi_enabled = Arc::clone(&self.wifi_enabled);
        
        self.thread = self.thread.spawn_simple(move || {

            let lcd = &mut *(lcd.lock().unwrap());

            let screen_route = unsafe{&mut *&raw mut SCREEN_ROUTE};

            let mut header = Header::new();
            if let Err(e) =  Text::new().draw(
                lcd, 
                &mut 0, 
                &rtc, 
                &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Loading..."), 
                ScreenParam::default(), 
                Option::None
            ) {
                log_info!(APP_TAG, "Error drawing text: {:?}", e);
                ErrorSignal::set(ErrorFlag::Display.into());
            }
            lcd.draw().unwrap_or_else(|e| {
                ErrorSignal::set(ErrorFlag::Display.into());
                log_info!(APP_TAG, "Error drawing on LCD: {:?}", e);
            });

            loop {
                //wait for display signal
                let mut display_signal = DisplaySignal::wait(EventGroup::MAX_MASK, TICK_INTERVAL_MS as u32);
                DisplaySignal::clear(display_signal);

                //get status signal
                let mut status_signal = StatusSignal::get();

                //build header
                if let Err(e) =  header.draw(lcd, &mut display_signal, &rtc, *wifi_enabled) {
                    if let Error::ReturnWithCode(_) = e {} else {
                        log_info!(APP_TAG, "Error drawing header: {:?}", e);
                        ErrorSignal::set(ErrorFlag::Display.into());
                    }
                }

                
                //draw current screen
                screen_route.draw(lcd, &mut display_signal, &mut status_signal, &rtc).unwrap_or_else(|e| {
                    log_info!(APP_TAG, "Error drawing screen route: {:?}", e);
                    ErrorSignal::set(ErrorFlag::Display.into());
                });

                
                //check if draw signal is set, if so, redraw the screen
                if display_signal & Draw as u32 != 0 {
                    lcd.draw().unwrap_or_else(|e| {
                        ErrorSignal::set(ErrorFlag::Display.into());
                        log_info!(APP_TAG, "Error drawing on LCD: {:?}", e);
                    });
                }
                
            }

        })?;

        Ok(())
    }
}

impl<T> OnClickable for Display<T>
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{
    fn on_click(&self, state: ButtonState) {
        match state {
            Pressed => {
                let _ = DisplaySignal::set_from_isr(ButtonPressed.into());
            }
            Released => {
                let _ = DisplaySignal::set_from_isr(ButtonReleased.into());
            }
            ButtonState::None => {}
        }
    }
}

impl<T> OnRotatableAndClickable for Display<T>
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{
    fn on_rotable(&self, direction: EncoderDirection, _: i32) {
        match direction {
            Clockwise => {
                let _ = DisplaySignal::set_from_isr(EncoderRotatedClockwise.into());
            }
            CounterClockwise => {
                let _ = DisplaySignal::set_from_isr(EncoderRotatedCounterClockwise.into());
            }
        }
    }

    fn on_click(&self, state: ButtonState) {
        match state {
            Pressed => {
                let _ = DisplaySignal::set_from_isr(EncoderButtonPressed.into());
            }
            Released => {
                let _ = DisplaySignal::set_from_isr(EncoderButtonReleased.into());
            }
            ButtonState::None => {}
        }
    }
}

impl<T> SetOnReceive<'static> for Display<T> 
where T: LCDDisplayFn + Sync + Send + Clone + 'static {

    #[inline]
    fn set_on_receive(&mut self, on_receive: &'static dyn OnReceive) {
        unsafe {
            ON_RECEIVE = Some(on_receive);
        }
    }
}

impl<T> SetTransmit for Display<T> 
where T: LCDDisplayFn + Sync + Send + Clone + 'static {

    #[inline]
    fn transmit(&self, _data: &[u8]) -> usize {
        // This is a display, it doesn't transmit data, so we can ignore this
        0
    }
}

impl<T> Display<T> 
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{
    pub(super) fn shared(rtc: Arc<Mutex<dyn RTC>>, lcd: T) -> Self{
        Self {
            rtc,
            lcd: Mutex::new_arc(lcd),
            wifi_enabled: Arc::new(true),
            thread: Thread::new_with_to_priority(THREAD_NAME, STACK_SIZE, ThreadPriority::BelowHigh),
        }
    }

    pub(super) fn set_enabled_wifi(&mut self, enabled: bool) {
        match Arc::get_mut(&mut self.wifi_enabled) {
            Some(wifi_enabled) => *wifi_enabled = enabled,
            core::option::Option::None => Arc::make_mut(&mut self.wifi_enabled).clone_from(&Arc::new(enabled)),    
        }
    }
}
