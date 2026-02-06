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

#include <pico/cyw43_arch.h>


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
