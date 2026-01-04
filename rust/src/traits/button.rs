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
#![allow(dead_code)]

use alloc::sync::Arc;
use osal_rs::os::Mutex;


#[derive(PartialEq, Eq)]
 pub enum ButtonState {
     Pressed,
     Released,
     None
 }

 
 pub trait SetClickable {
     fn set_on_click(&mut self, clicclable: Arc<Mutex<dyn OnClickable>>);
     fn get_state(&self) -> ButtonState;
 }

 pub trait OnClickable: Send + Sync {
    fn on_click(&mut self, state: ButtonState);
 }