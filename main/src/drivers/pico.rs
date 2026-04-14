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

pub(crate) mod ffi;
pub(crate) mod flash;
pub(crate) mod gpio;
pub(crate) mod hardware;
pub(crate) mod i2c;
pub(crate) mod lwip;
pub(crate) mod mbedtls;
pub(crate) mod rtc_ds3231;
pub(crate) mod uart;
pub(crate) mod wifi_cyw43;

use core::ffi::c_char;
use osal_rs::os::types::ThreadHandle;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vApplicationMallocFailedHook() -> ! {
    #[allow(clippy::empty_loop)]
    loop {}
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vApplicationIdleHook() {
    // Idle hook - can be used for low power modes
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vApplicationStackOverflowHook(_x_task: ThreadHandle, _pc_task_name: *mut c_char) -> ! {
    #[allow(clippy::empty_loop)]
    loop {}
}

/// Stack frame pushed by the processor on exception entry
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ExceptionFrame {
    r0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
    r12: u32,
    lr: u32,   // Link Register (return address)
    pc: u32,   // Program Counter (where fault occurred)
    xpsr: u32, // Program Status Register
}

/// System Control Block registers for fault diagnosis
#[repr(C)]
struct FaultRegisters {
    cfsr: u32,  // Configurable Fault Status Register (0xE000ED28)
    hfsr: u32,  // HardFault Status Register (0xE000ED2C)
    dfsr: u32,  // Debug Fault Status Register (0xE000ED30)
    mmfar: u32, // MemManage Fault Address Register (0xE000ED34)
    bfar: u32,  // BusFault Address Register (0xE000ED38)
    afsr: u32,  // Auxiliary Fault Status Register (0xE000ED3C)
}

impl FaultRegisters {
    unsafe fn read() -> Self {
        unsafe {
            Self {
                cfsr: core::ptr::read_volatile(0xE000_ED28 as *const u32),
                hfsr: core::ptr::read_volatile(0xE000_ED2C as *const u32),
                dfsr: core::ptr::read_volatile(0xE000_ED30 as *const u32),
                mmfar: core::ptr::read_volatile(0xE000_ED34 as *const u32),
                bfar: core::ptr::read_volatile(0xE000_ED38 as *const u32),
                afsr: core::ptr::read_volatile(0xE000_ED3C as *const u32),
            }
        }
    }

    #[allow(dead_code)]
    fn analyze(&self) -> &'static str {
        // CFSR is divided into:
        // - MMFSR (bits 0-7): MemManage Fault Status Register
        // - BFSR (bits 8-15): BusFault Status Register
        // - UFSR (bits 16-31): UsageFault Status Register
        
        let mmfsr = (self.cfsr & 0xFF) as u8;
        let bfsr = ((self.cfsr >> 8) & 0xFF) as u8;
        let ufsr = ((self.cfsr >> 16) & 0xFFFF) as u16;

        // Check HardFault causes
        if self.hfsr & (1 << 30) != 0 {
            // FORCED: Configurable fault escalated to HardFault
            if mmfsr != 0 {
                if mmfsr & (1 << 0) != 0 {
                    return "MemManage: Instruction access violation";
                }
                if mmfsr & (1 << 1) != 0 {
                    return "MemManage: Data access violation";
                }
                if mmfsr & (1 << 3) != 0 {
                    return "MemManage: Unstacking error";
                }
                if mmfsr & (1 << 4) != 0 {
                    return "MemManage: Stacking error";
                }
                if mmfsr & (1 << 5) != 0 {
                    return "MemManage: FP lazy state preservation";
                }
                if mmfsr & (1 << 7) != 0 {
                    return "MemManage: MMFAR valid";
                }
            }
            
            if bfsr != 0 {
                if bfsr & (1 << 0) != 0 {
                    return "BusFault: Instruction bus error";
                }
                if bfsr & (1 << 1) != 0 {
                    return "BusFault: Precise data bus error";
                }
                if bfsr & (1 << 2) != 0 {
                    return "BusFault: Imprecise data bus error";
                }
                if bfsr & (1 << 3) != 0 {
                    return "BusFault: Unstacking error";
                }
                if bfsr & (1 << 4) != 0 {
                    return "BusFault: Stacking error";
                }
                if bfsr & (1 << 5) != 0 {
                    return "BusFault: FP lazy state preservation";
                }
                if bfsr & (1 << 7) != 0 {
                    return "BusFault: BFAR valid";
                }
            }
            
            if ufsr != 0 {
                if ufsr & (1 << 0) != 0 {
                    return "UsageFault: Undefined instruction";
                }
                if ufsr & (1 << 1) != 0 {
                    return "UsageFault: Invalid state";
                }
                if ufsr & (1 << 2) != 0 {
                    return "UsageFault: Invalid PC load";
                }
                if ufsr & (1 << 3) != 0 {
                    return "UsageFault: No coprocessor";
                }
                if ufsr & (1 << 4) != 0 {
                    return "UsageFault: Stack overflow (STKOF)";
                }
                if ufsr & (1 << 8) != 0 {
                    return "UsageFault: Unaligned access";
                }
                if ufsr & (1 << 9) != 0 {
                    return "UsageFault: Divide by zero";
                }
            }
        }
        
        if self.hfsr & (1 << 1) != 0 {
            return "HardFault: Vector table read error";
        }
        
        "HardFault: Unknown cause"
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn hardfault_uart_print(s: &[u8]) {
    for &b in s {
        ffi::hhg_uart_putc(b);
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn hardfault_uart_print_hex(val: u32) {
    const HEX: &[u8] = b"0123456789ABCDEF";
    for i in (0..8).rev() {
        let nibble = ((val >> (i * 4)) & 0xF) as usize;
        ffi::hhg_uart_putc(HEX[nibble]);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn isr_hardfault() -> ! {
    // Determine which stack pointer was in use and get the exception frame
    let stack_ptr: *const ExceptionFrame;
    
    unsafe {
        core::arch::asm!(
            "tst lr, #4",           // Test bit 2 of LR (EXC_RETURN)
            "ite eq",               // If-Then-Else
            "mrseq {0}, msp",       // If using MSP, move MSP to output register
            "mrsne {0}, psp",       // If using PSP, move PSP to output register
            out(reg) stack_ptr,
            options(nomem, nostack, preserves_flags)
        );
    }

    // Read the exception frame from stack
    let frame = unsafe { &*stack_ptr };
    
    // Read fault status registers
    let fault_regs = unsafe { FaultRegisters::read() };
    
    // Analyze the fault
    let fault_cause = fault_regs.analyze();
    
    // Print fault info via UART
    unsafe {
        hardfault_uart_print(b"\r\n*** HARD FAULT ***\r\n");
        hardfault_uart_print(b"Cause: ");
        hardfault_uart_print(fault_cause.as_bytes());
        hardfault_uart_print(b"\r\n");
        hardfault_uart_print(b"PC  : 0x"); hardfault_uart_print_hex(frame.pc);    hardfault_uart_print(b"\r\n");
        hardfault_uart_print(b"LR  : 0x"); hardfault_uart_print_hex(frame.lr);    hardfault_uart_print(b"\r\n");
        hardfault_uart_print(b"R0  : 0x"); hardfault_uart_print_hex(frame.r0);    hardfault_uart_print(b"\r\n");
        hardfault_uart_print(b"R1  : 0x"); hardfault_uart_print_hex(frame.r1);    hardfault_uart_print(b"\r\n");
        hardfault_uart_print(b"R2  : 0x"); hardfault_uart_print_hex(frame.r2);    hardfault_uart_print(b"\r\n");
        hardfault_uart_print(b"R3  : 0x"); hardfault_uart_print_hex(frame.r3);    hardfault_uart_print(b"\r\n");
        hardfault_uart_print(b"R12 : 0x"); hardfault_uart_print_hex(frame.r12);   hardfault_uart_print(b"\r\n");
        hardfault_uart_print(b"XPSR: 0x"); hardfault_uart_print_hex(frame.xpsr);  hardfault_uart_print(b"\r\n");
        hardfault_uart_print(b"CFSR: 0x"); hardfault_uart_print_hex(fault_regs.cfsr); hardfault_uart_print(b"\r\n");
        hardfault_uart_print(b"HFSR: 0x"); hardfault_uart_print_hex(fault_regs.hfsr); hardfault_uart_print(b"\r\n");
        hardfault_uart_print(b"DFSR: 0x"); hardfault_uart_print_hex(fault_regs.dfsr); hardfault_uart_print(b"\r\n");
        if fault_regs.cfsr & 0x0080 != 0 {
            hardfault_uart_print(b"MMFAR: 0x"); hardfault_uart_print_hex(fault_regs.mmfar); hardfault_uart_print(b"\r\n");
        }
        if fault_regs.cfsr & 0x8000 != 0 {
            hardfault_uart_print(b"BFAR: 0x"); hardfault_uart_print_hex(fault_regs.bfar); hardfault_uart_print(b"\r\n");
        }
        hardfault_uart_print(b"******************\r\n");
    }

    // Store in static variables for debugger inspection
    static mut LAST_EXCEPTION_FRAME: Option<ExceptionFrame> = None;
    static mut LAST_FAULT_REGISTERS: Option<FaultRegisters> = None;

    unsafe {
        LAST_EXCEPTION_FRAME = Some(*frame);
        LAST_FAULT_REGISTERS = Some(fault_regs);
    }

    // Breakpoint for debugger
    // When debugger stops here, inspect:
    // - frame: r0-r3, r12, lr, pc, xpsr
    // - fault_regs: cfsr, hfsr, dfsr, mmfar, bfar, afsr
    // - fault_cause: human-readable description
    unsafe {
        core::arch::asm!("bkpt #0");
    }

    #[allow(clippy::empty_loop)]
    loop {}
}

