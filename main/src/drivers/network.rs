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

use core::ffi::c_void;

use osal_rs::utils::{Bytes, Error, Result};

use crate::drivers::plt::lwip::NETWORK_FN;
use crate::drivers::wifi::Auth;
use crate::traits::network::{IPV6_ADDR_LEN, IpAddress};

static mut WIFI_SSID: Bytes<32> = Bytes::new();
static mut WIFI_PASSWORD: Bytes<32> = Bytes::new();
static mut WIFI_HOSTNAME: Bytes<64> = Bytes::new();
static mut WIFI_AUTH: Auth = Auth::Wpa2;
static mut WIFI_ENABLED: bool = false;

static mut NTP_SERVER: Bytes<64> = Bytes::new();
static mut NTP_PORT: u16 = 123;
static mut NTP_MSG_LEN: usize = 48;
static mut NTP_ENABLED: bool = false;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum IpType {
    IPv4 = 0,
    IPv6 = 6,
    ANY = 46,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IP4Addr {
    pub addr: u32,
}

impl IpAddress for IP4Addr {}

#[allow(dead_code)]
impl IP4Addr {
    pub fn from_bytes(bytes: &mut Bytes<IPV6_ADDR_LEN>) -> Result<Self> {
        bytes.replace(b":", b"")?;

        if bytes.len() != 8 {
            return Err(Error::InvalidQueueSize);
        }

        let mut addr = 0u32;

        for (i, &byte) in bytes.as_slice().iter().enumerate() {
            addr |= (byte as u32) << (8 * (3 - i));
        }
        Ok(IP4Addr { addr })
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IP6Addr {
    pub addr: u128,
}

impl IpAddress for IP6Addr {}

#[allow(dead_code)]
impl IP6Addr {
    pub fn from_bytes(bytes: &mut Bytes<IPV6_ADDR_LEN>) -> Result<Self> {
        bytes.replace(b":", b"")?;

        if bytes.len() != 32 {
            return Err(Error::InvalidQueueSize);
        }

        let mut addr = 0u128;

        for (i, &byte) in bytes.as_slice().iter().enumerate() {
            addr |= (byte as u128) << (8 * (15 - i));
        }
        Ok(IP6Addr { addr })
    }
}

#[allow(dead_code)]
pub struct Udp(pub *mut c_void);

#[allow(dead_code)]
pub struct NetworkFn<'a> {
    pub dhcp_get_ip_address: fn() -> Bytes<IPV6_ADDR_LEN>,
    pub dhcp_get_binary_ip_address: fn() -> u32,
    pub dhcp_supplied_address: fn() -> bool,
    pub dns_resolve_addrress: fn(hostname: &Bytes<64>) -> Result<&'a dyn IpAddress>,
    pub ntp_request: fn(ipaddr_dest: &'a dyn IpAddress, port: u16, msg_len: u16) -> Result<()>,
    pub is_link_up: fn() -> bool,
}

#[allow(dead_code)]
pub struct Network;

#[allow(dead_code)]
impl Network {
    pub fn set_ntp(server: Bytes<64>, port: u16, msg_len: u16, enabled: bool) {
        unsafe {
            NTP_SERVER = server;
            NTP_PORT = port;
            NTP_MSG_LEN = msg_len as usize;
            NTP_ENABLED = enabled;
        }
    }

    pub fn set_wifi(
        ssid: Bytes<32>,
        password: Bytes<32>,
        hostname: Bytes<64>,
        auth: Auth,
        enabled: bool,
    ) {
        unsafe {
            WIFI_SSID = ssid;
            WIFI_PASSWORD = password;
            WIFI_HOSTNAME = hostname;
            WIFI_AUTH = auth;
            WIFI_ENABLED = enabled;
        }
    }

    #[inline]
    pub fn dhcp_get_ip_address() -> Bytes<IPV6_ADDR_LEN> {
        (NETWORK_FN.dhcp_get_ip_address)()
    }

    #[inline]
    pub fn dhcp_get_binary_ip_address() -> u32 {
        (NETWORK_FN.dhcp_get_binary_ip_address)()
    }

    #[inline]
    pub fn dhcp_supplied_address() -> bool {
        (NETWORK_FN.dhcp_supplied_address)()
    }

    pub fn dns_resolve_addrress<'a>(hostname: &Bytes<64>) -> Result<&'a dyn IpAddress> {
        (NETWORK_FN.dns_resolve_addrress)(hostname)
    }

    pub fn ntp_request(ipaddr_dest: &'static dyn IpAddress, port: u16, msg_len: u16) -> Result<()> {
        (NETWORK_FN.ntp_request)(ipaddr_dest, port, msg_len)
    }

    #[inline]
    pub fn is_link_up() -> bool {
        (NETWORK_FN.is_link_up)()
    }



}
