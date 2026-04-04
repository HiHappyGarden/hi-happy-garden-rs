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

use alloc::str;
use at_parser_rs::{AtError, AtResult};
use at_parser_rs::context::AtContext;
use osal_rs::utils::{Bytes, Result};
use osal_rs_serde::{Deserialize, Serialize};

use crate::apps::config::Config;
use crate::apps::parser::{CMD_SIZE, at_cmd_response};
use crate::drivers::encrypt::{EncryptGeneric, SHA256_RESULT_BYTES};

const APP_TAG: &str = "AppSession";

static mut USER_LOCAL: User = User::new();

static mut USER_LOGGED: Option<User> = None;
static mut USER_TMP: User = User::new();

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
        Ok(at_cmd_response!(Self::AT_RESP; self.email.as_str(), self.password.as_str()))
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
pub(super) struct Session ([User; Session::MAX_USERS]);

impl AtContext<{CMD_SIZE}> for Session {

    fn exec(&mut self) -> AtResult<'_, {CMD_SIZE}> {
        
        let password =  EncryptGeneric::get_sha256(unsafe { USER_TMP }.password.as_str().as_bytes()).map_err(|_| AtError::InvalidArgs)?;
        

        for User{email: user, password: pwd} in self.0.iter() {
            if *user == unsafe { USER_TMP }.email && pwd.as_str() == password.as_str() {
                unsafe { USER_LOGGED = Some(USER_TMP); }
                return Ok(at_cmd_response!(Self::AT_RESP; ""));
            }
        }

        Err(AtError::InvalidArgs)
    }

    fn query(&mut self) -> AtResult<'_, {CMD_SIZE}> {

        let logged = unsafe { *&raw const USER_LOGGED }.clone();

        Ok(
            at_cmd_response!(
                Self::AT_RESP; if logged.is_some() { 
                    logged.unwrap().email
                } else { 
                    Bytes::new() 
                }
            )
        )
    }

    fn test(&mut self) -> AtResult<'_, {CMD_SIZE}> {
        Ok(at_cmd_response!(Self::AT_RESP; "<user>,<password>"))
    }

    fn set(&mut self, args: at_parser_rs::Args) -> AtResult<'_, {CMD_SIZE}> {
        let arg0 = args.get(0).ok_or(AtError::InvalidArgs)?;
        let arg1 = args.get(1).ok_or(AtError::InvalidArgs)?;
        let arg2 = args.get(2).ok_or(AtError::InvalidArgs)?;

        

        if arg0 == "LI" { // Login
            unsafe {
                USER_TMP.email = Bytes::from_str(arg1);
                USER_TMP.password = Bytes::from_str(arg2);
            }
        } else if arg0 == "LO" { // Logout
            self.logout();
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

impl Session {
    pub const AT_CMD: &'static str = "AT+SESS";
    pub const AT_RESP: &'static str = "+SESS: ";
    pub const MAX_USERS : usize = 2;

    pub const fn new() -> Self {
        Self ([User::new(); Session::MAX_USERS])
    }

    pub fn logout(&mut self) {
        unsafe { USER_LOGGED = None; }
    }

    pub fn set_user(&mut self, user: &User) {
        self.0[1] = *user;   
    }

    pub fn set_user_local(&self) {
        unsafe { USER_LOCAL = self.0[1]; }
    }
}

