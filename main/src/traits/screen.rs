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

use osal_rs::utils::Bytes;

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::drivers::date_time::DateTime;
use crate::traits::integer::Integer;

#[derive(Debug, Clone)]
pub(super) struct PathParam<N> 
where N: Integer
{
    pub check: Option<bool>,
    pub input: Bytes<{DISPLAY_INPUT_MAX_SIZE}>,
    pub number: Option<N>,
    pub date_time: Option<DateTime>,
}

impl<N> Default for PathParam<N>
where N: Integer
{
    fn default() -> Self {
        Self {
            check: None,
            input: Bytes::<{DISPLAY_INPUT_MAX_SIZE}>::default(),
            number: None,
            date_time: None,
        }
    }
}



 pub trait Screen {
     
 }