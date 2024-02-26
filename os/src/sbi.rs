//! 目前的SBI实现较为简陋 2.26
//！可供参考的实现方法：
//！1.手写 sbi_call, 根据手册 file:///home/user/Downloads/riscv-sbi.pdf
//！2.基于已经有的库 例如 sbi-rt，opensbi, rustsbi

use core::arch::asm;

// (EID, FID)
const SBI_CONSOLE_PUTCHAR: (usize, usize) = (1, 0);

/// general sbi call
#[inline(always)]
fn sbi_call(eid_fid:(usize, usize), arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") arg0 => ret,
            in("x11") arg1,
            in("x12") arg2,
            in("x16") eid_fid.1,
            in("x17") eid_fid.0,
        );
    }
    ret
}

/// use sbi call to putchar in console (qemu uart handler)
pub fn console_putchar(c: usize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, c, 0, 0);
}

// TODO： 这里暂时使用qemu中的exit.其实可以使用sbi_call()来终止。
use crate::board::QEMUExit;
pub fn shutdown() -> ! {
    crate::board::QEMU_EXIT_HANDLE.exit_failure();
}
