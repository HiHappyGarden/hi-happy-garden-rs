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
use at_parser_rs::{AtError, AtResult, context::AtContext};
use osal_rs::utils::{Bytes, Result};
use osal_rs_serde::{Deserialize, Serialize};

use crate::apps::parser::{CMD_SIZE};
use crate::drivers::encrypt::{EncryptGeneric}; 

const APP_TAG: &str = "AppSession";
static mut USER_LOGGED: Option<User> = None;
static mut USER_TMP: User = User::new();

#[derive(Serialize, Deserialize, Clone, Copy)]
pub(super) struct User {
    user: Bytes<32>,
    password: Bytes<32>,
}

impl Default for User {
    fn default() -> Self {
        Self {
            user: Bytes::new(),
            password: Bytes::new(),
        }
    }
}

impl AtContext<{CMD_SIZE}> for User {

    fn query(&mut self) -> AtResult<{CMD_SIZE}> {
        let mut response = Bytes::<CMD_SIZE>::new();
        response.format(format_args!("{}{},{}", Self::AT_RESP, self.user.as_str(), self.password.as_str()));
        Ok(response)
    }

    fn test(&mut self) -> AtResult<{CMD_SIZE}> {
        let mut response = Bytes::<CMD_SIZE>::new();
        response.format(format_args!("{}<user>,<password>", Self::AT_RESP));
        Ok(response)
    }
    
    fn set(&mut self, args: at_parser_rs::Args) -> AtResult<{CMD_SIZE}> {
        let arg0 = args.get(0).ok_or(AtError::InvalidArgs)?;
        let arg1 = args.get(1).ok_or(AtError::InvalidArgs)?;


        if unsafe { &*&raw const USER_LOGGED }.is_some() {
            self.user = Bytes::from_str(arg0);
            self.password = Bytes::from_str(arg1);
        } else {
            return Err(AtError::InvalidArgs);
        }

        Ok(Bytes::new())
    }
}

impl User {

    const AT_CMD: &'static str = "AT+USR";
    pub const AT_RESP: &'static str = "+USR: ";
    pub const fn new() -> Self {
        Self { 
            user: Bytes::new(),
            password: Bytes::new(),
        }
    }

    #[inline]
    pub fn get_user(&self) -> &Bytes<32> {
        &self.user
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub(super) struct Session ([User; Session::MAX_USERS]);

impl AtContext<{CMD_SIZE}> for Session {

    fn exec(&self) -> AtResult<{CMD_SIZE}> {
        
        let password =  EncryptGeneric::get_sha256(unsafe { USER_TMP }.password.as_str().as_bytes()).map_err(|_| AtError::InvalidArgs)?;
        

        for User{user, password: pwd} in self.0.iter() {
            if *user == unsafe { USER_TMP }.user && pwd.as_str() == password.as_str() {
                unsafe { USER_LOGGED = Some(USER_TMP); }
                return Ok(Bytes::new());
            }
        }

        Err(AtError::InvalidArgs)
    }

    fn query(&mut self) -> AtResult<{CMD_SIZE}> {
        let mut response = Bytes::<CMD_SIZE>::new();
        
        response.format(format_args!("{}{}", Self::AT_RESP, if unsafe { &*&raw const USER_LOGGED }.is_some() { "LOGGED" } else { "NO_LOGGED" }));
        
        Ok(response)
    }

    fn test(&mut self) -> AtResult<{CMD_SIZE}> {
        let mut response = Bytes::<CMD_SIZE>::new();
        response.format(format_args!("{}<user>,<password>", Self::AT_RESP));
        Ok(response)
    }

    fn set(&mut self, args: at_parser_rs::Args) -> AtResult<{CMD_SIZE}> {
        let arg0 = args.get(0).ok_or(AtError::InvalidArgs)?;
        let arg1 = args.get(1).ok_or(AtError::InvalidArgs)?;
        let arg2 = args.get(2).ok_or(AtError::InvalidArgs)?;

        

        if arg0 == "LI" { // Login
            unsafe {
                USER_TMP.user = Bytes::from_str(arg1);
                USER_TMP.password = Bytes::from_str(arg2);
            }
        } else if arg0 == "LO" { // Logout
            self.logout();
        } else {
            return Err(AtError::InvalidArgs);
        }

        Ok(Bytes::new())
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

    pub fn set_users(&mut self, users: &[User; Session::MAX_USERS]) {
        for (i, user) in users.iter().enumerate() {
            self.0[i] = *user;
        }
    }

    pub fn login(&mut self, user: User) {
        unsafe { USER_LOGGED = Some(user); }
    }

    pub fn logout(&mut self) {
        unsafe { USER_LOGGED = None; }
    }

    pub fn get_logged_user(&self) -> Option<User> {
        unsafe { USER_LOGGED }
    }

    pub(super) fn get_user_logged(&self) -> Option<User> {
        unsafe { *&raw const USER_LOGGED }.clone()
    }

}

