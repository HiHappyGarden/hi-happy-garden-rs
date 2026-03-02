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

mod header;

use alloc::sync::Arc;
use osal_rs::log_info;
use osal_rs::os::{Mutex, MutexFn, Thread, ThreadFn};
use osal_rs::os::types::StackType;

use crate::apps::display::header::Header;
use crate::apps::signals::display::{DisplayFlag::{self, *}, DisplaySignal};
use crate::apps::signals::error::ErrorFlag;
use crate::apps::signals::error::{ErrorSignal};
use crate::drivers::date_time;
use crate::drivers::platform::ThreadPriority;

use crate::traits::button::{ButtonState::{self, *}, OnClickable};
use crate::traits::encoder::{EncoderDirection::{self, *}, OnRotatableAndClickable};
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::signal::Signal;
use crate::traits::state::Initializable;
use crate::traits::wifi::RSSIStatus;
use crate::traits::rtc::RTC;


const APP_TAG: &str = "AppDisplay";
const APP_THREAD_NAME: &str = "display_trd";
const APP_STACK_SIZE: StackType = 1024;

pub struct Display<T>
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{
    rtc: Arc<Mutex<dyn RTC>>,
    lcd: Arc<Mutex<T>>,
    thread: Thread
}

impl<T> Display<T> 
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{
    pub fn new(rtc: Arc<Mutex<dyn RTC>>, lcd: T) -> Self{
        Self {
            rtc,
            lcd: Mutex::new_arc(lcd),
            thread: Thread::new_with_to_priority(APP_THREAD_NAME, APP_STACK_SIZE, ThreadPriority::Normal),
        }
    }
}

impl<T> Initializable for Display<T>
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{
    fn init(&mut self) -> osal_rs::utils::Result<()> {
        log_info!(APP_TAG, "Init LCD");


        DisplaySignal::init()?;
        let lcd = Arc::clone(&self.lcd);
        let rtc = Arc::clone(&self.rtc);


        
        self.thread.spawn_simple(move || {


            let mut header = Header::new( Arc::clone(&lcd));

            loop {
                let signals = DisplaySignal::wait(0x00FFFFFF, 100);
                DisplaySignal::clear(signals);



                if signals > 0 {
                    lcd.lock().unwrap().clear();

                    match DisplayFlag::from(signals) {
                        ButtonPressed => log_info!(APP_TAG, "Button Pressed"),
                        ButtonReleased => log_info!(APP_TAG, "Button Released"),
                        EncoderRotatedClockwise => log_info!(APP_TAG, "Encoder Rotated Clockwise"),
                        EncoderRotatedCounterClockwise => log_info!(APP_TAG, "Encoder Rotated Counter Clockwise"),
                        EncoderButtonPressed => log_info!(APP_TAG, "Encoder Button Pressed"),
                        EncoderButtonReleased => log_info!(APP_TAG, "Encoder Button Released"),
                        WifiStatusUnknown | WifiStatusExcellent | WifiStatusGood | WifiStatusFair | WifiStatusWeak | WifiStatusNoSignal => Self::draw_header(&mut header, &rtc, signals),
                        
                        _ => {}
                    }

                    
                    lcd.lock().unwrap().draw().unwrap_or_else(|e| {
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

impl<T> Display<T>
where T: LCDDisplayFn + Sync + Send + Clone + 'static 
{
    fn draw_header(header: &mut Header<T>, rtc: &Arc<Mutex<dyn RTC>>, signals: u32) {
            let date_time = rtc.lock().unwrap().get_timestamp().unwrap_or_else(|e| {
                log_info!(APP_TAG, "Error getting date time: {:?}", e);
                ErrorSignal::set(ErrorFlag::DateTime.into());
                0
            });

            let date_time = date_time::DateTime::from_timestamp_locale(date_time, true).unwrap_or_else(|e| {
                log_info!(APP_TAG, "Error converting timestamp to datetime: {:?}", e);
                ErrorSignal::set(ErrorFlag::DateTime.into());
                date_time::DateTime::default()
            });

            if let Err(e) =  header.draw(date_time, RSSIStatus::from_bites( (signals >> 6) as u8 )) {
                log_info!(APP_TAG, "Error drawing header: {:?}", e);
                ErrorSignal::set(ErrorFlag::Display.into());
            }
    }

}