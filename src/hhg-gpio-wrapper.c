/**
 * @file hhg-gpio-wrapper.c
 * @brief C wrapper functions for Pico SDK GPIO/PWM to ensure proper ARM/Thumb linkage
 */

#include "pico/types.h"
#include "hardware/gpio.h"
#include "hardware/pwm.h"

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

