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
use core::ffi::{c_ushort, c_void};
use core::slice::from_raw_parts;
use osal_rs::utils::{Bytes, Error, Result};
use crate::drivers::network::{IpType, NetworkFn, Udp, UdpRecvFn, IPV6_ADDR_LEN};
use crate::drivers::pico::ffi::{ip4_addr, pbuf, udp_pcb};
use crate::drivers::plt::ffi::{hhg_dhcp_get_binary_ip_address, hhg_dhcp_get_ip_address, hhg_dhcp_supplied_address, hhg_netif_is_link_up, hhg_udp_new_ip_type, lwip_ip_addr_type};

pub static NETWORK_FN: NetworkFn = NetworkFn {
    get_ip_address,
    get_binary_ip_address,
    supplied_address,
    udp_new_ip_type,
    udp_recv,
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
    use IpType::*;
    use lwip_ip_addr_type::*;

    let ip_type = match ip_type {
        IPv4 => IPADDR_TYPE_V4,
        IPv6 => IPADDR_TYPE_V6,
        ANY => IPADDR_TYPE_ANY,
    } as u8;

    let ret = unsafe { hhg_udp_new_ip_type(ip_type) };
    if ret.is_null() {
        return Err(Error::NullPtr);
    }

    Ok(Udp(ret))
}

extern "C" fn _ntp_recv (_arg: *mut c_void, _pcb: *mut udp_pcb, _pbuf: *mut pbuf, _addr: *const ip4_addr, _port: c_ushort) {
    // auto state = static_cast<struct ntp*>(arg);
    // uint8_t mode = pbuf_get_at(p, 0) & 0x7;
    // uint8_t stratum = pbuf_get_at(p, 1);
    //
    // // Check the result
    // if (ip_addr_cmp(addr, &state->server_address) && port == HHG_NTP_PORT && p->tot_len == HHG_NTP_MSG_LEN && mode == 0x4 && stratum != 0)
    // {
    //     uint8_t seconds_buf[4] = {0};
    //     pbuf_copy_partial(p, seconds_buf, sizeof(seconds_buf), 40);
    //     uint32_t seconds_since_1900 = seconds_buf[0] << 24 | seconds_buf[1] << 16 | seconds_buf[2] << 8 | seconds_buf[3];
    //     uint32_t seconds_since_1970 = seconds_since_1900 - NTP_DELTA;
    //     time_t epoch = seconds_since_1970;
    //     if(state->on_callback)
    //     {
    //         state->on_callback(exit::OK, epoch);
    //     }
    //     singleton->ntp.state = ntp::state::NONE;
    // }
    // else
    // {
    //     if(state->error)
    //     {
    //         *state->error = OSAL_ERROR_APPEND(*state->error, "Invalid ntp response", error_type::OS_ECONNABORTED);
    //         OSAL_ERROR_PTR_SET_POSITION(*state->error);
    //     }
    //
    //     OSAL_LOG_DEBUG(APP_TAG, "NTP request - KO");
    //     if(state->on_callback)
    //     {
    //         state->on_callback(exit::KO, 0);
    //     }
    // }
    // pbuf_free(p);
}

fn udp_recv(_pcb: &Udp, _recv: Box<UdpRecvFn>) {
//TODO: implement this function
}

fn is_link_up() -> bool {
    unsafe { hhg_netif_is_link_up() == 1 }
}