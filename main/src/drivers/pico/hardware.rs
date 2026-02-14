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

use osal_rs::{log_error, log_info};
use osal_rs::os::types::UBaseType;
use osal_rs::os::{System, SystemFn, ToPriority};
use osal_rs::utils::{Error, OsalRsBool, Result};

use crate::drivers::button::Button;
use crate::drivers::date_time::DateTime;
use crate::drivers::encoder::Encoder;
use crate::drivers::filesystem::{Filesystem, FsStat};
use crate::drivers::i2c::I2C;
use crate::drivers::pico::ffi::{hhg_get_unique_id};
use crate::drivers::relays::Relays;
use crate::drivers::rgb_led::RgbLed;
use crate::drivers::rtc::RTC;
use crate::drivers::uart::Uart;
use crate::drivers::gpio::Gpio;
use crate::drivers::platform::{GpioPeripheral, I2C_BAUDRATE, I2C0_INSTANCE, I2C1_INSTANCE, LCDDisplay};
use crate::drivers::plt::flash::{FS_CONFIG_DIR, FS_DATA_DIR, FS_LOG_DIR};
use crate::drivers::plt::flash::lfs_errors::LFS_ERR_EXIST;
use crate::drivers::wifi::Wifi;

use crate::traits::rgb_led::RgbLed as RgbLedFn;
use crate::traits::relays::Relays as RelaysFn;
use crate::traits::button::{OnClickable, SetClickable as ButtonOnClickable};
use crate::traits::encoder::{OnRotatableAndClickable as EncoderOnRotatableAndClickable, SetRotatableAndClickable};
use crate::traits::hardware::HardwareFn;
use crate::traits::rtc::RTC as RTCFn;
use crate::traits::state::Initializable;
use crate::traits::wifi::{OnWifiChangeStatus, SetOnWifiChangeStatus};


const APP_TAG: &str = "Hardware";

#[allow(dead_code)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ThreadPriority {
    None = 0,
    Idle = 1,
    Low = 4,
    BelowNormal = 8,
    Normal = 12,
    AboveNormal = 16,
    BelowHigh = 19,
    High = 23,
    AboveHigh = 27,
    Realtime = 31,
}

impl ToPriority for ThreadPriority {
    #[inline]
    fn to_priority(&self) -> UBaseType {
        *self as UBaseType
    }
}

#[allow(unused)]
impl ThreadPriority {
    pub fn from_priority(priority: UBaseType) -> Self {
        use ThreadPriority::*;
        match priority {
            1 => Idle,
            2..=4 => Low,
            5..=8 => BelowNormal,
            9..=12 => Normal,
            13..=16 => AboveNormal,
            17..=19 => BelowHigh,
            20..=23 => High,
            24..=27 => AboveHigh,
            28..=31 => Realtime,
            _ => None,
        }
    }
}

pub struct Hardware {
    uart: Uart,
    encoder: Encoder,
    button: Button,
    rgb_led: RgbLed,
    relays: Relays,
    i2c0: I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>,
    i2c1: I2C<{I2C1_INSTANCE}, {I2C_BAUDRATE}>,
    wifi: Wifi,
    rtc: RTC,
}

impl Initializable for Hardware {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init hardware");

        self.wifi.init()?;

        Gpio::new().init()?;

        self.uart.init()?;
        
        self.relays.init()?;

        self.encoder.init()?;

        self.button.init()?;

        self.rgb_led.init()?;

        self.i2c0.init()?;
        
        self.i2c1.init()?;
        
        self.rtc.set_i2c(self.i2c0.clone());
        self.rtc.init()?;

        self.init_fs()?;


        if self.rtc.is_to_synch() {
            let timestamp = self.rtc.get_rtc_timestamp()?;

            let now = DateTime::from_timestamp(timestamp)?;
            log_info!(APP_TAG, "Sync RTS with POWMAN ({})", now);

            self.rtc.set_timestamp(timestamp as u64)?;
        }

        log_info!(APP_TAG, "Hardware initialized successfully heap_free:{}", System::get_free_heap_size());
        Ok(())
    } 
}

impl RgbLedFn for Hardware {
    #[inline]
    fn set_color(&self, red: u8, green: u8, blue: u8) {
        self.rgb_led.set_color(red, green, blue);
    }

    #[inline]
    fn set_red(&self, red: u8) {
        self.rgb_led.set_red(red);
    }
    
    #[inline]
    fn set_green(&self, green: u8) {
        self.rgb_led.set_green(green);
    }

    #[inline]
    fn set_blue(&self, blue: u8) {
        self.rgb_led.set_blue(blue);
    }
}

impl RelaysFn for Hardware {

    #[inline]
    fn set_relay_state(&self, relay_index: GpioPeripheral, state: bool) -> OsalRsBool {
        self.relays.set_relay_state(relay_index, state)
    }
}

