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

//! lwIP on the host network.
//!
//! Twin of `src/pico/hhg-lwip.c`. While the radio is associated
//! ([`super::cyw43::is_joined`]) the board has the host address, names are
//! resolved by the host resolver and UDP goes through host sockets; the
//! receive callback runs on a thread of its own, as it runs in the lwIP
//! context on the board. Without association there is no link: DNS and
//! UDP fail as lwIP fails without a route.
//!
//! `ip_addr` keeps lwIP's layout: the address in network byte order in
//! memory, so `addr` is `u32::from_ne_bytes(octets)`.

use alloc::boxed::Box;
use alloc::vec;
use core::ffi::{CStr, c_char, c_uchar, c_uint, c_ushort, c_void};
use core::ptr::null_mut;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket};
use std::sync::Mutex;
use std::time::Duration;

use crate::drivers::network::IP4Addr;
use crate::drivers::pico::ffi::err_enum::{ERR_ARG, ERR_OK, ERR_RTE, ERR_VAL};
use crate::drivers::pico::ffi::{ip_addr, pbuf, udp_pcb, udp_recv_fn};
use crate::traits::network::IPV6_ADDR_LEN;

use super::cyw43::is_joined;
use super::lock;

/// A receive thread with nothing to read for this long ends: the firmware
/// never removes its PCBs, and each NTP request creates one
const RECV_IDLE_TIMEOUT: Duration = Duration::from_secs(10);

/// Largest datagram on the board (Ethernet MTU)
const MAX_DATAGRAM: usize = 1_500;

/// `ip4addr_ntoa` text of the board address. Like lwIP's own buffer it is
/// static and rewritten by every call: callers copy it right away.
static IP_TEXT: Mutex<[u8; IPV6_ADDR_LEN]> = Mutex::new([0u8; IPV6_ADDR_LEN]);

/// The address the host uses to reach the network, the one the emulated
/// board gets by DHCP.
fn host_ipv4() -> Ipv4Addr {
    // Connecting a UDP socket only selects the route, nothing is sent
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|socket| socket.connect("192.0.2.1:9").map(|_| socket))
        .and_then(|socket| socket.local_addr())
        .ok()
        .and_then(|addr| match addr {
            SocketAddr::V4(addr) => Some(*addr.ip()),
            SocketAddr::V6(_) => None,
        })
        .unwrap_or(Ipv4Addr::LOCALHOST)
}

fn to_ip_addr(ip: Ipv4Addr) -> ip_addr {
    IP4Addr { addr: u32::from_ne_bytes(ip.octets()) }
}

fn from_ip_addr(ip: &ip_addr) -> Ipv4Addr {
    Ipv4Addr::from(ip.addr.to_ne_bytes())
}

pub(in crate::drivers) unsafe fn hhg_dhcp_get_ip_address() -> *const c_char {
    let ip = if is_joined() { host_ipv4() } else { Ipv4Addr::UNSPECIFIED };
    let text = alloc::format!("{ip}");

    let mut buffer = lock(&IP_TEXT);
    buffer.fill(0);
    buffer[..text.len()].copy_from_slice(text.as_bytes());
    buffer.as_ptr() as *const c_char
}

pub(in crate::drivers) unsafe fn hhg_dhcp_get_binary_ip_address() -> c_uint {
    if is_joined() { to_ip_addr(host_ipv4()).addr } else { 0 }
}

pub(in crate::drivers) unsafe fn hhg_dhcp_supplied_address() -> bool {
    is_joined()
}

pub(in crate::drivers) unsafe fn hhg_netif_is_link_up() -> c_uchar {
    is_joined() as c_uchar
}

pub(in crate::drivers) unsafe fn hhg_ip_addr_cmp(addr: *const ip_addr, addr2: *const ip_addr) -> i32 {
    match unsafe { (addr.as_ref(), addr2.as_ref()) } {
        (Some(a), Some(b)) => (a.addr == b.addr) as i32,
        _ => 0,
    }
}

/// Resolves right away with the host resolver: to the firmware it looks like
/// an answer already in the lwIP cache (`ERR_OK`, callback not called).
pub(in crate::drivers) unsafe fn hhg_dns_gethostbyname(
    hostname: *const c_char,
    addr: *mut ip_addr,
    _dns_found_callback: extern "C" fn(name: *const c_char, ipaddr: *const ip_addr, callback_arg: *mut c_void),
    _callback_arg: *mut c_void,
) -> c_char {
    if hostname.is_null() || addr.is_null() {
        return ERR_ARG as i8 as c_char;
    }
    if !is_joined() {
        return ERR_RTE as i8 as c_char;
    }

    let Ok(hostname) = unsafe { CStr::from_ptr(hostname) }.to_str() else {
        return ERR_ARG as i8 as c_char;
    };
    let resolved = (hostname, 0)
        .to_socket_addrs()
        .ok()
        .and_then(|mut addrs| addrs.find_map(|addr| match addr {
            SocketAddr::V4(addr) => Some(*addr.ip()),
            SocketAddr::V6(_) => None,
        }));

    match resolved {
        Some(ip) => {
            unsafe { *addr = to_ip_addr(ip) };
            ERR_OK as i8 as c_char
        }
        None => ERR_VAL as i8 as c_char,
    }
}

/// `struct udp_pcb` followed by the host socket behind it: the firmware only
/// ever sees the first field.
#[repr(C)]
struct EmulatedPcb {
    pcb: udp_pcb,
    socket: UdpSocket,
}

