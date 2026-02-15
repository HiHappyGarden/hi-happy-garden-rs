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

use osal_rs::utils::Bytes;

pub(crate) const IPV6_ADDR_LEN: usize = 40;

pub trait IpAddress {
    
    fn is_ipv4(&self, bytes: Bytes<IPV6_ADDR_LEN>) -> bool {
        let mut counter = 0u8;
        for &byte in bytes.as_slice() {
            if byte == b':' {
                counter += 1;
            }
        }
        counter == 4
    }


    fn is_ipv6(&self, bytes: Bytes<IPV6_ADDR_LEN>) -> bool {
        let mut counter = 0u8;
        for &byte in bytes.as_slice() {
            if byte == b':' {
                counter += 1;
            }
        }
        counter == 7
    }

}

