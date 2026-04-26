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

use core::time::Duration;

use alloc::boxed::Box;
use alloc::sync::Arc;
use osal_rs::{log_debug, log_info};
use osal_rs::os::types::StackType;
use osal_rs::os::{System, Thread, ThreadFn, ThreadParam};
use osal_rs::utils::{Error, Result};

use crate::apps::config::Config;
use crate::apps::display::Display;
use crate::apps::parser::Parser;
use crate::apps::signals::error::ErrorSignal;
use crate::apps::signals::status::{StatusFlag, StatusSignal};
use crate::apps::system_led::SystemLed;
use crate::apps::wifi::Wifi;
use crate::drivers::platform::{Hardware, LCDDisplay, ThreadPriority};
use crate::traits::hardware::HardwareFn;
use crate::traits::rx_tx::SetOnReceive;
use crate::traits::signal::Signal;
use crate::traits::state::Initializable;
use crate::traits::wifi::SetOnWifiChangeStatus;

const APP_TAG: &str = "AppMain";
const THREAD_NAME: &str = "app_main_trd";
const STACK_SIZE: StackType = 1_536;
const TICK_INTERVAL_MS: u16 = 100;

macro_rules! set_current_status {
    ($status_old:expr, $status_current:expr, $status:expr) => {
        log_debug!(APP_TAG, "Status change: {:?} -> {:?}", $status_current, $status);
        StatusSignal::clear($status_current.into());
        $status_old = $status_current;
        $status_current = $status;
        StatusSignal::set($status.into());
    };
}


#[derive(Clone, Copy)]
struct AppMainPtr(usize);

pub struct AppMain {
    hardware: &'static mut Hardware,
    display: Display<LCDDisplay>,
    wifi: Wifi,
    parser: Parser,
    system_led: SystemLed,
    thread: Option<Thread>
}

unsafe impl Sync for AppMain {}
unsafe impl Send for AppMain {}


impl Initializable for AppMain{
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init app main");

        StatusSignal::init()?;
        ErrorSignal::init()?;

        let config = Config::shared();
        

        config.init()?;
        self.system_led.init()?;
        self.parser.init()?;
        self.wifi.init()?;
        self.display.init()?;
        self.display.set_enabled_wifi(config.get_wifi_config().is_enabled());

        //main FSM thread
        let app_param = AppMainPtr(self as *mut Self as usize); // Pass AppMain pointer as usize to thread
        let mut thread = Thread::new_with_to_priority(THREAD_NAME, STACK_SIZE, ThreadPriority::BelowHigh);

        self.thread = Some(thread.spawn(Some(Arc::new(app_param)), Self::thread_handler)?);

        Ok(())
    }
}

impl AppMain {
    pub fn new(hardware: &'static mut Hardware) -> Self {
        
        let display = Display::shared(hardware.get_rtc(), hardware.get_lcd_display());

        Self {
            hardware,
            display,
            wifi: Wifi::shared(),
            parser: Parser::shared(),
            system_led: SystemLed::new(),
            thread: None,  
        }
    }

    fn check_config(config: &Config, status_current: &mut StatusFlag, status_old: &mut StatusFlag) {
        let serial = config.get_serial();
        if serial.is_empty() {

        } else {
            set_current_status!(*status_old, *status_current, StatusFlag::EnableWifi);
        }
    }

    fn thread_handler(_: Box<dyn ThreadFn>, param: Option<ThreadParam>) -> Result<ThreadParam> {
        
        // SAFETY: AppMain lives in static mut APP_MAIN, initialized once at startup. We use raw pointers to avoid borrow checker issues, then convert to 'static refs.
        let me = unsafe { &mut *param // Get the thread parameter
        .as_deref() // Defrence the Option<Arc<dyn Any + Send + Sync>> to Option<&(dyn Any + Send + Sync)>
        .and_then(|param| param.downcast_ref::<AppMainPtr>()) // Option<&(dyn Any + Send + Sync)> to Option<&AppMainPtr>
        .map(|param| param.0 as *mut AppMain) // Get from Option<&AppMainPtr> the usize pointer and convert to *mut AppMain
        .ok_or(Error::Unhandled("Missing AppMain thread param"))? }; // Extract AppMain pointer or return error if missing



        let config = Config::shared();

        let mut status_current = StatusFlag::None;
        let mut status_old = StatusFlag::None;


        // SAFETY: AppMain lives in static mut APP_MAIN, initialized once at startup.
        // We use raw pointers to avoid borrow checker issues, then convert to 'static refs.
        unsafe {
            let display_ptr = &raw mut me.display;
            let wifi_ptr = &raw mut me.wifi;
            let parser_ptr = &raw mut me.parser;
            let hardware_ptr = &raw mut me.hardware;

            loop {
                match status_current {
                    StatusFlag::None => {
                        set_current_status!(status_old, status_current, StatusFlag::Startup);
                    }
                    StatusFlag::Startup => {
                        log_debug!(APP_TAG, "Start MAIN FSM");
                        
                        set_current_status!(status_old, status_current, StatusFlag::EnableSystemHandler);
                    }
                    StatusFlag::EnableSystemHandler => {
                        set_current_status!(status_old, status_current, StatusFlag::EnableSession);
                    }
                    StatusFlag::EnableSession => {
                        set_current_status!(status_old, status_current, StatusFlag::EnableParser);
                    }
                    StatusFlag::EnableParser => {
                        // Set transmit function pointer on parser 
                        Parser::set_uart_transmit(*hardware_ptr);
                        (*hardware_ptr).set_on_receive(&*parser_ptr);

                        set_current_status!(status_old, status_current, StatusFlag::EnableDisplay);
                    }
                    StatusFlag::EnableDisplay => {
                        // Set hardware callbacks - convert raw pointers to 'static references
                        (*hardware_ptr).set_button_handler(&*display_ptr);
                        (*hardware_ptr).set_encoder_handler(&*display_ptr);

                        Parser::set_system_transmit(&*display_ptr);
                        (&mut *display_ptr).set_on_receive(&*parser_ptr);
                        set_current_status!(status_old, status_current, StatusFlag::CheckConfig);
                    }
                    StatusFlag::CheckConfig => Self::check_config(&config, &mut status_current, &mut status_old),
                    StatusFlag::EnableWifi => {
                        // Set RTC for wifi
                        (*wifi_ptr).set_rtc((*hardware_ptr).get_rtc());

                        // Set wifi configuration change callback
                        (*hardware_ptr).set_on_wifi_change_status(&mut *wifi_ptr);

                        set_current_status!(status_old, status_current, StatusFlag::Ready);
                    }
                    StatusFlag::Ready => {

                        StatusSignal::set(StatusFlag::Ready.into());
                        
                    },
                    StatusFlag::Error => todo!(),
                    StatusFlag::Reset | _  => todo!(),
                
                }
                System::delay_with_to_tick(Duration::from_millis(TICK_INTERVAL_MS.into()));
            }
        
        }

    }

}