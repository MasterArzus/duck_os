//! 地址转换函数

/*
    1. 虚拟地址和物理地址之间的转换
    2. 页号和地址之间的转换
    3. align相关的函数，etc...
    4. 解引用物理地址直接得到值 (mut 和 不可变)（T 和 byte）
*/


use crate::config::mm::{PAGE_SIZE, PAGE_SIZE_BITS, PHY_TO_VIRT_OFFSET, SV39_VPN_1, SV39_VPN_2, SV39_VPN_3};

use super::page_table::PageTableEntry;

pub type VirtAddr = usize;
pub type PhysAddr = usize;

/// 默认了 paddr 在 低32位 仅限内核地址空间
pub fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    paddr | PHY_TO_VIRT_OFFSET
}

/// 物理地址 ---> 物理页号
pub fn phys_to_ppn(paddr: PhysAddr) -> usize {
    paddr >> PAGE_SIZE_BITS
}

/// 物理地址 ---> 下一个物理页号
pub fn phys_to_ppn_next(paddr: PhysAddr) -> usize {
    ( paddr >> PAGE_SIZE_BITS ) + 1
}

/// 虚拟地址转为物理地址 仅限内核地址空间
pub fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
    vaddr - PHY_TO_VIRT_OFFSET
}

/// 虚拟地址转页号 (页首)
pub fn virt_to_vpn(vaddr: VirtAddr) -> usize {
    vaddr >> PAGE_SIZE_BITS
}

/// 虚拟地址转下一页的页号
pub fn virt_to_next_vpn(vaddr: VirtAddr) -> usize {
    (vaddr >> PAGE_SIZE_BITS) + 1
}

/// 虚拟地址所在页的页首地址
pub fn align_down(vaddr: VirtAddr) -> VirtAddr {
    vaddr & !(PAGE_SIZE - 1)
}

/// 虚拟地址所在下一页的页首地址
pub fn align_up(vaddr: VirtAddr) -> VirtAddr {
    (vaddr + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

/// 虚拟页号 --> 虚拟地址
pub fn vpn_to_virt(vpn: usize) -> VirtAddr {
    vpn << PAGE_SIZE_BITS
}

/// 物理页号 ---> 物理地址
pub fn ppn_to_phys(ppn: usize) -> PhysAddr {
    ppn << PAGE_SIZE_BITS
}

/// 地址 -> 页内偏移 offset
pub fn vaddr_offset(vaddr: VirtAddr) -> usize {
    vaddr & (PAGE_SIZE - 1)
}

/// 用户地址空间中 虚拟地址查询物理地址 pte的三级 (vpn2 vpn1 vpn0)
pub fn vaddr_to_pte_vpn(vaddr: VirtAddr) -> (usize, usize, usize) {
    (
        (vaddr >> SV39_VPN_3) & 0x1ff,
        (vaddr >> SV39_VPN_2) & 0x1ff,
        (vaddr >> SV39_VPN_1) & 0x1ff
    )
}

/// 直接从物理地址 解引用获得pte array  4k的页面有512个usize
pub fn pte_array(paddr: PhysAddr) -> &'static mut [PageTableEntry] {
    unsafe {
        core::slice::from_raw_parts_mut(paddr as *mut PageTableEntry, 512)
    }
}

/// 从物理地址 解引用获得byte array
pub fn byte_array(paddr: PhysAddr) -> &'static mut [u8] {
    unsafe {
        core::slice::from_raw_parts_mut(paddr as *mut u8, PAGE_SIZE)
    }
}

/// 从物理地址 解引用获得结构体 T immutable
pub fn get_ref<T>(paddr: PhysAddr) -> &'static T {
    unsafe {
        (paddr as *const T).as_ref().unwrap()
    }
}
/// 从物理地址 解引用获得结构体 T mutable
pub fn get_mut<T>(paddr: PhysAddr) -> &'static mut T {
    unsafe {
        (paddr as *mut T).as_mut().unwrap()
    }
}