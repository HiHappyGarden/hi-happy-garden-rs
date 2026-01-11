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


/**
 * @file hhg-gpio-wrapper.c
 * @brief C wrapper functions for Pico SDK GPIO/PWM to ensure proper ARM/Thumb linkage
 */

#include <pico/types.h>
#include <hardware/gpio.h>
#include <hardware/pwm.h>
#include <hardware/irq.h>
#include "hardware/adc.h"

// Wrapper functions with explicit ARM/Thumb compatibility
void hhg_gpio_init(uint gpio) {
    gpio_init(gpio);
}

void hhg_gpio_set_dir(uint gpio, bool out) {
    gpio_set_dir(gpio, out);
}

void hhg_gpio_put(uint gpio, bool value) {
    gpio_put(gpio, value);
}

bool hhg_gpio_get(uint gpio) {
    return gpio_get(gpio);
}


void hhg_gpio_pull_up(uint gpio) {
    gpio_pull_up(gpio);
}

void hhg_gpio_pull_down(uint gpio) {
    gpio_pull_down(gpio);
}

void hhg_gpio_disable_pulls(uint gpio) {
    gpio_disable_pulls(gpio);
}

void hhg_gpio_set_function(uint gpio, uint32_t fn) {
    // gpio_function values are simple integers 0-31
    gpio_set_function(gpio, fn);
}

uint hhg_pwm_gpio_to_slice_num(uint gpio) {
    return pwm_gpio_to_slice_num(gpio);
}

pwm_config hhg_pwm_get_default_config(void) {
    return pwm_get_default_config();
}

void hhg_pwm_config_set_clkdiv(pwm_config *c, float div) {
    pwm_config_set_clkdiv(c, div);
}

void hhg_pwm_init(uint slice_num, pwm_config *c, bool start) {
    pwm_init(slice_num, c, start);
}


void hhg_pwm_set_gpio_level(uint gpio, uint16_t level) {
    pwm_set_gpio_level(gpio, level);
}

// ISR dispatcher for multiple GPIO pins
typedef void (*simple_gpio_callback_t)(void);

#define MAX_GPIO_CALLBACKS 32
static simple_gpio_callback_t gpio_callbacks[MAX_GPIO_CALLBACKS] = {0};

static void gpio_dispatcher_isr(uint gpio, uint32_t event_mask) {
    if (gpio < MAX_GPIO_CALLBACKS && gpio_callbacks[gpio] != NULL) {
        gpio_callbacks[gpio]();
    }
}

void hhg_gpio_set_irq_enabled_with_callback(uint gpio, uint32_t events, bool enabled, gpio_irq_callback_t callback) {
    // Store the callback in our dispatcher table
    if (gpio < MAX_GPIO_CALLBACKS) {
        gpio_callbacks[gpio] = (simple_gpio_callback_t)callback;
    }
    
    // Register our dispatcher as the actual interrupt handler (only once)
    static bool dispatcher_registered = false;
    if (!dispatcher_registered) {
        gpio_set_irq_callback(gpio_dispatcher_isr);
        irq_set_enabled(IO_IRQ_BANK0, true);
        dispatcher_registered = true;
    }
    
    // Enable/disable the interrupt for this specific GPIO
    gpio_set_irq_enabled(gpio, events, enabled);
}

void hhd_irq_set_enabled(uint num, bool enabled) {
    irq_set_enabled(num, enabled);
}

void hhg_gpio_set_irq_enabled(uint gpio, uint32_t events, bool enabled) {
    adc_init();
    adc_set_temp_sensor_enabled(true);
    adc_select_input(4);

    gpio_set_irq_enabled(gpio, events, enabled);
}

void hhg_adc_init() {
    adc_init();
}

void hhg_adc_set_temp_sensor_enabled(bool enable) {
    adc_set_temp_sensor_enabled(enable);
}

void hhg_adc_select_input(uint input) {
    adc_select_input(input);
}

uint16_t hhg_adc_read() {
    return adc_read();
}
