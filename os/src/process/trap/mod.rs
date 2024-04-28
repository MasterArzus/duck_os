use core::arch::global_asm;

use riscv::register::{mtvec, scause::{self, Exception, Trap}, sstatus, stval, stvec};

use crate::syscall::syscall;

use self::context::TrapContext;

pub mod context;

global_asm!(include_str!("trap.S"));

pub fn init_stvec() {
    extern "C" {
        fn __alltraps();
    }
    unsafe {
        // 目前使用大表里面写分支
        stvec::write(__alltraps as usize, mtvec::TrapMode::Direct);
    }
}

#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    match sstatus::read().spp() {
        sstatus::SPP::Supervisor => kernel_trap_handler(cx),
        sstatus::SPP::User => user_trap_handler(cx),
    }
}

#[no_mangle]
pub fn user_trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            let num = syscall(
                cx.x[17],
                [cx.x[10], cx.x[11], cx.x[12], cx.x[13], cx.x[14], cx.x[15]],
            ) as usize;
            cx.set_register(context::Register::a0, num);
        }
        _ => {
            println!(
                "[kernel] encounter page fault, addr {:#x}, instruction {:#x} scause {:?}",
            stval, 0, scause.cause());
            panic!();
        }
    }
    cx
}

#[no_mangle]
pub fn kernel_trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
                       
        }
        _ => {
            panic!();
        }
    }
    cx
}