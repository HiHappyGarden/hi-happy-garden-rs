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

use osal_rs::utils::ArcMux;

use crate::traits::button::OnClickable;
use crate::traits::encoder::OnRotatableAndClickable;

pub trait HardwareFn {

    fn set_button_handler(&mut self, clicclable: ArcMux<dyn OnClickable>);

    fn set_encoder_handler(&mut self, rotate_and_click: ArcMux<dyn OnRotatableAndClickable>);

}

