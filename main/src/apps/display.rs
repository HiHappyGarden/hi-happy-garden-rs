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

use crate::apps::signals::display::{DisplayFlag::{self, *}, DisplaySignal};
use crate::apps::signals::error::{ErrorSignal, DisplayFlag::DisplayError};
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
    rtc: Arc<&'static dyn RTC>,
    lcd: Arc<Mutex<T>>,
    thread: Thread
}

impl<T> Display<T> 
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{
    pub fn new(rtc: Arc<&'static dyn RTC>, lcd: T) -> Self{
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



            
            //lcd.lock().unwrap().draw_str("Hello, World!", 10, 10, &FONT_8X8);

            let mut header = header::Header::new(Arc::clone(&rtc), Arc::clone(&lcd));

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
                        WifiStatusUnknown | WifiStatusExcellent | WifiStatusGood | WifiStatusFair | WifiStatusWeak | WifiStatusNoSignal => {
                            if let Err(e) =  header.draw(RSSIStatus::from_bites( (signals >> 6) as u8 )) {
                                ErrorSignal::set(DisplayError.into());
                                log_info!(APP_TAG, "Error drawing header: {:?}", e);
                            }
                        }
                        
                        _ => {}
                    }

                    
                    lcd.lock().unwrap().draw().unwrap_or_else(|e| {
                        ErrorSignal::set(DisplayError.into());
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