impl HardwareFn<'static> for Hardware {
    #[inline]
    fn set_button_handler(&mut self, clickable: &'static dyn OnClickable) {
        self.button.set_on_click(clickable);
    }

    #[inline]
    fn set_encoder_handler(&mut self, rotable_and_clickable: &'static dyn EncoderOnRotatableAndClickable) {
        self.encoder.set_on_rotate_and_click(rotable_and_clickable);
    }
    
    fn get_temperature(&self) -> f32 {
        let gpio = Gpio::new();

        let mut sum = 0f32;
        for _ in 0..Self::SAMPLES {
            let raw_value = gpio.read(&GpioPeripheral::InternalTemp).unwrap_or(0);
            let temp = Self::temperature_conversion(raw_value);
            sum += temp / Self::SAMPLES as f32;
            System::delay(10);
        }
        sum 
    }
}

impl SetOnWifiChangeStatus<'static> for Hardware {
    fn set_on_wifi_change_status(&mut self, on_wifi_change_status: &'static dyn OnWifiChangeStatus) {
        self.wifi.set_on_wifi_change_status(on_wifi_change_status);
    }
}

impl RTCFn for Hardware {
    #[inline]
    fn set_timestamp(&self, timestamp: u64) {
        self.rtc.set_timestamp(timestamp).unwrap_or_else(|e| log_error!(APP_TAG, "Failed to set timestamp: {:?}", e));
    }

    #[inline]
    fn get_timestamp(&self) -> u64 {
        self.rtc.get_timestamp().unwrap_or_else(|e| {
            log_error!(APP_TAG, "Failed to get timestamp: {:?}", e);
            0
        })
    }

    #[inline]
    fn is_to_synch(&self) -> bool {
        self.rtc.is_to_synch()
    }
}

impl Hardware {
    pub fn new() -> Self {        

        Self { 
            uart: Uart::new(),
            encoder: Encoder::new(),
            button: Button::new(),
            rgb_led: RgbLed::new(),
            relays: Relays::new(),
            i2c0: I2C::new(),
            i2c1: I2C::new_with_address(LCDDisplay::I2C_ADDRESS),
            wifi: Wifi::new(),
            rtc: RTC::new(),
        }
    }

    pub fn set_internal_led(&self, state: bool) {
        Gpio::new().write(&GpioPeripheral::InternalLed, if state {1} else {0});
    }

    pub fn get_lcd_display(&mut self) -> LCDDisplay {
        let mut ret = LCDDisplay::new(self.i2c1.clone());
        ret.init().unwrap();
        ret
    }

    pub fn init_fs(&self) -> Result<()> {
        Filesystem::mount(true)?;

        //Filesystem::remove_recursive("/")?;

        if let Err(Error::ReturnWithCode(code)) = Filesystem::mkdir(FS_CONFIG_DIR) {
            if code != LFS_ERR_EXIST {
                return Err(Error::ReturnWithCode(code))
            }
        } else {
            log_info!(APP_TAG, "Created {FS_CONFIG_DIR} directory");
        }

        if let Err(Error::ReturnWithCode(code)) = Filesystem::mkdir(FS_DATA_DIR) {
            if code != LFS_ERR_EXIST {
                return Err(Error::ReturnWithCode(code))
            }
        } else {
            log_info!(APP_TAG, "Created {FS_DATA_DIR} directory");
        }

        if let Err(Error::ReturnWithCode(code)) = Filesystem::mkdir(FS_LOG_DIR) {
            if code != LFS_ERR_EXIST {
                return Err(Error::ReturnWithCode(code))
            }
        } else {
            log_info!(APP_TAG, "Created {FS_LOG_DIR} directory");
        }

        //test wtrite/read
        // let data = b"Hello, Hi Happy Garden!";
        // let mut file = Filesystem::open("text.txt", open_flags::WRONLY | open_flags::CREAT)?;
        // let bytes_written = file.write(data, true)?;
        // log_info!(APP_TAG, "Wrote {} bytes to text.txt", bytes_written);
        // file.close()?;
        //
        //
        // let mut file = Filesystem::open("text.txt", open_flags::RDONLY)?;
        // let read_buffer = file.read(true)?;
        // log_info!(APP_TAG, "Read from text.txt: {}", core::str::from_utf8(&read_buffer).unwrap_or("Invalid UTF-8"));
        // file.close()?;
        //
        // //ls /
        // for (name, type_) in Filesystem::ls("/")? {
        //     log_info!(APP_TAG, "{} - {}", name, type_);
        // }

        let FsStat{block_size, block_count, blocks_used} = Filesystem::stat_fs()?;

        let total_size = (block_size as u64) * (block_count as u64);
        let used_size = (block_size as u64) * (blocks_used as u64);
        let free_size = total_size - used_size;

        log_info!(
            APP_TAG, 
            "Filesystem total:{} bytes ({} KB), used:{} bytes ({} KB), free:{} bytes ({} KB)",
            total_size, total_size / 1024,
            used_size, used_size / 1024,
            free_size, free_size / 1024
        );

        Ok(())
    }
    

    pub fn get_unique_id() -> [u8; 8] {
        let mut id_buffer = [0u8; 8];
        unsafe {
            hhg_get_unique_id(id_buffer.as_mut_ptr());
        }
        id_buffer
    }   

}

