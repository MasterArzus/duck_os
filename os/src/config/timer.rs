//! Configuration in timer


pub const TICKS_PER_SEC: usize = 100;
pub const MSEC_PER_SEC: usize = 1_000;
pub const USEC_PER_SEC: usize = 1_000_000;
pub const NSEC_PER_SEC: usize = 1_000_000_000;

// QEMU的时钟频率
// TODO：不确定到底应该为多少？
pub const CLOCK_FREQUENCY: usize = 400_000_000; 