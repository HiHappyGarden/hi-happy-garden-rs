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
use alloc::boxed::Box;

use core::any::Any;
use core::ffi::c_void;

use osal_rs::utils::{Bytes, Result};

use crate::drivers::plt::lwip::NETWORK_FN;

pub(crate) const IPV6_ADDR_LEN: usize = 40;

#[allow(dead_code)]
pub enum IpType {
    IPv4,
    IPv6,
    ANY,
}

#[allow(dead_code)]
pub struct Udp(pub *mut c_void);

#[allow(dead_code)]
pub type UdpRecvFn = dyn Fn(dyn Any);

#[allow(dead_code)]
pub struct NetworkFn {
    pub get_ip_address: fn() -> Bytes<IPV6_ADDR_LEN>,
    pub get_binary_ip_address: fn () -> u32,
    pub supplied_address: fn() -> bool,
    pub udp_new_ip_type: fn(ip_type: IpType) -> Result<Udp>,
    pub udp_recv: fn(pcb: &Udp, recv: Box<UdpRecvFn>),
    pub is_link_up: fn() -> bool
}

#[allow(dead_code)]
pub struct Network;

#[allow(dead_code)]
impl Network {
    #[inline]
    pub fn get_ip_address() -> Bytes<IPV6_ADDR_LEN> {
        (NETWORK_FN.get_ip_address)()
    }

    #[inline]
    pub fn get_binary_ip_address() -> u32 {
        (NETWORK_FN.get_binary_ip_address)()
    }

    #[inline]
    pub fn supplied_address() -> bool {
        (NETWORK_FN.supplied_address)()
    }

    #[inline]
    pub fn udp_new_ip_type(ip_type: IpType) -> Result<Udp> {
        (NETWORK_FN.udp_new_ip_type)(ip_type)
    }

    #[inline]
    pub fn udp_recv(pcb: &Udp, recv: Box<UdpRecvFn>) {
        (NETWORK_FN.udp_recv)(pcb, recv)
    }

    #[inline]
    pub fn is_link_up() -> bool {
        (NETWORK_FN.is_link_up)()
    }
}
