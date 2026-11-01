/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * any later version.
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

use osal_rs::os::types::TickType;
use core::ffi::{CStr, c_char, c_void};
use core::ptr::null_mut;
use core::slice::from_raw_parts_mut;
use core::sync::atomic::{AtomicU8, AtomicU32, Ordering};
use osal_rs::os::{System, SystemFn};
use osal_rs::log_error;
use osal_rs::utils::{Bytes, Error, Result};
use crate::drivers::network::{IP4Addr, NetworkFn};
use crate::drivers::pico::ffi::{lwip_ip_addr_type::IPADDR_TYPE_V4, hhg_cyw43_arch_lwip_begin, hhg_cyw43_arch_lwip_end, hhg_cyw43_arch_poll, hhg_dns_gethostbyname, hhg_pbuf_alloc, hhg_pbuf_copy_partial, hhg_pbuf_free, hhg_pbuf_get_at, hhg_udp_new_ip_type, hhg_udp_recv, hhg_udp_remove, hhg_udp_sendto, ip_addr, pbuf, udp_pcb, hhg_dhcp_get_binary_ip_address, hhg_dhcp_get_ip_address, hhg_dhcp_supplied_address, hhg_netif_is_link_up};
use crate::traits::network::IPV6_ADDR_LEN;

const APP_TAG: &str = "LWIP";

const DNS_PENDING: u8 = 0;
const DNS_FOUND: u8 = 1;
const DNS_FAILED: u8 = 2;

// Written by the lwIP callbacks, which run on the lwIP (tcpip) thread with
// `pico_cyw43_arch_lwip_sys_freertos`, and polled by the requesting thread
static DNS_STATE: AtomicU8 = AtomicU8::new(DNS_PENDING);
static DNS_ADDR: AtomicU32 = AtomicU32::new(0);
/// Seconds since 1970 of the last NTP answer, 0 while none came
static NTP_SECONDS: AtomicU32 = AtomicU32::new(0);

pub(in crate::drivers) static NETWORK_FN: NetworkFn = NetworkFn {
    dhcp_get_ip_address,
    dhcp_get_binary_ip_address,
    dhcp_supplied_address,
    dns_resolve_addrress,
    ntp_request,
    is_link_up
};

fn dhcp_get_ip_address() -> Bytes<IPV6_ADDR_LEN> {
    let mut bytes = Bytes::<IPV6_ADDR_LEN>::new();

    let ret = unsafe { hhg_dhcp_get_ip_address() };

    if !ret.is_null() {
        // `ip4addr_ntoa` text: NUL terminated, in a buffer shorter than `IPV6_ADDR_LEN`
        let text = unsafe { CStr::from_ptr(ret) }.to_bytes();
        let len = text.len().min(IPV6_ADDR_LEN);
        bytes.as_mut_slice()[..len].copy_from_slice(&text[..len]);
    }

    bytes
}

#[inline]
fn dhcp_get_binary_ip_address() -> u32 {
    unsafe { hhg_dhcp_get_binary_ip_address() }
}


fn dhcp_supplied_address() -> bool {
    unsafe { hhg_dhcp_supplied_address() }
}


extern "C" fn dns_found_callback(_name: *const c_char, ipaddr: *const ip_addr, _: *mut c_void) {
    // lwIP passes a NULL ipaddr when the lookup fails (NXDOMAIN, no answer from the server, ...)
    if ipaddr.is_null() {
        log_error!(APP_TAG, "DNS lookup failed: no address returned");
        DNS_STATE.store(DNS_FAILED, Ordering::Release);
        return;
    }

    // SAFETY: The callback is called by the C code, ipaddr was checked for NULL above
    DNS_ADDR.store(unsafe { (*ipaddr).addr }, Ordering::Relaxed);
    DNS_STATE.store(DNS_FOUND, Ordering::Release);
}

