


#![allow(non_camel_case_types)]
#![allow(dead_code)]

use core::{ffi::{c_char, c_int, c_long, c_uint, c_void}, ptr::null_mut};
use core::ffi::{c_uchar, c_ushort};

#[repr(C)]
pub struct pwm_config {
    pub csr: u32,
    pub div: u32,
    pub top: u32,
}

pub(super) const GPIO_OUT: bool = true;
pub(super) const GPIO_IN: bool = false;  

pub(super) mod gpio_function_type {
    pub const GPIO_FUNC_HSTX: u32 = 0;
    pub const GPIO_FUNC_SPI: u32 = 1;
    pub const GPIO_FUNC_UART: u32 = 2;
    pub const GPIO_FUNC_I2C: u32 = 3;
    pub const GPIO_FUNC_PWM: u32 = 4;
    pub const GPIO_FUNC_SIO: u32 = 5;
    pub const GPIO_FUNC_PIO0: u32 = 6;
    pub const GPIO_FUNC_PIO1: u32 = 7;
    pub const GPIO_FUNC_PIO2: u32 = 8;
    pub const GPIO_FUNC_GPCK: u32 = 9;
    pub const GPIO_FUNC_XIP_CS1: u32 = 9;
    pub const GPIO_FUNC_CORESIGHT_TRACE: u32 = 9;
    pub const GPIO_FUNC_USB: u32 = 10;
    pub const GPIO_FUNC_UART_AUX: u32 = 11;
    pub const GPIO_FUNC_NULL: u32 = 0x1f;
}

#[repr(C)]
pub enum pico_error_codes {
    PICO_OK = 0,                                ///< No error; the operation succeeded
    PICO_ERROR_GENERIC = -1,                    ///< An unspecified error occurred
    PICO_ERROR_TIMEOUT = -2,                    ///< The function failed due to timeout
    PICO_ERROR_NO_DATA = -3,                    ///< Attempt for example to read from an empty buffer/FIFO
    PICO_ERROR_NOT_PERMITTED = -4,              ///< Permission violation e.g. write to read-only flash partition, or security violation
    PICO_ERROR_INVALID_ARG = -5,                ///< Argument is outside of range of supported values`
    PICO_ERROR_IO = -6,                         ///< An I/O error occurred
    PICO_ERROR_BADAUTH = -7,                    ///< The authorization failed due to bad credentials
    PICO_ERROR_CONNECT_FAILED = -8,             ///< The connection failed
    PICO_ERROR_INSUFFICIENT_RESOURCES = -9,     ///< Dynamic allocation of resources failed
    PICO_ERROR_INVALID_ADDRESS = -10,           ///< Address argument was out-of-bounds or was determined to be an address that the caller may not access
    PICO_ERROR_BAD_ALIGNMENT = -11,             ///< Address was mis-aligned (usually not on word boundary)
    PICO_ERROR_INVALID_STATE = -12,             ///< Something happened or failed to happen in the past, and consequently we (currently) can't service the request
    PICO_ERROR_BUFFER_TOO_SMALL = -13,          ///< A user-allocated buffer was too small to hold the result or working state of this function
    PICO_ERROR_PRECONDITION_NOT_MET = -14,      ///< The call failed because another function must be called first
    PICO_ERROR_MODIFIED_DATA = -15,             ///< Cached data was determined to be inconsistent with the actual version of the data
    PICO_ERROR_INVALID_DATA = -16,              ///< A data structure failed to validate
    PICO_ERROR_NOT_FOUND = -17,                 ///< Attempted to access something that does not exist; or, a search failed
    PICO_ERROR_UNSUPPORTED_MODIFICATION = -18,  ///< Write is impossible based on previous writes; e.g. attempted to clear an OTP bit
    PICO_ERROR_LOCK_REQUIRED = -19,             ///< A required lock is not owned
    PICO_ERROR_VERSION_MISMATCH = -20,          ///< A version mismatch occurred (e.g. trying to run PIO version 1 code on RP2040)
    PICO_ERROR_RESOURCE_IN_USE = -21,           
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub(super) enum gpio_irq_level {
    GPIO_IRQ_LEVEL_LOW = 0x1,  
    GPIO_IRQ_LEVEL_HIGH = 0x2, 
    GPIO_IRQ_EDGE_FALL = 0x4,  
    GPIO_IRQ_EDGE_RISE = 0x8  
}


#[repr(C)]
#[derive(Clone, Copy)]
pub enum uart_parity {
    UART_PARITY_NONE = 0,
    UART_PARITY_EVEN = 1,
    UART_PARITY_ODD = 2,
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum aes_mode {
    AES_ENCRYPT = 1,
    AES_DECRYPT = 0,
}
pub struct MbedtlsAes (pub *mut c_void);

unsafe impl Send for MbedtlsAes {}
unsafe impl Sync for MbedtlsAes {}


pub type irq_handler_t = unsafe extern "C" fn();

// Type aliases for littlefs types
pub type LfsSize = u32;
pub type LfsSsize = i32;
pub type LfsSoff = i32;
pub type LfsOff = u32;

pub mod cyw43_auth {
    ///< No authorisation required (open)
    pub const OPEN: u32 = 0;

