/*!
    页大小
    页对应的字节
    内核地址的offset
    堆的大小
*/

/// 内核中堆的大小
pub const KERNEL_HEAP_SIZE: usize = 0xc0_0000; // 12MB: 1 represent 1 bit

/// 内核虚拟地址与物理地址的offset
pub const PHY_TO_VIRT_OFFSET: usize = 0xffff_ffc0_0000_0000;

/// 内核虚拟页号与物理页号的offset
pub const PHY_TO_VIRT_PPN_OFFSET: usize = 0xffff_ffc0_0000_0;

/// 页的大小bit
pub const PAGE_SIZE_BITS: usize = 0xc;

/// 页的大小 4kb
pub const PAGE_SIZE: usize = 0x1000;

/// vpn不同的索引对应的不同的位数
pub const SV39_VPN_1: usize = 12;
/// vpn不同的索引对应的不同的位数
pub const SV39_VPN_2: usize = 21;
/// vpn不同的索引对应的不同的位数
pub const SV39_VPN_3: usize = 30;

/// physical frame memory 终点位置
pub const MEMORY_END: usize = 0x8800_0000 + PHY_TO_VIRT_OFFSET;

/// LOW_LIMIT mmap函数中使用的
pub const LOW_LIMIT: usize = 0x1000;

/// UPPER_LIMIT mmap函数中使用的
pub const UPPER_LIMIT: usize = 0xffff_ffff_ffff_ffff;