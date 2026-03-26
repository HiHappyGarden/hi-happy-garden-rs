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


#[derive(PartialEq, Eq)]
 pub enum ButtonState {
     Pressed,
     Released,
     None
 }

 
 pub trait SetClickable<'a> {
     fn set_on_click(&mut self, clicclable: &'a dyn OnClickable);
     fn get_state(&self) -> ButtonState;
 }

pub trait OnClickable: Send + Sync {
    fn on_click(&self, state: ButtonState);
 }