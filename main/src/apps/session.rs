/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
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
#![allow(dead_code)]

use core::time::Duration;

use alloc::sync::Arc;
use at_parser_rs::{AtError, AtResult, at_quoted};
use at_parser_rs::context::AtContext;
use osal_rs::{access_static_option, log_error, log_info};
use osal_rs::os::{Timer, TimerFn, ToTick};
use osal_rs::utils::{Bytes, Error, Result};
use osal_rs_serde::{Deserialize, Serialize};

use crate::apps::config::Config;
use crate::apps::parser::{CMD_SIZE, at_cmd_response};
use crate::drivers::encrypt::{EncryptGeneric, SHA256_RESULT_BYTES};
use crate::traits::signal::Signal;
use crate::traits::state::Initializable;
use crate::apps::signals::status::{StatusSignal, StatusFlag};

const APP_TAG: &str = "AppSession";

static mut USER_LOCAL: User = User::new();

static mut USER_LOGGED: Option<User> = None;
static mut USER_TMP: User = User::new();

static mut TIMER: Option<Timer> = None;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub(super) struct User {
    email: Bytes<32>,
    password: Bytes<{SHA256_RESULT_BYTES * 2}>,
}

impl Default for User {
    fn default() -> Self {
        Self {
            email: Bytes::new(),
            password: Bytes::new(),
        }
    }
}

impl AtContext<{CMD_SIZE}> for User {

    fn exec(&mut self) -> AtResult<'_, {CMD_SIZE}> {

        let config = Config::shared();

        config.get_session().set_user(self);
        config.apply_session();
        self.clear();
        
        Ok(at_cmd_response!(Self::AT_RESP; ""))
    }

    #[inline]
    fn query(&mut self) -> AtResult<'_, {CMD_SIZE}> {
        Ok(at_cmd_response!(Self::AT_RESP; at_quoted!(self.email.as_str()), at_quoted!(self.password.as_str())))
    }

    #[inline]
    fn test(&mut self) -> AtResult<'_, {CMD_SIZE}> {
        Ok(at_cmd_response!(Self::AT_RESP; "<user>,<password>"))
    } 
    
    fn set(&mut self, args: at_parser_rs::Args) -> AtResult<'_, {CMD_SIZE}> {
        if unsafe { USER_LOGGED }.is_some() {
            return Err(AtError::Unhandled("Not logged".into()));
        }

        let arg0 = args.get(0).ok_or(AtError::InvalidArgs)?;
        if arg0.len() > 32 {
            return Err(AtError::Unhandled("Max len 32"));
        }

        let arg1 = args.get(1).ok_or(AtError::InvalidArgs)?;
        if arg1.len() > 32 {
            return Err(AtError::Unhandled("Max len 32"));
        }

        let arg1 = EncryptGeneric::get_sha256(arg1.as_bytes()).map_err(|_| AtError::InvalidArgs)?;

        self.email = Bytes::from_str(arg0);
        self.password = Bytes::from_str(arg1.as_str());
        
        Ok(at_cmd_response!(Self::AT_RESP; ""))
    }
}



impl User {

    pub const AT_CMD: &'static str = "AT+USR";
    pub const AT_RESP: &'static str = "+USR: ";

    const fn new() -> Self {
        Self { 
            email: Bytes::new(),
            password: Bytes::new(),
        }
    }

    pub fn get_local() -> &'static mut User {
        unsafe { &mut *&raw mut USER_LOCAL }
    }

    fn clear(&mut self) {
        self.email.clear();
        self.password.clear();
    }

}


#[derive(Serialize, Deserialize, Clone, Copy)]
pub(super) struct Session {
    users: [User; Session::MAX_USERS],
}

impl AtContext<{CMD_SIZE}> for Session {

    fn exec(&mut self) -> AtResult<'_, {CMD_SIZE}> {
        
