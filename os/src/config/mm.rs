/*!
    页大小
    页对应的字节
    内核地址的offset
    堆的大小
*/

/// 内核中堆的大小
pub const KERNEL_HEAP_SIZE: usize = 0xc0_0000; // 12MB: 1 represent 1 bit

/// 内核虚拟地址与物理地址的offset
pub const KERNEL_DIRECT_OFFSET: usize = 0xffff_ffc0_0000_0;

/// 页的大小bit
pub const PAGE_SIZE_BITS: usize = 0xc;