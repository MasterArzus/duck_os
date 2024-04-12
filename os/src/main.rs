//! This is main mod. It is simple now

// #![deny(missing_docs)]
// #![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(sync_unsafe_cell)]

use core::arch::global_asm;
// use core::arch::asm
use log::*;

#[macro_use]
mod console;
mod lang_items;
mod logging;
mod sbi;
pub mod process;
pub mod mm;
pub mod fs;
pub mod config;
pub mod timer;
pub mod utils;
pub mod driver;
mod syscall;
pub mod sync;

extern crate alloc;
extern crate bitmap_allocator;

#[path = "boards/qemu.rs"]
mod board;

global_asm!(include_str!("entry.S"));

/// clear BSS segment
pub fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

pub fn layout() {
    extern "C" {
        fn stext(); // begin addr of text segment
        fn etext(); // end addr of text segment
        fn srodata(); // start addr of Read-Only data segment
        fn erodata(); // end addr of Read-Only data ssegment
        fn sdata(); // start addr of data segment
        fn edata(); // end addr of data segment
        fn sbss(); // start addr of BSS segment
        fn ebss(); // end addr of BSS segment
        fn boot_stack_lower(); // stack lower bound
        fn boot_stack_top(); // stack top
    }
    trace!(
        "[kernel] .text [{:#x}, {:#x})",
        stext as usize,
        etext as usize
    );
    trace!(
        "[kernel] .rodata [{:#x}, {:#x})",
        srodata as usize, erodata as usize
    );
    trace!(
        "[kernel] .data [{:#x}, {:#x})",
        sdata as usize, edata as usize
    );
    trace!(
        "[kernel] boot_stack top=bottom={:#x}, lower_bound={:#x}",
        boot_stack_top as usize, boot_stack_lower as usize
    );
    trace!("[kernel] .bss [{:#x}, {:#x})", sbss as usize, ebss as usize);
}

/// the rust entry-point of os
#[no_mangle]
pub fn rust_main() {
    
    clear_bss();
    logging::init();
    println!("[kernel] Hello, world!");
    layout();

    // // Warning: 这里我们自己自动的让qemu终止!
    // use crate::board::QEMUExit;
    // crate::board::QEMU_EXIT_HANDLE.exit_success(); // CI autotest success
}
