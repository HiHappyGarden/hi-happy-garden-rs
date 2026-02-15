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
use core::slice::from_raw_parts;
use osal_rs::utils::{Bytes, Error, Result};
use crate::drivers::network::{IpType, NetworkFn, Udp, UdpRecvFn};
use crate::drivers::plt::ffi::{hhg_dhcp_get_binary_ip_address, hhg_dhcp_get_ip_address, hhg_dhcp_supplied_address, hhg_netif_is_link_up, hhg_udp_new_ip_type};
use crate::traits::network::{IPV6_ADDR_LEN, IpAddress};

pub static NETWORK_FN: NetworkFn = NetworkFn {
    get_ip_address,
    get_binary_ip_address,
    supplied_address,
    udp_new_ip_type,
    udp_recv,
    dns_resolve_addrress,
    is_link_up
};

fn get_ip_address() -> Bytes<IPV6_ADDR_LEN> {

    let ret = unsafe { hhg_dhcp_get_ip_address() };

    if !ret.is_null() {
        let slice = unsafe { from_raw_parts(ret as *const _, IPV6_ADDR_LEN) };
        let mut bytes = Bytes::<IPV6_ADDR_LEN>::new();
        bytes.as_mut_slice().copy_from_slice(slice);
        return bytes;
    }

    Bytes::<IPV6_ADDR_LEN>::new()
}

#[inline]
fn get_binary_ip_address() -> u32 {
    unsafe { hhg_dhcp_get_binary_ip_address() }
}


fn supplied_address() -> bool {
    unsafe { hhg_dhcp_supplied_address() }
}

fn udp_new_ip_type(ip_type: IpType) -> Result<Udp> {
    let ret = unsafe { hhg_udp_new_ip_type(ip_type as u8) };
    if ret.is_null() {
        return Err(Error::NullPtr);
    }

    Ok(Udp(ret))
}
    

fn udp_recv(_pcb: &Udp, _callback: fn(i64)) {

}

pub fn dns_resolve_addrress<'a>(_hostname: &Bytes<32>) -> Result<&'a dyn IpAddress> {
    Err(Error::Empty)
}

fn is_link_up() -> bool {
    unsafe { hhg_netif_is_link_up() == 1 }
}