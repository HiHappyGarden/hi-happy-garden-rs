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
use osal_rs::os::{Mutex, Thread, ThreadFn};
use osal_rs::os::types::StackType;
use osal_rs::utils::Result;

use crate::apps::signals::display::{DisplayFlag::{self, *}, DisplaySignal};
use crate::apps::signals::error::{ErrorSignal, DisplayFlag::DisplayError};
use crate::drivers::platform::ThreadPriority;
use crate::traits::button::{ButtonState::{self, *}, OnClickable};
use crate::traits::encoder::{EncoderDirection::{self, *}, OnRotatableAndClickable};
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::signal::Signal;
use crate::traits::state::Initializable;
use crate::traits::wifi::RSSIStatus;
 

const APP_TAG: &str = "AppDisplay";
const APP_THREAD_NAME: &str = "display_trd";
const APP_STACK_SIZE: StackType = 1024;

pub struct Display<T>
where T: LCDDisplayFn + Clone + 'static
{
    lcd: Arc<Mutex<T>>,
    thread: Thread,

}

impl<T> Display<T> 
where T: LCDDisplayFn + Clone + 'static
{
    pub fn new(lcd: T) -> Self{
        Self {
            lcd: Mutex::new_arc(lcd),
            thread: Thread::new_with_to_priority(APP_THREAD_NAME, APP_STACK_SIZE, ThreadPriority::Normal),
        }
    }

    pub fn draw(&mut self) -> Result<()> {

        //self.lcd.invert_orientation()?;

        // self.lcd.draw_pixel(1, 1, LCDWriteMode::ADD)?;
        // self.lcd.draw_pixel(1, 2, LCDWriteMode::ADD)?;
        // self.lcd.draw_pixel(1, 3, LCDWriteMode::ADD)?;
        // self.lcd.draw_pixel(2, 4, LCDWriteMode::ADD)?;
        // self.lcd.draw_pixel(2, 5, LCDWriteMode::ADD)?;
        

        // self.lcd.draw_bitmap_image(30, 20, IC_WIFI_NO_SIGNAL.0, IC_WIFI_NO_SIGNAL.1, &IC_WIFI_NO_SIGNAL.2, LCDWriteMode::ADD)?;

        // self.lcd.draw_str("ciao", 80, 50, &FONT_8X8)?;
        // self.lcd.draw()?;
        Ok(())
    }
}

impl<T> Initializable for Display<T>
where T: LCDDisplayFn + Clone + 'static
{
    fn init(&mut self) -> osal_rs::utils::Result<()> {
        log_info!(APP_TAG, "Init LCD");


        DisplaySignal::init()?;
        let lcd = Arc::clone(&self.lcd);

        self.thread.spawn_simple(move || {


            //lcd.lock().unwrap().draw_str("Hello, World!", 10, 10, &FONT_8X8);

            let mut header = header::Header::new(Arc::clone(&lcd));

            loop {
                let signals = DisplaySignal::wait(0x00FFFFFF, 100);
                DisplaySignal::clear(signals);

                if signals > 0 {
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

                    
                }
            }


        })?;

        Ok(())
    }
}

impl<T> OnClickable for Display<T>
where T: LCDDisplayFn + Clone + 'static
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
where T: LCDDisplayFn + Clone + 'static
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