/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
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
pub mod commons;
mod check;
mod date_time_editor;
mod date;
mod header;
mod input;
mod number;
mod text;
mod time;


use alloc::sync::Arc;
use osal_rs::log_info;
use osal_rs::os::{EventGroup, Mutex, MutexFn, Thread, ThreadFn};
use osal_rs::os::types::StackType;
use osal_rs::utils::{Bytes, Error};

use crate::apps::display::check::Check;
use crate::apps::display::header::Header;
use crate::apps::display::input::Input;

use crate::apps::display::number::Number;
use crate::apps::display::text::Text;
use crate::apps::display::time::Time;
use crate::apps::signals::display::{DisplayFlag::{*}, DisplaySignal};
use crate::apps::signals::error::{ErrorSignal, ErrorFlag};
use crate::drivers::date_time::DateTime;
use crate::drivers::platform::ThreadPriority;

use crate::traits::button::{ButtonState::{self, *}, OnClickable};
use crate::traits::encoder::{EncoderDirection::{self, *}, OnRotatableAndClickable};
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::screen::{ScreenParam, Screen};
use crate::traits::signal::Signal;
use crate::traits::state::Initializable;
use crate::traits::rtc::RTC;


const APP_TAG: &str = "AppDisplay";
const APP_THREAD_NAME: &str = "display_trd";
const APP_STACK_SIZE: StackType = 2_560;
const TICK_INTERVAL_MS: u16 = 100;

#[allow(dead_code)]
pub const DISPLAY_INPUT_MAX_SIZE: usize = crate::apps::display::input::MAX_SIZE;

pub struct Display<T>
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
        log_info!(APP_TAG, "Init LCD");


        DisplaySignal::init()?;
        let lcd = Arc::clone(&self.lcd);
        let rtc = Arc::clone(&self.rtc);

        let wifi_enabled = Arc::clone(&self.wifi_enabled);
        
        self.thread.spawn_simple(move || {

            let lcd = &mut *(lcd.lock().unwrap());


            let mut header = Header::new();   
            let mut _check = Check::new();
            let mut _time = Time::new();
            let mut _number = Number::new( 0, 100);
            let mut _text = Text::new();
            let mut input = Input::new();
            
            loop {
                let mut signals = DisplaySignal::wait(EventGroup::MAX_MASK, TICK_INTERVAL_MS as u32);
                DisplaySignal::clear(signals);

                let date_time = rtc.lock().unwrap().get_timestamp().unwrap_or_else(|e| {
                    log_info!(APP_TAG, "Error getting date time: {:?}", e);
                    ErrorSignal::set(ErrorFlag::DateTime.into());
                    0
                });


                let mut date_time = DateTime::from_timestamp_locale(date_time, true).unwrap_or_else(|e| {
                    log_info!(APP_TAG, "Error converting timestamp to datetime: {:?}", e);
                    ErrorSignal::set(ErrorFlag::DateTime.into());
                    DateTime::default()
                });

                // match DisplayFlag::from(signals) {
                //     ButtonPressed => log_info!(APP_TAG, "Button Pressed"),
                //     ButtonReleased => log_info!(APP_TAG, "Button Released"),
                //     EncoderRotatedClockwise => log_info!(APP_TAG, "Encoder Rotated Clockwise"),
                //     EncoderRotatedCounterClockwise => log_info!(APP_TAG, "Encoder Rotated Counter Clockwise"),
                //     EncoderButtonPressed => log_info!(APP_TAG, "Encoder Button Pressed"),
                //     EncoderButtonReleased => log_info!(APP_TAG, "Encoder Button Released"),
                    
                //     _ => {}
                // }


                if let Err(e) =  header.draw(lcd, &mut signals, &date_time, *wifi_enabled) {
                    if let Error::ReturnWithCode(_) = e {} else {
                        log_info!(APP_TAG, "Error drawing header: {:?}", e);
                        ErrorSignal::set(ErrorFlag::Display.into());
                    }
                }

                // if let Err(e) =  check.draw(&mut signals, &date_time, &Bytes::<64>::from_str("ciao sono antonio e programmo molto"), true, Some(|state| log_info!(APP_TAG, "Check state changed: {:?}", state))) {
                //     log_info!(APP_TAG, "Error drawing check: {:?}", e);
                //     ErrorSignal::set(ErrorFlag::Display.into());
                // }

                // let mut test = date_time.clone();
                //         test.year += 1;
                //         test.month += 1;        
                // if let Err(e) =  time.draw(&mut signals, &date_time, &Bytes::<64>::from_str("Insert time"), Option::None, Some(|time| log_info!(APP_TAG, "Time: {:?}", time))) {
                //     log_info!(APP_TAG, "Error drawing time: {:?}", e);
                //     ErrorSignal::set(ErrorFlag::Display.into());
                // }
                // if let Err(e) =  number.draw(&mut signals, &date_time, &Bytes::<64>::from_str("Insert number"), 3, Some(|number| log_info!(APP_TAG, "Number: {:?}", number))) {
                //     log_info!(APP_TAG, "Error drawing number: {:?}", e);
                //     ErrorSignal::set(ErrorFlag::Display.into());
                // }

                // if let Err(e) =  text.draw(&date_time, &Bytes::<64>::from_str("Insert text|questa è una stringa molto lunga che scorre")) {
                //     log_info!(APP_TAG, "Error drawing text: {:?}", e);
                //     ErrorSignal::set(ErrorFlag::Display.into());
                // }

                let mut p = ScreenParam::default();
                p.input = Some(Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Initial text"));

                if let Err(e) =  input.draw(lcd, &mut signals, &date_time, &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Insert text"),p , Some(|txt, confirmed| log_info!(APP_TAG, "Input: {:?}, Confirmed: {:?}", txt, confirmed))) {
                    log_info!(APP_TAG, "Error drawing text: {:?}", e);
                    ErrorSignal::set(ErrorFlag::Display.into());
                }
                


                if signals & Draw as u32 != 0 {
                    lcd.draw().unwrap_or_else(|e| {
                        ErrorSignal::set(ErrorFlag::Display.into());
                        log_info!(APP_TAG, "Error drawing on LCD: {:?}", e);
                    });
                }


                if date_time.millis >= 1000 {
                    date_time.millis = 0;
                } else {
                    date_time.millis += TICK_INTERVAL_MS;    
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

impl<T> Display<T> 
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{
    pub fn new(rtc: Arc<Mutex<dyn RTC>>, lcd: T) -> Self{
        Self {
            rtc,
            lcd: Mutex::new_arc(lcd),
            wifi_enabled: Arc::new(true),
            thread: Thread::new_with_to_priority(APP_THREAD_NAME, APP_STACK_SIZE, ThreadPriority::Normal),
        }
    }

    pub fn set_enabled_wifi(&mut self, enabled: bool) {
        match Arc::get_mut(&mut self.wifi_enabled) {
            Some(wifi_enabled) => *wifi_enabled = enabled,
            core::option::Option::None => Arc::make_mut(&mut self.wifi_enabled).clone_from(&Arc::new(enabled)),    
        }
    }
}
