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

u16_t hhg_pbuf_copy_partial(struct pbuf *buf, void *dataptr, u16_t len, u16_t offset) {
    return pbuf_copy_partial(buf, dataptr, len, offset);
}

err_t hhg_pbuf_alloc(u16_t length) {
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

err_t hhg_dns_gethostbyname(const char *hostname, ip_addr_t *addr, dns_found_callback found, void *callback_arg) {
    return dns_gethostbyname(hostname, addr, found, callback_arg);
}

err_t hhg_udp_sendto(void *buf, struct pbuf *p, const ip_addr_t *ipaddr, u16_t port) {
    return udp_sendto((struct pbuf *)buf, p, ipaddr, port);
}

//-----------------test functions-----------------

static ip_addr_t static_ip_addr = {0};
static void dns_found(const char *hostname, const ip_addr_t *ipaddr, void *arg)
{
    void (*dns_found)(bool found, const ip_addr_t *addr) = (void (*)(bool, const ip_addr_t *))arg;
    if (ipaddr)
    {
        dns_found(true, ipaddr);
    }
    else
    {
        dns_found(false, NULL);
        free(arg);
    }
    
}

void hhg_udp_recv(void *pcb, udp_recv_fn recv, void *recv_arg) {
    udp_recv((struct udp_pcb *)pcb, recv, recv_arg);
}

static struct udp_pcb *pcb = NULL;
s8_t hhg_ntp_request(const ip_addr_t *ipaddr_dest, s16_t port, s16_t msg_size) {
    // cyw43_arch_lwip_begin/end should be used around calls into lwIP to ensure correct locking.
    // You can omit them if you are in a callback from lwIP. Note that when using pico_cyw_arch_poll
    // these calls are a no-op and can be omitted, but it is a good practice to use them in
    // case you switch the cyw43_arch type later.
    
    cyw43_arch_lwip_begin();
    struct pbuf *p = pbuf_alloc(PBUF_TRANSPORT, msg_size, PBUF_RAM);
    if(!p)
    {
        cyw43_arch_lwip_end();
        return -1;
    }

    uint8_t* req = (uint8_t*)p->payload;
    memset(req, 0, msg_size);
    req[0] = 0x1b;

    pcb = udp_new_ip_type(IPADDR_TYPE_V4);
    udp_sendto(pcb, p, ipaddr_dest, port);

    pbuf_free(p);
    cyw43_arch_lwip_end();

    return 0;
}