pub(in crate::drivers) unsafe fn hhg_udp_new_ip_type(_type: c_uchar) -> *mut udp_pcb {
    let Ok(socket) = UdpSocket::bind("0.0.0.0:0") else {
        return null_mut();
    };

    let pcb = udp_pcb {
        local_ip: IP4Addr::default(),
        remote_ip: IP4Addr::default(),
        netif_idx: 0,
        so_options: 0,
        tos: 0,
        ttl: 255,
        next: null_mut(),
        flags: 0,
        local_port: socket.local_addr().map(|addr| addr.port()).unwrap_or(0),
        remote_port: 0,
        recv: None,
        recv_arg: null_mut(),
    };
    Box::into_raw(Box::new(EmulatedPcb { pcb, socket })) as *mut udp_pcb
}

pub(in crate::drivers) unsafe fn hhg_udp_sendto(pcb: *mut udp_pcb, p: *mut pbuf, ipaddr: *const ip_addr, port: u16) -> i8 {
    let (Some(pcb), Some(p), Some(ipaddr)) = (unsafe { (pcb as *mut EmulatedPcb).as_ref() }, unsafe { p.as_ref() }, unsafe { ipaddr.as_ref() }) else {
        return ERR_ARG as i8;
    };
    if !is_joined() {
        return ERR_RTE as i8;
    }

    let data = unsafe { core::slice::from_raw_parts(p.payload as *const u8, p.len as usize) };
    let target = SocketAddrV4::new(from_ip_addr(ipaddr), port);
    match pcb.socket.send_to(data, target) {
        Ok(_) => ERR_OK as i8,
        Err(_) => ERR_RTE as i8,
    }
}

/// Raw pointers crossing into the receive thread, owned by the firmware as
/// they are on the board
struct RecvContext {
    pcb: usize,
    recv: udp_recv_fn,
    recv_arg: usize,
}

pub(in crate::drivers) unsafe fn hhg_udp_recv(pcb: *mut udp_pcb, recv: udp_recv_fn, recv_arg: *mut c_void) {
    let Some(emulated) = (unsafe { (pcb as *mut EmulatedPcb).as_mut() }) else {
        return;
    };
    emulated.pcb.recv = Some(recv);
    emulated.pcb.recv_arg = recv_arg;

    let Ok(socket) = emulated.socket.try_clone() else {
        return;
    };
    if socket.set_read_timeout(Some(RECV_IDLE_TIMEOUT)).is_err() {
        return;
    }

    let context = RecvContext { pcb: pcb as usize, recv, recv_arg: recv_arg as usize };
    let spawned = std::thread::Builder::new()
        .name("emu_lwip_udp".into())
        .spawn(move || {
            let mut buffer = vec![0u8; MAX_DATAGRAM];
            while let Ok((len, SocketAddr::V4(from))) = socket.recv_from(&mut buffer) {
                let p = unsafe { hhg_pbuf_alloc(len as c_ushort) };
                let Some(datagram) = (unsafe { (p as *mut EmulatedPbuf).as_mut() }) else {
                    continue;
                };
                datagram.data.copy_from_slice(&buffer[..len]);

                let addr = to_ip_addr(*from.ip());
                // The callback owns the pbuf from here, as with lwIP
                unsafe { (context.recv)(context.recv_arg as *mut c_void, context.pcb as *mut udp_pcb, p, &addr, from.port()) };
            }
        });
    if let Err(e) = spawned {
        std::eprintln!("emulator: UDP receive thread not started: {e}");
    }
}

/// `struct pbuf` followed by the buffer its `payload` points into: a single
/// PBUF_RAM pbuf, never chained.
#[repr(C)]
struct EmulatedPbuf {
    pbuf: pbuf,
    data: Box<[u8]>,
}

pub(in crate::drivers) unsafe fn hhg_pbuf_alloc(length: c_ushort) -> *mut pbuf {
    let mut data = vec![0u8; length as usize].into_boxed_slice();
    let pbuf = pbuf {
        next: null_mut(),
        // The boxed slice does not move when the box itself does
        payload: data.as_mut_ptr() as *mut c_void,
        tot_len: length,
        len: length,
        type_internal: 0,
        flags: 0,
        ref_count: 1,
        if_idx: 0,
    };
    Box::into_raw(Box::new(EmulatedPbuf { pbuf, data })) as *mut pbuf
}

pub(in crate::drivers) unsafe fn hhg_pbuf_free(p: *mut pbuf) -> c_uchar {
    if p.is_null() {
        return 0;
    }
    drop(unsafe { Box::from_raw(p as *mut EmulatedPbuf) });
    1
}

pub(in crate::drivers) unsafe fn hhg_pbuf_copy_partial(buf: *mut pbuf, dataptr: *mut c_void, len: u16, offset: u16) -> u16 {
    let Some(p) = (unsafe { (buf as *const EmulatedPbuf).as_ref() }) else {
        return 0;
    };
    if dataptr.is_null() {
        return 0;
    }
    let available = p.data.len().saturating_sub(offset as usize);
    let count = available.min(len as usize);
    unsafe { core::ptr::copy_nonoverlapping(p.data.as_ptr().add(offset as usize), dataptr as *mut u8, count) };
    count as u16
}

pub(in crate::drivers) unsafe fn hhg_pbuf_get_at(p: *const pbuf, offset: u16) -> u8 {
    match unsafe { (p as *const EmulatedPbuf).as_ref() } {
        Some(p) => p.data.get(offset as usize).copied().unwrap_or(0),
        None => 0,
    }
}