fn dns_resolve_addrress(hostname: &Bytes<64>) -> Result<IP4Addr> {

    DNS_STATE.store(DNS_PENDING, Ordering::Release);

    let mut addr = IP4Addr::default();

    let dns_result = unsafe {
        hhg_cyw43_arch_lwip_begin();
        let ret = hhg_dns_gethostbyname(
            hostname.as_cstr().as_ptr(),
            &mut addr,
            dns_found_callback,
            null_mut()
        );
        hhg_cyw43_arch_lwip_end();
        ret
    };

    // 0 (ERR_OK): Address already in cache, callback won't be called
    // -5 (ERR_INPROGRESS): query sent, waiting for response, callback will be called when response is received
    // anthings else: error, callback won't be called

    if dns_result == 0 {
        Ok(addr)
    } else if dns_result == -5 {
        // Query sent, waiting for response, callback will be called when response is received
        const TIMEOUT_MS: TickType = 5000;
        const POLL_INTERVAL_MS: TickType = 10;
        let max_attempts = TIMEOUT_MS / POLL_INTERVAL_MS;

        for _ in 0..max_attempts {
            unsafe { hhg_cyw43_arch_poll(); }

            match DNS_STATE.load(Ordering::Acquire) {
                DNS_FOUND => return Ok(IP4Addr { addr: DNS_ADDR.load(Ordering::Relaxed) }),
                DNS_FAILED => return Err(Error::Empty),
                _ => {}
            }

            System::delay(POLL_INTERVAL_MS);
        }

        log_error!(APP_TAG, "DNS lookup for {} got no answer within {}ms", hostname, TIMEOUT_MS);
        Err(Error::Empty)
    } else {
        // -6 (ERR_VAL): no DNS server configured, -16 (ERR_ARG): bad hostname or DNS not initialized, -1 (ERR_MEM): DNS table full
        log_error!(APP_TAG, "dns_gethostbyname({}) failed with err={}", hostname, dns_result);
        Err(Error::Empty)
    }
}

extern "C" fn ntp_recv(_: *mut c_void, _: *mut udp_pcb, p: *mut pbuf, _: *const ip_addr, _: u16) {
    if p.is_null() {
        return;
    }

    let  mode  = unsafe {
        hhg_pbuf_get_at(p, 0) & 0x7
    };
    let  stratum  = unsafe {
        hhg_pbuf_get_at(p, 1)
    };


    if mode == 0x4 && stratum != 0 {
        const NTP_DELTA: u32 = 2_208_988_800; // Seconds between 1900 and 1970

        let mut seconds_buf: [u8; 4] = [0, 0, 0, 0];
        let copied = unsafe {
            hhg_pbuf_copy_partial(p, seconds_buf.as_mut_ptr() as *mut c_void, seconds_buf.len() as u16, 40)
        };
        let seconds_since_1900 = u32::from_be_bytes(seconds_buf);

        if copied as usize == seconds_buf.len() {
            if let Some(seconds_since_1970) = seconds_since_1900.checked_sub(NTP_DELTA) {
                NTP_SECONDS.store(seconds_since_1970, Ordering::Release);
            }
        }
    }

    // The receive callback owns `p`, whatever it holds
    unsafe {
        hhg_pbuf_free(p);
    }
}

fn ntp_request(ipaddr_dest: IP4Addr, port: u16, msg_len: u16) -> Result<i64> {

    NTP_SECONDS.store(0, Ordering::Release);

    unsafe {
        hhg_cyw43_arch_lwip_begin();
    }


    let pcb = unsafe { hhg_udp_new_ip_type(IPADDR_TYPE_V4) };
    if pcb.is_null() {
        unsafe {
            hhg_cyw43_arch_lwip_end();
        }
        return Err(Error::NullPtr);
    }

    let pbuf = unsafe { hhg_pbuf_alloc(msg_len) };
    if pbuf.is_null() {
        unsafe {
            hhg_udp_remove(pcb);
            hhg_cyw43_arch_lwip_end();
        }
        return Err(Error::NullPtr);
    }

    let req = unsafe {
        from_raw_parts_mut(
            (*pbuf).payload as *mut u8,
            (*pbuf).len as usize
        )
    };

    req.fill(0);
    req[0] = 0x1b;

    let ret = unsafe {
        // Before sending: the answer could otherwise come back with no callback set
        hhg_udp_recv(pcb, ntp_recv, null_mut());
        let ret = hhg_udp_sendto(pcb, pbuf, &ipaddr_dest, port);
        // udp_sendto() does not take the pbuf
        hhg_pbuf_free(pbuf);
        if ret < 0 {
            hhg_udp_remove(pcb);
        }
        hhg_cyw43_arch_lwip_end();
        ret
    };
    if ret < 0 {
        return Err(Error::ReturnWithCode(ret as i32));
    }

    const TIMEOUT_MS: TickType = 5000;
    const POLL_INTERVAL_MS: TickType = 10;
    let max_attempts = TIMEOUT_MS / POLL_INTERVAL_MS;

    for _ in 0..max_attempts {
        unsafe { hhg_cyw43_arch_poll(); }

        if NTP_SECONDS.load(Ordering::Acquire) != 0 {
            break;
        }

        System::delay(POLL_INTERVAL_MS);
    }

    // Under the core lock: no callback of this pcb runs while it is freed
    unsafe {
        hhg_cyw43_arch_lwip_begin();
        hhg_udp_remove(pcb);
        hhg_cyw43_arch_lwip_end();
    }

    match NTP_SECONDS.load(Ordering::Acquire) {
        0 => Err(Error::Empty),
        seconds => Ok(seconds as i64),
    }
}

fn is_link_up() -> bool {
    unsafe { hhg_netif_is_link_up() == 1 }
}
