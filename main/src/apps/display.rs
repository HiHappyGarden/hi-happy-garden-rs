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

#[cfg(feature = "tests")]
pub(super) mod tests;

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


use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;

use alloc::sync::Arc;
use osal_rs::log_info;
use osal_rs::os::{EventGroup, Mutex, MutexFn, System, SystemFn, Thread, ThreadFn};
use osal_rs::os::types::{EventBits, StackType, TickType};
use osal_rs::utils::{Bytes, Error, OsalRsBool, Result};

use crate::apps::display::header::Header;
use crate::apps::display::commons::{MAX_SIZE, clean_context};

use crate::apps::display::text::Text;
use crate::apps::screen_route::ScreenRoute;
use crate::apps::signals::display::DisplayFlag;
use crate::apps::signals::display::{DisplayFlag::{*}, DisplaySignal};
use crate::apps::signals::error::{ErrorSignal, ErrorFlag};
use crate::apps::signals::status::{StatusFlag, StatusSignal};
use crate::apps::sprinkler::DISBURSEMENT_MASK;
use crate::apps::sprinkler::zone::ZoneController;
use crate::drivers::platform::ThreadPriority;

use crate::traits::button::{ButtonState::{self, *}, OnClickable};
use crate::traits::encoder::{EncoderDirection::{self, *}, OnRotatableAndClickable};
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::rx_tx::{OnReceive, SetOnReceive, SetTransmit};
use crate::traits::screen::{Answer, Screen, ScreenParam};
use crate::traits::signal::Signal;
use crate::traits::state::Initializable;
use crate::traits::rtc::RTC;


const APP_TAG: &str = "AppDisplay";
const THREAD_NAME: &str = "app_display_thr";
const STACK_SIZE: StackType = 1_024 * 7;
const TICK_INTERVAL_MS: u16 = 100;
const SLEEP_PERIOD: Duration = Duration::from_secs(60);

/// Input events that bring the display out of sleep.
const WAKE_MASK: EventBits = ButtonReleased as EventBits
                | EncoderButtonReleased as EventBits
                | EncoderRotatedClockwise as EventBits
                | EncoderRotatedCounterClockwise as EventBits;

/// Status bits that keep the display awake while set: an active irrigation
/// or an open remote command session.
const KEEP_AWAKE_MASK: EventBits = DISBURSEMENT_MASK
                | StatusFlag::SystemCmd as EventBits
                | StatusFlag::MqttCmd as EventBits
                | StatusFlag::UartCmd as EventBits;

/// Event bits the display task waits on and consumes. `Disbursement` is a level
/// flag driven by AppSystemLed (blink phase): it is only read, never waited on
/// (it would busy-loop the task) nor cleared here.
const EVENT_MASK: EventBits = EventGroup::MAX_MASK & !(Disbursement as EventBits);

pub(crate) const DISPLAY_INPUT_MAX_SIZE: usize = MAX_SIZE;

static WIFI_ENABLED: AtomicBool = AtomicBool::new(true);

static mut ON_RECEIVE: Option<&'static dyn OnReceive> = Option::None;

pub(super) struct Display<T>
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{
    rtc: Arc<Mutex<dyn RTC>>,
    lcd: Arc<Mutex<T>>,
    thread: Thread
}

