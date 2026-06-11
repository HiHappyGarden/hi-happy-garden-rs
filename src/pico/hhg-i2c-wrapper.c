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



#include <pico/types.h>
#include <pico/error.h>
#include <pico/binary_info.h>
#include <pico/platform.h>
#include <hardware/i2c.h>
#include <hardware/dma.h>
#include <hardware/gpio.h>
#include <hardware/regs/i2c.h>

extern void * pvPortMalloc(size_t xWantedSize);
extern void vPortFree(void *pv);


void* hhg_i2c_instance(uint8_t i2c_num) {
    if (i2c_num > 1) {
        return NULL;
    }
    return I2C_INSTANCE(i2c_num);
}

uint hhg_i2c_init(void *i2c, uint baudrate) {
    if (i2c == NULL) {
        return 1;
    }
    return i2c_init((i2c_inst_t *)i2c, baudrate);
}

void hhg_i2c0_init_pins_with_func(void) {
    bi_decl(bi_2pins_with_func(16, 17, GPIO_FUNC_I2C));
}

void hhg_i2c1_init_pins_with_func(void) {
    bi_decl(bi_2pins_with_func(2, 3, GPIO_FUNC_I2C));
}

int hhg_i2c_write_blocking(void *i2c, uint8_t addr, const uint8_t *src, size_t len, bool nostop) {
    if (i2c == NULL) {
        return PICO_ERROR_INVALID_ARG;
    }
    //return i2c_write_timeout_per_char_us((i2c_inst_t *)i2c, addr, src, len, nostop, 1000);
    return i2c_write_blocking((i2c_inst_t *)i2c, addr, src, len, nostop);
}

int hhg_i2c_write_blocking_dma(void *i2c, uint8_t addr, const uint8_t *src, size_t len, bool nostop) {
    if (i2c == NULL || src == NULL || len == 0) {
        return PICO_ERROR_INVALID_ARG;
    }

    i2c_inst_t *instance = (i2c_inst_t *)i2c;

    // DMA path is intentionally limited to I2C1; other instances keep the normal blocking write.
    if (instance != i2c1) {
        return i2c_write_blocking(instance, addr, src, len, nostop);
    }

    uint16_t *tx_cmd = (uint16_t *)pvPortMalloc(sizeof(uint16_t) * len);
    if (tx_cmd == NULL) {
        return i2c_write_blocking(instance, addr, src, len, nostop);
    }

    instance->hw->enable = 0;
    instance->hw->tar = addr;
    instance->hw->enable = 1;

    for (size_t i = 0; i < len; i++) {
        uint16_t cmd = src[i];
        if (i == 0 && instance->restart_on_next) {
            cmd |= I2C_IC_DATA_CMD_RESTART_BITS;
        }
        if (i == (len - 1) && !nostop) {
            cmd |= I2C_IC_DATA_CMD_STOP_BITS;
        }
        tx_cmd[i] = cmd;
    }

    int dma_ch = dma_claim_unused_channel(false);
    if (dma_ch < 0) {
        vPortFree(tx_cmd);
        return i2c_write_blocking(instance, addr, src, len, nostop);
    }

    dma_channel_config cfg = dma_channel_get_default_config((uint)dma_ch);
    channel_config_set_transfer_data_size(&cfg, DMA_SIZE_16);
    channel_config_set_read_increment(&cfg, true);
    channel_config_set_write_increment(&cfg, false);
    channel_config_set_dreq(&cfg, i2c_get_dreq(instance, true));

    dma_channel_configure(
        (uint)dma_ch,
        &cfg,
        &instance->hw->data_cmd,
        tx_cmd,
        len,
        true
    );

    dma_channel_wait_for_finish_blocking((uint)dma_ch);
    dma_channel_unclaim((uint)dma_ch);

    uint32_t abort_reason = instance->hw->tx_abrt_source;
    if (abort_reason) {
        instance->hw->clr_tx_abrt;
        instance->restart_on_next = false;
        vPortFree(tx_cmd);

        if (abort_reason & I2C_IC_TX_ABRT_SOURCE_ABRT_7B_ADDR_NOACK_BITS) {
            return PICO_ERROR_GENERIC;
        }
        if (abort_reason & I2C_IC_TX_ABRT_SOURCE_ABRT_TXDATA_NOACK_BITS) {
            return PICO_ERROR_GENERIC;
        }
        return PICO_ERROR_GENERIC;
    }

    if (!nostop) {
        while (!(instance->hw->raw_intr_stat & I2C_IC_RAW_INTR_STAT_STOP_DET_BITS)) {
            tight_loop_contents();
        }
        instance->hw->clr_stop_det;
    }

    instance->restart_on_next = nostop;
    vPortFree(tx_cmd);

    return (int)len;
}

int hhg_i2c_read_blocking(void *i2c, uint8_t addr, uint8_t *dst, size_t len, bool nostop) {
    if (i2c == NULL) {
        return PICO_ERROR_INVALID_ARG;
    }
    //return i2c_read_timeout_per_char_us((i2c_inst_t *)i2c, addr, dst, len, nostop, 1000);
    return i2c_read_blocking((i2c_inst_t *)i2c, addr, dst, len, nostop);
}

void hhg_i2c_deinit(void *i2c) {
    if (i2c == NULL) {
        return;
    }
    i2c_deinit((i2c_inst_t *)i2c);
}
