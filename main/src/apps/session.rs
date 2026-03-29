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
use osal_rs::{log_info, utils::{Bytes, Result}};
use osal_rs_serde::{Deserialize, Serialize};

use crate::traits::state::Initializable;

const APP_TAG: &str = "AppSession";
static mut USER_LOGGED: Option<User> = None;

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

impl User {

    pub const fn new() -> Self {
        Self { 
            user: Bytes::new(),
            password: Bytes::new(),
        }
    }

    pub fn get_user(&self) -> &Bytes<32> {
        &self.user
    }

    pub fn get_password(&self) -> &Bytes<32> {
        &self.password
    }

    pub fn set_user(&mut self, user: Bytes<32>) {
        self.user = user;
    }

    pub fn set_password(&mut self, password: Bytes<32>) {
        self.password = password;
    }
}


pub struct Session ([User; Session::MAX_USERS]);


impl Initializable for Session {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init app session");
        
        Ok(())
    }
}


impl Session {

    pub const MAX_USERS : usize = 2;

    pub const fn new() -> Self {
        Self ([User::new(); Session::MAX_USERS])
    }

    #[inline]
    pub fn set_users(&mut self, users: &[User; Session::MAX_USERS]) {
        self.0 = users.clone();
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

}