impl<T> Initializable for Display<T>
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init app display");


        DisplaySignal::init()?;
        let lcd = Arc::clone(&self.lcd);
        let rtc = Arc::clone(&self.rtc);

        self.thread = self.thread.spawn_simple(move || {

            let lcd = &mut *(lcd.lock().unwrap());

            let mut screen_route = ScreenRoute::new();

            let mut header = Header::new();
            if let Err(e) =  Text::new().draw(
                lcd, 
                &mut 0, 
                &rtc, 
                &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("Loading..."), 
                ScreenParam::default()
            ) {
                log_info!(APP_TAG, "Error drawing text: {:?}", e);
                ErrorSignal::set(ErrorFlag::Display.into());
            }
            lcd.draw().unwrap_or_else(|e| {
                ErrorSignal::set(ErrorFlag::Display.into());
                log_info!(APP_TAG, "Error drawing on LCD: {:?}", e);
            });

            let mut sleep_mode = false;
            let mut sleep_timer = System::get_current_time();

            let mut disbursement_text= Option::<Text>::None;

            loop {
                //wait for display signal
                let mut display_signal = DisplaySignal::wait(EVENT_MASK, TICK_INTERVAL_MS as TickType) & EVENT_MASK;
                DisplaySignal::clear(display_signal);

                //get status signal (after the wait, so it is up to date)
                let mut status_signal = StatusSignal::get();
                let keep_awake = status_signal & KEEP_AWAKE_MASK != 0;

                if keep_awake {
                    //irrigation or remote command in progress: never sleep, restart the timer
                    sleep_timer = System::get_current_time();
                }

                if !sleep_mode && !keep_awake && DisplayFlag::Sleep.check_signal(display_signal) {
                    lcd.turn_off()?;
                    sleep_mode = true;
                }

                if sleep_mode {
                    if display_signal & WAKE_MASK == 0 && !keep_awake {
                        continue;
                    }

                    lcd.turn_on()?;
                    sleep_mode = false;
                    sleep_timer = System::get_current_time();

                    //swallow the waking event and force a full redraw
                    display_signal = Draw as EventBits;
                }

                //build header
                if let Err(e) =  header.draw(lcd, &mut display_signal, &status_signal, &rtc, WIFI_ENABLED.load(Ordering::Relaxed)) {
                    if let Error::ReturnWithCode(_) = e {} else {
                        log_info!(APP_TAG, "Error drawing header: {:?}", e);
                        ErrorSignal::set(ErrorFlag::Display.into());
                    }
                }

                let disbursement_active: bool = status_signal & DISBURSEMENT_MASK != 0;
                if disbursement_active {


                    Self::disbursement_handler(
                        &mut display_signal, 
                        &mut status_signal,
                        lcd, 
                        rtc.clone(), 
                        &mut disbursement_text
                    ).unwrap_or_else(|e| {
                            log_info!(APP_TAG, "Error drawing disbursement: {:?}", e);
                            ErrorSignal::set(ErrorFlag::Display.into());
                        });

                } else {
                    if disbursement_text.take().is_some() {
                        //disbursement just ended: force a full redraw of the current screen
                        display_signal |= Draw as EventBits;
                    }

                    //draw current screen
                    screen_route.draw(lcd, &mut display_signal, &mut status_signal, &rtc).unwrap_or_else(|e| {
                        log_info!(APP_TAG, "Error drawing screen route: {:?}", e);
                        ErrorSignal::set(ErrorFlag::Display.into());
                    });
                }


                
                //check if draw signal is set, if so, redraw the screen
                if display_signal & (Draw as EventBits | DrawInternal as EventBits) != 0 {
                    lcd.draw().unwrap_or_else(|e| {
                        ErrorSignal::set(ErrorFlag::Display.into());
                        log_info!(APP_TAG, "Error drawing on LCD: {:?}", e);
                    });

                    if display_signal & Draw as EventBits != 0 {
                        sleep_timer = System::get_current_time();
                    }
                }

                if System::check_timer(&sleep_timer, &SLEEP_PERIOD) == OsalRsBool::True {
                    DisplaySignal::set(Sleep as EventBits);
                }
                
            }

        })?;

        Ok(())
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
    pub(super) fn new(lcd: T, rtc: Arc<Mutex<dyn RTC>>) -> Self {
        Self {
            rtc,
            lcd: Mutex::new_arc(lcd),
            thread: Thread::new_with_to_priority(THREAD_NAME, STACK_SIZE, ThreadPriority::BelowHigh)
        }
    }

    pub(super) fn set_enabled_wifi(&mut self, enabled: bool) {
        WIFI_ENABLED.store(enabled, Ordering::Relaxed);
    }

    fn disbursement_handler(
        display_signal: &mut EventBits, 
        status_signal: &mut EventBits, 
        lcd: &mut dyn LCDDisplayFn, 
        rtc: Arc<Mutex<dyn RTC + 'static>>, 
        disbursement_text: &mut Option<Text>) -> Result<()> {

        let idx = if StatusFlag::DisbursementZoneRelay0.check_signal(*status_signal) {
            0usize
        } else if StatusFlag::DisbursementZoneRelay1.check_signal(*status_signal) {
            1usize
        } else if StatusFlag::DisbursementZoneRelay2.check_signal(*status_signal) {
            2usize
        } else if StatusFlag::DisbursementZoneRelay3.check_signal(*status_signal) {
            3usize
        } else {
            // disbursement ended between the mask check and here: nothing to draw
            return Ok(());
        };

        // first tick of a disbursement: full redraw (header + content)
        if disbursement_text.is_none() {
            *disbursement_text = Some(Text::new());
            *display_signal |= Draw as EventBits;
        }

        // blink phase is owned by AppSystemLed (in sync with the LED)
        if DisplaySignal::get() & Disbursement as EventBits != 0 {
            // visible phase: redraw the zone description every tick (keeps scrolling alive)
            let description = ZoneController::shared()[idx].description.clone();

            let answer = if let Some(text) = disbursement_text.as_mut() {
                if description.as_str().is_empty() {
                    text.draw(lcd, display_signal, &rtc, &Bytes::<MAX_SIZE>::from("Disbursement"), ScreenParam::None)?
                } else {
                    text.draw(lcd, display_signal, &rtc, &description, ScreenParam::None)?
                }
            } else {
                Answer::Pending
            };

            *display_signal |= DrawInternal as EventBits;

            if let Answer::Cancelled = answer {
                // only a request: AppSprinkler closes the relays and clears
                // the Disbursement* flags together with the GPIO outputs
                StatusSignal::set(StatusFlag::DisbursementForcedStop as EventBits);
            }
        } else {
            // hidden phase: clear only the content area, keep the header
            clean_context(lcd)?;
            *display_signal |= DrawInternal as EventBits;
        }

        Ok(())
    }
}

/// Button and encoder input for the display, as seen by the drivers.
///
/// The drivers keep this callback for ever. Holding a `&Display` instead
/// would forbid any later `&mut Display`, so the board gets a zero sized
/// forwarder that only raises a [`DisplaySignal`], which is all it did anyway.
pub(super) struct DisplayInput;

pub(super) static DISPLAY_INPUT: DisplayInput = DisplayInput;

impl OnClickable for DisplayInput {
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

impl OnRotatableAndClickable for DisplayInput {
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
