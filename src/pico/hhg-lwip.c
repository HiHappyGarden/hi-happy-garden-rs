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


#include "hhg-config.h"

#include <pico/cyw43_arch.h>
#include <lwip/dns.h>
#include <lwip/pbuf.h>
#include <lwip/udp.h>



const char * hhg_dhcp_get_ip_address(void) {
    return ip4addr_ntoa(&cyw43_state.netif[CYW43_ITF_STA].ip_addr);
}

u32_t hhg_dhcp_get_binary_ip_address(void) {
    return cyw43_state.netif[CYW43_ITF_STA].ip_addr.addr;
}


u8_t hhg_dhcp_supplied_address(void) {
    return dhcp_supplied_address(&cyw43_state.netif[CYW43_ITF_STA]);
}

void* hhg_udp_new_ip_type(u8_t type) {
    return udp_new_ip_type(type);
}

u16_t hhg_pbuf_copy_partial(const void *buf, void *dataptr, u16_t len, u16_t offset) {
    return pbuf_copy_partial((const struct pbuf *)buf, dataptr, len, offset);
}


    // void ntp_recv(void *arg, struct udp_pcb *pcb, struct pbuf *p, const ip_addr_t *addr, u16_t port)
    // {
    //     auto state = static_cast<struct ntp*>(arg);
    //     uint8_t mode = pbuf_get_at(p, 0) & 0x7;
    //     uint8_t stratum = pbuf_get_at(p, 1);

    //     // Check the result
    //     if (ip_addr_cmp(addr, &state->server_address) && port == HHG_NTP_PORT && p->tot_len == HHG_NTP_MSG_LEN && mode == 0x4 && stratum != 0)
    //     {
    //         uint8_t seconds_buf[4] = {0};
    //         pbuf_copy_partial(p, seconds_buf, sizeof(seconds_buf), 40);
    //         uint32_t seconds_since_1900 = seconds_buf[0] << 24 | seconds_buf[1] << 16 | seconds_buf[2] << 8 | seconds_buf[3];
    //         uint32_t seconds_since_1970 = seconds_since_1900 - NTP_DELTA;
    //         time_t epoch = seconds_since_1970;
    //         if(state->on_callback)
    //         {
    //             state->on_callback(exit::OK, epoch);
    //         }
    //         singleton->ntp.state = ntp::state::NONE;
    //     }
    //     else
    //     {
    //         if(state->error)
    //         {
    //             *state->error = OSAL_ERROR_APPEND(*state->error, "Invalid ntp response", error_type::OS_ECONNABORTED);
    //             OSAL_ERROR_PTR_SET_POSITION(*state->error);
    //         }

    //         OSAL_LOG_DEBUG(APP_TAG, "NTP request - KO");
    //         if(state->on_callback)
    //         {
    //             state->on_callback(exit::KO, 0);
    //         }
    //     }
    //     pbuf_free(p);
    // }

//void             udp_recv       (struct udp_pcb *pcb, udp_recv_fn recv, void *recv_arg);

void hhg_udp_recv(void *pcb, udp_recv_fn recv, void *recv_arg) {
    udp_recv((struct udp_pcb *)pcb, recv, recv_arg);
}

void * hhg_pbuf_alloc(u16_t length) {
    return pbuf_alloc(PBUF_TRANSPORT, length, PBUF_RAM);
}

u8_t hhg_pbuf_free(void *p) {
    return pbuf_free((struct pbuf *)p);
}

u8_t hhg_netif_is_link_up(void) {
    return netif_is_link_up(netif_default);
}

u8_t hhg_ip_addr_cmp(const ip_addr_t *addr, const ip_addr_t *addr2) {
    return ip_addr_cmp(addr, addr2);
}


    // void dns_found(const char *hostname, const ip_addr_t *ipaddr, void *arg)
    // {
    //     auto state = static_cast<struct ntp*>(arg);
    //     if (ipaddr)
    //     {
    //         state->server_address = *ipaddr;
    //         OSAL_LOG_DEBUG(APP_TAG, "NTP address %s", ipaddr_ntoa(ipaddr));

    //         singleton->ntp.state = ntp::state::DNS_FOUND;

    //         ntp_request(state);
    //     }
    //     else
    //     {
    //         OSAL_LOG_DEBUG(APP_TAG, "NTP dns request failed");
    //         if(state->error && *state->error)
    //         {
    //             *state->error = OSAL_ERROR_APPEND(*state->error, "NTP dns request failed", error_type::OS_EADDRNOTAVAIL);
    //             OSAL_ERROR_PTR_SET_POSITION(*state->error);
    //         }

    //         singleton->ntp.state = ntp::state::NONE;
    //     }
    // }


//typedef void (*dns_found_callback)(const char *name, const ip_addr_t *ipaddr, void *callback_arg);
s8_t hhg_dns_gethostbyname(const char *hostname, ip_addr_t *addr, dns_found_callback found, void *callback_arg) {
    return dns_gethostbyname(hostname, addr, found, callback_arg);
}