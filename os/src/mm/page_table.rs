//！ sv39 三级页表

/*
    1. 页表项 pte 中的各个位有关函数
    2. page_table 有 root_paddr 和 Frames
    3. 页表的函数
        1） activate 切换页表
        2） 找页表项 find_pte 和 find_pte_create
        3） map 和 unmap 可以有单次和多次映射之分
        4） 可以将虚拟地址翻译为物理地址
        5） 页表的token
*/

use core::arch::asm;
use alloc::vec::Vec;
use alloc::vec;
use riscv::register::satp::{self};

use crate::config::mm::KERNEL_PTE_POS;

use super::{
    address::{phys_to_ppn, ppn_to_phys, pte_array, vaddr_offset, vaddr_to_pte_vpn, virt_to_vpn, vpn_to_virt, PhysAddr, VirtAddr}, 
    allocator::frame::{alloc_frame, FrameTracker}, 
    memory_set::mem_set::KERNEL_SPACE, 
    type_cast::PTEFlags,
};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct PageTableEntry {
    pub pte: usize,
}

/// pte中不同位的操作 设置位、判断位、初始化
impl PageTableEntry {
    ///
    pub fn new(ppn: usize, flags: PTEFlags) -> Self {
        PageTableEntry {
            pte: (ppn << 10) | (flags.bits() as usize)
        }
    }

    /// 
    pub fn empty() -> Self {
        PageTableEntry { pte: 0 }
    }

    ///
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits((self.pte & ((1 << 9) - 1)) as u16).unwrap()
    }

    ///
    pub fn ppn(&self) -> usize {
        self.pte >> 10 & ((1usize << 44) - 1)
    }

    ///
    pub fn is_valid(&self) -> bool {
        self.flags().contains(PTEFlags::V)
    }

    ///
    pub fn is_writable(&self) -> bool {
        self.flags().contains(PTEFlags::W)
    }

    ///
    pub fn is_readable(&self) -> bool {
        self.flags().contains(PTEFlags::R)
    }

    ///
    pub fn is_executable(&self) -> bool {
        self.flags().contains(PTEFlags::X)
    }

    ///
    pub fn is_user(&self) -> bool {
        self.flags().contains(PTEFlags::U)
    }

    ///
    pub fn set_flags(&mut self, flags: PTEFlags) {
        let new_flags = flags.bits() as usize;
        self.pte = (self.ppn() << 10 ) | new_flags;
    }
}

pub struct PageTable {
    root_paddr: usize,
    frames: Vec<FrameTracker>
}

impl PageTable {
    /// 页表的初始化
    pub fn new() -> Self {
        let frame = alloc_frame().unwrap();
        PageTable {
            root_paddr: ppn_to_phys(frame.ppn),
            frames: vec![frame]
        }
    }

    /// 需要一个全局变量KERNEL 然后要做映射
    /// TODO: 是否可以直接访问物理地址拿到想要的数据？？
    /// TODO：同时是否是在第 258项？？
    pub fn new_user() -> Self {
        let frame = alloc_frame().unwrap();
        let kernel_root_paddr = unsafe {
            KERNEL_SPACE
                .as_ref()
                .unwrap()
                .pt
                .get()
                .as_ref()
                .unwrap()
                .root_paddr()
        };
        let user_pte_array = pte_array(ppn_to_phys(frame.ppn));
        let kernel_pte_array = pte_array(kernel_root_paddr);
        (*user_pte_array)[KERNEL_PTE_POS] = (*kernel_pte_array)[KERNEL_PTE_POS];
        PageTable { root_paddr: ppn_to_phys(frame.ppn), frames: vec![frame] }
    }

    pub fn root_ppn(&self) -> usize {
        phys_to_ppn(self.root_paddr)
    }

    pub fn root_paddr(&self) -> usize {
        self.root_paddr
    }

    /// page table token
    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn()
    }

    /// activate page_table
    pub fn activate(&self) {
        // TODO: 不知道有没有逻辑问题？
        let old_satp = satp::read().ppn();
        if old_satp != self.root_ppn() {
            let satp = self.token();
            unsafe {
                satp::write(satp);
                asm!("sfence.vma");
            }
        }
    }

    /// 找页表项 如果没有则创建一个 返回物理页号
    pub fn find_pte_create(&mut self, vaddr: VirtAddr) -> &mut PageTableEntry {
        let idx: (usize, usize, usize) = vaddr_to_pte_vpn(vaddr);
        let idx_array: [usize; 3] = [idx.0, idx.1, idx.2];
        let mut pa = self.root_paddr;
        for i in 0..3 {
            let pte = &mut pte_array(pa)[idx_array[i]];
            if i == 2 {
                return pte;
            }
            if !pte.is_valid() {
                let frame = alloc_frame().unwrap();
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);        
            }
            pa = ppn_to_phys(pte.ppn());
        }
        unreachable!();
    }
    // 找页表项 如果没有，则返回None
    pub fn find_pte(&self, vaddr: VirtAddr) -> Option<&mut PageTableEntry> {
        let idx: (usize, usize, usize) = vaddr_to_pte_vpn(vaddr);
        let idx_array: [usize; 3] = [idx.0, idx.1, idx.2];
        let mut pa = self.root_paddr;
        for i in 0..3 {
            let pte = &mut pte_array(pa)[idx_array[i]];
            if !pte.is_valid() {
                return None;      
            }
            if i == 2 {
                return Some(pte);
            }
            
            pa = ppn_to_phys(pte.ppn());
        }
        unreachable!();
    }

    /// 映射一次虚拟页号和物理页号 同时要有flags
    pub fn map_one(&mut self, vpn: usize, ppn: usize, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn_to_virt(vpn));
        if pte.is_valid() {
            panic!("The corresponding pte is not valid.");
        }
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::A | PTEFlags::D | PTEFlags::V);
    }

    /// 解除映射
    pub fn unmap(&self, vpn: usize) {
        let pte = self.find_pte(vpn_to_virt(vpn)).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmap", vpn);
        *pte = PageTableEntry::empty();
    }

    /// 由vpn查找pte
    pub fn translate_vpn_to_pte(&self, vpn: usize) -> Option<PageTableEntry> {
        if let Some(pte) = self.find_pte(vpn_to_virt(vpn)) {
            Some(*pte)
        } else {
            None
        }
    }

    /// 由va查找pa
    pub fn translate_va_to_pa(&self, vaddr: VirtAddr) -> Option<PhysAddr> {
        let vpn = virt_to_vpn(vaddr);
        let offset = vaddr_offset(vaddr);
        if let Some(pte) = self.translate_vpn_to_pte(vpn) {
            assert!(pte.is_valid());
            Some(ppn_to_phys(pte.ppn()) + offset)
        } else {
            panic!("The va is not mapped! No pte");
        }
    }

    pub fn modify_flags(&self, vpn: usize, flags: PTEFlags) {
        let mut pte = self.translate_vpn_to_pte(vpn).unwrap();
        pte.set_flags(flags);
    }

}