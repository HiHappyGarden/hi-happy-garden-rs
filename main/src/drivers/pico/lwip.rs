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

use core::slice::from_raw_parts;
use osal_rs::utils::{Bytes, Error, Result};
use crate::drivers::network::NetworkFn;
use crate::drivers::plt::ffi::{hhg_dhcp_get_binary_ip_address, hhg_dhcp_get_ip_address, hhg_dhcp_supplied_address, hhg_netif_is_link_up};
use crate::traits::network::{IPV6_ADDR_LEN, IpAddress};

pub static NETWORK_FN: NetworkFn = NetworkFn {
    dhcp_get_ip_address,
    dhcp_get_binary_ip_address,
    dhcp_supplied_address,
    dns_resolve_addrress,
    ntp_request,
    is_link_up
};

fn dhcp_get_ip_address() -> Bytes<IPV6_ADDR_LEN> {

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
fn dhcp_get_binary_ip_address() -> u32 {
    unsafe { hhg_dhcp_get_binary_ip_address() }
}


fn dhcp_supplied_address() -> bool {
    unsafe { hhg_dhcp_supplied_address() }
}

    
fn dns_resolve_addrress<'a>(_hostname: &Bytes<64>) -> Result<&'a dyn IpAddress> {
    Err(Error::Empty)
}

fn ntp_request(_ipaddr_dest: &'static dyn IpAddress, _port: u16, _msg_len: u16) -> Result<()> {
    //unsafe { hhg_ntp_request(server.as_ptr() as *const _, port as i16, msg_len as i16) };
    Ok(())
}

fn is_link_up() -> bool {
    unsafe { hhg_netif_is_link_up() == 1 }
}