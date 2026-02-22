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

use core::ffi::{c_char, c_void};
use core::ptr::null_mut;
use core::slice::from_raw_parts;
use osal_rs::log_info;
use osal_rs::os::{System, SystemFn};
use osal_rs::utils::{Bytes, Error, Result};
use crate::drivers::network::{IP4Addr, NetworkFn};
use crate::drivers::pico::ffi::{hhg_cyw43_arch_lwip_begin, hhg_cyw43_arch_lwip_end, hhg_cyw43_arch_poll, hhg_dns_gethostbyname};
use crate::drivers::plt::ffi::{hhg_dhcp_get_binary_ip_address, hhg_dhcp_get_ip_address, hhg_dhcp_supplied_address, hhg_netif_is_link_up};
use crate::traits::network::{IPV6_ADDR_LEN, IpAddress};

static mut IP_ADDRES_FOUND: Option<IP4Addr> = None;

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


extern "C" fn dns_found_callback(_name: *const c_char, ipaddr: *const IP4Addr, _: *mut c_void) {
    // SAFETY: The callback is called by the C code, so we can assume that the pointers are valid
    
    
    let ipaddr = unsafe { &*(ipaddr as *const IP4Addr) };


    unsafe {
        IP_ADDRES_FOUND = Some(*ipaddr);
    }
}

fn dns_resolve_addrress<'a>(hostname: &Bytes<64>) -> Result<&'a dyn IpAddress> {

    unsafe {
        IP_ADDRES_FOUND = None;
    }

    unsafe {
        hhg_cyw43_arch_lwip_begin();   
    }

    let mut addr = IP4Addr::default();
    let dns_result = unsafe { 
        hhg_dns_gethostbyname(
            hostname.as_cstring().as_ptr(), 
            &mut addr, 
            dns_found_callback, 
            null_mut()
        ) 
    };

    unsafe {
        hhg_cyw43_arch_lwip_end();   
    }

    // 0 (ERR_OK): Address already in cache, callback won't be called
    // 251/0xFB (ERR_INPROGRESS as u8): query sent, waiting for response, callback will be called when response is received
    // anthings else: error, callback won't be called
    
    if dns_result == 0 {
        // Address already in cache, callback won't be called
        unsafe {
            IP_ADDRES_FOUND = Some(addr);
        }
        return if let Some(ipaddr) = unsafe { &*&raw const IP_ADDRES_FOUND } {
            Ok(ipaddr)
        } else {
            Err(Error::Empty)
        };
    } else if dns_result as i8 == -5 {
        // Query sent, waiting for response, callback will be called when response is received
        const TIMEOUT_MS: u32 = 5000;
        const POLL_INTERVAL_MS: u32 = 10;
        let max_attempts = TIMEOUT_MS / POLL_INTERVAL_MS;
        
        for _ in 0..max_attempts {
            unsafe { hhg_cyw43_arch_poll(); }
            
            if unsafe { IP_ADDRES_FOUND }.is_some() {
                break;
            }
            
            System::delay(POLL_INTERVAL_MS);
        }

        if let Some(ipaddr) = unsafe { &*&raw const IP_ADDRES_FOUND } {
            Ok(ipaddr)
        } else {  
            Err(Error::Empty)
        }
    } else {
        Err(Error::Empty)
    }
}

fn ntp_request(_ipaddr_dest: &'static dyn IpAddress, _port: u16, _msg_len: u16) -> Result<()> {
    //unsafe { hhg_ntp_request(server.as_ptr() as *const _, port as i16, msg_len as i16) };
    Ok(())
}

fn is_link_up() -> bool {
    unsafe { hhg_netif_is_link_up() == 1 }
}