        let password =  EncryptGeneric::get_sha256(unsafe { USER_TMP }.password.as_str().as_bytes()).map_err(|_| AtError::InvalidArgs)?;
        

        for User{email: user, password: pwd} in self.users.iter() {
            if *user == unsafe { USER_TMP }.email && pwd.as_str() == password.as_str() {
                unsafe { USER_LOGGED = Some(USER_TMP); }
                return Ok(at_cmd_response!(Self::AT_RESP; ""));
            }
        }

        Err(AtError::InvalidArgs)
    }

    fn query(&mut self) -> AtResult<'_, {CMD_SIZE}> {

        let logged = unsafe { *&raw const USER_LOGGED };

        let mut ret = Bytes::<{CMD_SIZE}>::new();

        if logged.is_some() { 
            ret.format(format_args!("\"{}\"", access_static_option!(USER_LOGGED).email.as_str()));
        } 

        Ok(at_cmd_response!(Self::AT_RESP; ret))
    }

    fn test(&mut self) -> AtResult<'_, {CMD_SIZE}> {
        Ok(at_cmd_response!(Self::AT_RESP; "<i|o>,<user>,<password>"))
    }

    fn set(&mut self, args: at_parser_rs::Args) -> AtResult<'_, {CMD_SIZE}> {
        let arg0 = args.get(0).ok_or(AtError::InvalidArgs)?;
        let arg1 = args.get(1).ok_or(AtError::InvalidArgs)?;
        let arg2 = args.get(2).ok_or(AtError::InvalidArgs)?;

        

        if arg0 == "i" { // Login
            unsafe {
                USER_TMP.email = Bytes::from_str(arg1);
                USER_TMP.password = Bytes::from_str(arg2);
            }
            StatusSignal::set(StatusFlag::UserLogged.into());
            access_static_option!(TIMER).start(0);
        } else if arg0 == "o" { // Logout
            Self::logout();
        } else {
            return Err(AtError::InvalidArgs);
        }

        Ok(at_cmd_response!(Self::AT_RESP; ""))
    }
}


impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}


impl Initializable for Session {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init app session");

        if let Ok(timer) = Timer::new("autoreload_timer",
        Duration::from_millis(50).to_ticks(),
        false,
        None,
        |_, _| {

            Self::logout();

            Ok(Arc::new(()))
        }) {
            unsafe {
                TIMER = Some(timer);
            }
        } else {
            log_error!(APP_TAG, "Error creating timer");
            return Err(Error::OutOfMemory)
        }

        Ok(())
    }
}

impl Session {
    pub const AT_CMD: &'static str = "AT+SESS";
    pub const AT_RESP: &'static str = "+SESS: ";
    pub const MAX_USERS : usize = 2;

    pub const fn new() -> Self {
        Self { users: [User::new(); Session::MAX_USERS] }
    }

    pub fn logout() {
        unsafe { USER_LOGGED = None; }
        StatusSignal::clear(StatusFlag::UserLogged.into());
        StatusSignal::clear(StatusFlag::UartCmd.into());
        StatusSignal::clear(StatusFlag::MqttCmd.into());
        StatusSignal::clear(StatusFlag::DisplayCmd.into());
    }

    pub fn set_user(&mut self, user: &User) {
        self.users[1] = *user;   
    }

    pub fn set_user_local(&self) {
        unsafe { USER_LOCAL = self.users[1]; }
    }

    pub fn set_system_user(&mut self, email: &str, password: &str) -> Result<()> {
        if email.is_empty() {
            return Ok(());
        }
        let hashed = EncryptGeneric::get_sha256(password.as_bytes())
            .map_err(|_| Error::Unhandled("Failed to hash system user password"))?;
        self.users[0].email = Bytes::from_str(email);
        self.users[0].password = Bytes::from_str(hashed.as_str());
        Ok(())
    }

    pub fn reset_timer() {
        access_static_option!(TIMER).reset(0);
    }
}

