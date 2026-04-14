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
use osal_rs::{access_static_option, log_error, log_info, log_warning};
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

/// Temp user data for update local user
static mut USER_LOCAL: User = User::new();

/// User currently logged in, None if no user is logged
static mut USER_LOGGED: Option<User> = None;

/// Temporary user used for login/logout operations
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

    fn exec(&mut self, at_response: &'static str) -> AtResult<'_, {CMD_SIZE}> {
        if unsafe { USER_LOGGED }.is_none() {
            return Err((at_response, AtError::Unhandled("Not logged".into())));
        }

        let config = Config::shared();

        config.get_session().set_user(self);
        config.apply_session();
        self.clear();
        
        Ok(at_cmd_response!(at_response; ""))
    }

    fn query(&mut self, at_response: &'static str) -> AtResult<'_, {CMD_SIZE}> {
        if unsafe { USER_LOGGED }.is_none() {
            return Err((at_response, AtError::Unhandled("Not logged".into())));
        }
        Ok(at_cmd_response!(at_response; at_quoted!(self.email.as_str()), at_quoted!(self.password.as_str())))
    }

    #[inline]
    fn test(&mut self, at_response: &'static str) -> AtResult<'_, {CMD_SIZE}> {
        Ok(at_cmd_response!(at_response; "<email>,<password>"))
    } 
    
    fn set(&mut self, at_response: &'static str, args: at_parser_rs::Args) -> AtResult<'_, {CMD_SIZE}> {
        if unsafe { USER_LOGGED }.is_none() {
            return Err((at_response, AtError::Unhandled("Not logged".into())));
        }

        let arg0 = args.get(0).ok_or((at_response, AtError::InvalidArgs))?;
        if arg0.len() > 32 {
            return Err((at_response, AtError::Unhandled("email max len 32")));

        }

        let arg1 = args.get(1).ok_or((at_response, AtError::InvalidArgs))?;
        if arg1.len() > 32 {
            return Err((at_response, AtError::Unhandled("password max len 32")));
        }

        self.email = Bytes::from_str(arg0.as_ref());
        self.password = EncryptGeneric::get_sha256(arg1.as_bytes()).map_err(|_| (at_response, AtError::InvalidArgs))?;
        
        Ok(at_cmd_response!(at_response; ""))
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

    fn exec(&mut self, at_response: &'static str) -> AtResult<'_, {CMD_SIZE}> {
        
        
        unsafe {
            match USER_TMP {
                User{email, password} if email.len() == 0 || password.len() == 0 => {
                    Self::logout();
                    Ok(at_cmd_response!(at_response; ""))
                }
                User{email, password} if email.len() > 0 && password.len() > 0 => self.login(at_response),
                User { .. } => Err((at_response, AtError::InvalidArgs))
            }
        }

    }

    fn query(&mut self, at_response: &'static str) -> AtResult<'_, {CMD_SIZE}> {

        let logged = unsafe { *&raw const USER_LOGGED };
        
        if logged.is_some() { 
            Ok(at_cmd_response!(at_response; access_static_option!(USER_LOGGED).email.as_str()))
        } else {
            Err((at_response, AtError::InvalidArgs))
        }
        
    }

    #[inline]
    fn test(&mut self, at_response: &'static str) -> AtResult<'_, {CMD_SIZE}> {
        Ok(at_cmd_response!(at_response; "<i|o>,<email>,<password>"))
    }

    fn set(&mut self, at_response: &'static str, args: at_parser_rs::Args) -> AtResult<'_, {CMD_SIZE}> {
        let arg0 = args.get(0).ok_or((at_response, AtError::InvalidArgs))?;

        if arg0 == "i" { // Login
            let arg1 = args.get(1).ok_or((at_response, AtError::InvalidArgs))?;
            let arg2 = args.get(2).ok_or((at_response, AtError::InvalidArgs))?;
            unsafe {
                USER_TMP.email = Bytes::from_str(arg1.as_ref());
                USER_TMP.password = EncryptGeneric::get_sha256(arg2.as_bytes()).map_err(|_| (at_response, AtError::InvalidArgs))?;
            }
        } else if arg0 == "o" { // Logout
            unsafe {
                (*USER_TMP.email).fill(0);
                (*USER_TMP.password).fill(0);
            }
            
        } else {
            return Err((at_response, AtError::InvalidArgs));
        }

        Ok(at_cmd_response!(at_response; ""))
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
        Duration::from_mins(5).to_ticks(),
        false,
        None,
        |_, _| {

            Self::logout();
            
            log_warning!(APP_TAG, "Session timeout, user logged out");

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

    fn login(&self, at_response: &'static str) -> AtResult<'_, {CMD_SIZE}> {
        let user_tmp = unsafe { USER_TMP }.clone();

        if user_tmp.email.len() == 0 || user_tmp.password.len() == 0 {
            return Err((at_response, AtError::InvalidArgs));
        }

        for User{email: user, password: pwd} in self.users.iter() {
            if *user == user_tmp.email && pwd.as_raw_bytes() == user_tmp.password.as_raw_bytes() {
                unsafe { USER_LOGGED = Some(USER_TMP); }

                StatusSignal::set(StatusFlag::UserLogged.into());
                access_static_option!(TIMER).start(0);

                return Ok(at_cmd_response!(at_response; unsafe { USER_TMP.email } ));
            }
        }

        if unsafe { USER_LOGGED }.is_none() {
            unsafe {
                (*USER_TMP.email).fill(0);
                (*USER_TMP.password).fill(0);
            }
        }

        Err((at_response, AtError::InvalidArgs))
    }


    fn logout() {
        unsafe { USER_LOGGED = None; }
        StatusSignal::clear(StatusFlag::UserLogged.into());
        StatusSignal::clear(StatusFlag::UartCmd.into());
        StatusSignal::clear(StatusFlag::MqttCmd.into());
        StatusSignal::clear(StatusFlag::DisplayCmd.into());
    }

    #[inline]
    pub fn set_user(&mut self, user: &User) {
        self.users[1] = *user;   
    }

    #[inline]
    pub fn set_user_local(&self) {
        unsafe { USER_LOCAL = self.users[1]; }
    }

    pub fn set_system_user(&mut self, email: &str, password: &str) -> Result<()> {
        if email.is_empty() || password.is_empty() {
            return Err(Error::Empty);
        }
        self.users[0].email = Bytes::from_str(email);
        self.users[0].password = Bytes::from_str(password);
        Ok(())
    }

    #[inline]
    pub fn reset_timer() {
        access_static_option!(TIMER).reset(0);
    }
}