    ///< WPA authorisation
    pub const WPA_TKIP_PSK: u32 = 0x00200002;

    ///< WPA2 authorisation (preferred)
    pub const WPA2_AES_PSK: u32 = 0x00400004;

    ///< WPA2/WPA mixed authorisation
    pub const WPA2_MIXED_PSK: u32 = 0x00400006;

    ///< WPA3 AES authorisation
    pub const WPA3_SAE_AES_PSK: u32 = 0x01000004;

    ///< WPA2/WPA3 authorisation
    pub const WPA3_WPA2_AES_PSK: u32 = 0x01400004;
}

#[repr(i32)]
pub enum CYW43Itf {
    ///< Client interface STA mode
    STA = 0,
    ///< Access point (AP) interface mode
    AP = 1,
}

#[repr(C)]
pub struct udp_pcb {
    _private: [u8; 0],
}

#[repr(C)]
pub struct pbuf {
    _private: [u8; 0],
}

#[repr(C)]
pub struct ip4_addr {
    pub addr: u32
}

pub type udp_recv_fn = extern "C" fn(arg: *mut c_void, pcb: *mut udp_pcb, pbuf: *mut pbuf, addr: *const ip4_addr, port: c_ushort);

unsafe extern "C" {
    pub(super) fn hhg_gpio_init(gpio: u32);
    pub(super) fn hhg_gpio_set_dir(gpio: u32, out: bool);
    pub(super) fn hhg_gpio_put(gpio: u32, value: bool);
    pub(super) fn hhg_gpio_get(gpio: u32) -> bool;
    pub(super) fn hhg_gpio_pull_up(gpio: u32);
    pub(super) fn hhg_gpio_pull_down(gpio: u32);
    pub(super) fn hhg_gpio_disable_pulls(gpio: u32);
    pub(super) fn hhg_gpio_set_function(gpio: u32, fn_: u32);
    pub(super) fn hhg_pwm_gpio_to_slice_num(gpio: u32) -> u32;
    pub(super) fn hhg_pwm_get_default_config() -> pwm_config;
    pub(super) fn hhg_pwm_config_set_clkdiv(c: *mut pwm_config, div: f32);
    pub(super) fn hhg_pwm_config_set_wrap(c: *mut pwm_config, wrap: u16);
    pub(super) fn hhg_pwm_init(slice_num: u32, c: *mut pwm_config, start: bool);
    pub(super) fn hhg_pwm_set_gpio_level(gpio: u32, level: u16);
    pub(super) fn hhg_gpio_set_irq_enabled_with_callback(gpio: u32, events: u32, enabled: bool, callback: extern "C" fn());
    pub(super) fn hhg_gpio_set_irq_enabled(gpio: u32, events: u32, enabled: bool);

    pub(super) fn hhg_uart_init(baudrate: c_uint) -> c_uint;
    pub(super) fn hhg_uart_deinit();
    pub(super) fn hhg_uart_set_hw_flow(cts: bool, rts: bool);
    pub(super) fn hhg_uart_set_format(data_bits: c_uint, stop_bits: c_uint, parity: uart_parity);
    pub(super) fn hhg_uart_irq_set_exclusive_handler(handler: irq_handler_t);
    pub(super) fn hhg_uart_irq_set_enabled(enabled: bool);
    pub(super) fn hhg_uart_set_irq_enables(rx_en: bool, tx_en: bool);
    pub(super) fn hhg_uart_is_readable() -> bool;
    pub(super) fn hhg_uart_getc() -> u8;
    pub(super) fn hhg_uart_putc(c: u8);

    pub(super) fn hhg_adc_init();
    pub(super) fn hhg_adc_set_temp_sensor_enabled(enable: bool);
    pub(super) fn hhg_adc_select_input(input: c_uint);
    pub(super) fn hhg_adc_read() -> u16;


    pub(super) fn hhg_cyw43_arch_gpio_put(wl_gpio: u32, value: bool);
    pub(super) fn hhg_cyw43_arch_init() -> c_int;
    pub(super) fn hhg_cyw43_arch_deinit();
    pub(super) fn  hhg_cyw43_arch_enable_sta_mode();
    pub(super) fn  hhg_cyw43_arch_disable_sta_mode();
    pub(super) fn hhg_cyw43_wifi_link_status(itf: c_uint) -> c_int;
    pub(super) fn hhg_cyw43_arch_wifi_connect_async(ssid: *const c_char, pw: *const c_char, auth: c_uint) -> c_int;
    pub(super) fn hhg_cyw43_arch_lwip_begin();
    pub(super) fn hhg_cyw43_arch_lwip_end();


    pub(super) fn hhg_dhcp_get_ip_address() -> *const c_char;
    pub(super) fn hhg_dhcp_get_binary_ip_address() -> c_uint;
    pub(super) fn  hhg_dhcp_supplied_address() -> bool;
    pub(super) fn hhg_udp_new_ip_type(_type: c_uchar) -> *mut c_void;
    pub(super) fn hhg_udp_recv(pcb: *mut c_void, recv: udp_recv_fn, recv_arg: *mut c_void);
    pub(super) fn hhg_pbuf_alloc(length: c_ushort) -> *mut c_void;
    pub(super) fn hhg_pbuf_free(p: *mut c_void) -> c_uchar;
    pub(super) fn hhg_netif_is_link_up() -> c_uchar;


    pub(super) fn hhg_i2c_instance(i2c_num: u8) -> *mut c_void;
    pub(super) fn hhg_i2c_init(i2c: *mut c_void, baudrate: c_uint) -> c_uint;
    pub(super) fn hhg_i2c_init_pins_with_func();
    pub(super) fn hhg_i2c_write_blocking(i2c: *mut c_void, addr: u8, src: *const u8, len: usize, nostop: bool) -> i32;
    pub(super) fn hhg_i2c_read_blocking(i2c: *mut c_void, addr: u8, dst: *mut u8, len: usize, nostop: bool) -> i32;

    pub(super) fn hhg_flash_mount(format: bool) -> c_int;
    pub(super) fn hhg_flash_open(path: *const c_char, flags: c_int) -> *mut c_void;
    pub(super) fn hhg_flash_close(file: *mut c_void) -> c_int;
    pub(super) fn hhg_flash_write(file: *mut c_void, buffer: *const c_void, size: LfsSize) -> LfsSize;
    pub(super) fn hhg_flash_read(file: *mut c_void, buffer: *mut c_void, size: LfsSize) -> LfsSize;
    pub(super) fn hhg_flash_rewind(file: *mut c_void) -> c_int;
    pub(super) fn hhg_flash_umount() -> c_int;
    pub(super) fn hhg_flash_remove(path: *const c_char) -> c_int;
    pub(super) fn hhg_flash_rename(oldpath: *const c_char, newpath: *const c_char) -> c_int;
    pub(super) fn hhg_flash_fsstat(
        block_size: *mut LfsSize,
        block_count: *mut LfsSize,
        blocks_used: *mut LfsSize,
    ) -> c_int;
    pub(super) fn hhg_flash_lseek(file: *mut c_void, off: LfsSoff, whence: c_int) -> LfsSoff;
    pub(super) fn hhg_flash_truncate(file: *mut c_void, size: LfsOff) -> c_int;
    pub(super) fn hhg_flash_tell(file: *mut c_void) -> LfsSoff;
    pub(super) fn hhg_flash_stat(
        path: *const c_char,
        type_: *mut u8,
        size: *mut LfsSize,
        name: *mut c_char,
    ) -> c_int;
    pub(super) fn hhg_flash_getattr(
        path: *const c_char,
        type_: u8,
        buffer: *mut c_void,
        size: LfsSize,
    ) -> LfsSsize;
    pub(super) fn hhg_flash_setattr(
        path: *const c_char,
        type_: u8,
        buffer: *const c_void,
        size: LfsSize,
    ) -> c_int;
    pub(super) fn hhg_flash_removeattr(path: *const c_char, type_: u8) -> c_int;
    pub(super) fn hhg_flash_fflush(file: *mut c_void) -> c_int;
    pub(super) fn hhg_flash_size(file: *mut c_void) -> LfsSoff;
    pub(super) fn hhg_flash_mkdir(path: *const c_char) -> c_int;
    pub(super) fn hhg_flash_dir_open(path: *const c_char) -> *mut c_void;
    pub(super) fn hhg_flash_dir_close(dir: *mut c_void) -> c_int;
    pub(super) fn hhg_flash_dir_read(
        dir: *mut c_void,
        type_: *mut u8,
        size: *mut LfsSize,
        name: *mut c_char,
    ) -> c_int;
    pub(super) fn hhg_flash_dir_seek(dir: *mut c_void, off: LfsOff) -> c_int;
    pub(super) fn hhg_flash_dir_tell(dir: *mut c_void) -> LfsSoff;
    pub(super) fn hhg_flash_dir_rewind(dir: *mut c_void) -> c_int;
    pub(super) fn hhg_flash_errmsg(err: c_int) -> *const c_char;

    pub(super) fn hhg_get_unique_id(id_buffer: *mut u8);

    pub(super) fn hhg_mbedtls_aes_init() -> *mut c_void;
    pub(super) fn hhg_mbedtls_aes_setkey_enc(aes: *mut c_void, key: *const u8, keybits: u32) -> i32;
    pub(super) fn hhg_mbedtls_aes_crypt_cbc(aes: *mut c_void, mode: i32, length: usize, iv: *mut u8, input: *const u8, output: *mut u8) -> i32;
    pub(super) fn hhg_mbedtls_aes_setkey_dec(aes: *mut c_void, key: *const u8, keybits: u32) -> i32;
    pub(super) fn hhg_mbedtls_aes_free(aes: *mut c_void);
    
}   
