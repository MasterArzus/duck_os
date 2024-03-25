//! memory_set模块

/*
    1. 懒分配
        1）maturin: 在插入虚拟逻辑段的时候，使用push_lazy函数，插入时分配None的Frame
                    同时写入物理地址为0的pte。之后在page_fault中，使用get_frame来得到frame,
                    如果没有分配，则进行分配。
        2）Tiantix: 同样在插入虚拟逻辑段的时候，使用push_lazy函数，插入的时候直接不做映射。在发生
                    page_fault时，让对应的page_fault进行分配，并做好映射。
    2. 数据结构
        1) page_table
        2) areas 不同vma的集合，使用BTreeMap管理
        3）heap_range（与brk系统调用有关，可以用上那个区间管理的东西）
    3. 函数功能
        1）new 和 new_from_global(用户态的地址空间，有内核的映射)
        2）token
        3）通过vpn找vm_area
        4) 插入vma，主要就是两个push函数
        5）读写（其实就是使用下层的
        6）page_fault
        7）克隆地址空间 用在fork、clone函数
        还有一些莫名奇妙的函数，反正先不管吧，那些函数也是需要去看大量代码才能理解的，先实现一些通用的功能。
*/

use alloc::sync::Arc;
use log::info;
use riscv::register::scause::Scause;

use crate::{
    config::mm::MEMORY_END, 
    mm::{cow::CowManager, page_table::PageTable, pma::PhysMemoryAddr, type_cast::{MapPermission, PTEFlags}, vma::{MapType, VirtMemoryAddr, VmaType}, vma_range::vma_range::VmaRange}, utils::cell::SyncUnsafeCell,
};

use super::page_fault::MmapPageFaultHandler;

pub struct MemeorySet {
    // TODO: areas需不需要加锁？？
    pub areas: VmaRange,
    // 底下没有数据结构拥有页表，所以不用Arc，没有多个所有者
    // pt的借用关系难以管理，所以使用cell,但是为什么要使用sync? (一般情况下在多线程中传递引用)
    pub pt: SyncUnsafeCell<PageTable>,
    // is_user: bool,
    // pub heap_range
    pub cow_manager: CowManager,
}

// 这里没有选择上一把大锁，而是上细粒度锁，上在page_table
pub static mut KERNEL_SPACE: Option<MemeorySet> = None;

pub fn init_kernel_space() {
    unsafe {
        KERNEL_SPACE = Some(MemeorySet::new_kernel());
        KERNEL_SPACE.as_ref().unwrap().activate();
    }
}

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss();
    fn ebss();
    fn ekernel();
}


impl MemeorySet {
    // 现在仅考虑了最基本的 sections部分，还有trampoline等之类的还没有考虑
    pub fn new_kernel() -> Self {
        let mut kernel_memory_set = MemeorySet {
            areas: VmaRange::new(), 
            pt: SyncUnsafeCell::new(PageTable::new()),
            cow_manager: CowManager::new()
        };
        info!(
            "[kernel] initial kernel. [stext..etext] is [{:#x}..{:#x}]",
            stext as usize, etext as usize,
        );
        info!(
            "[kernel] initial kernel. [srodata..erodata] is [{:#x}..{:#x}]",
            srodata as usize, erodata as usize,
        );
        info!(
            "[kernel] initial kernel. [sdata..edata] is [{:#x}..{:#x}]",
            sdata as usize, edata as usize,
        );
        info!(
            "[kernel] initial kernel. [sbss..ebss] is [{:#x}..{:#x}]",
            sbss as usize, ebss as usize,
        );
        info!(
            "[kernel] initial kernel. [ekernel..MEMORY_END] is [{:#x}..{:#x}]",
            ekernel as usize, MEMORY_END as usize,
        );

        kernel_memory_set.push(
            VirtMemoryAddr::new(
                (stext as usize), 
                (etext as usize), 
                MapPermission::R | MapPermission::X, 
                MapType::Direct, 
                VmaType::Elf,
                None
            )
        );

        kernel_memory_set.push(
            VirtMemoryAddr::new(
                (srodata as usize), 
                (erodata as usize), 
                MapPermission::R, 
                MapType::Direct, 
                VmaType::Elf,
                None
            )
        );

        kernel_memory_set.push(
            VirtMemoryAddr::new(
                (sdata as usize), 
                (edata as usize), 
                MapPermission::R | MapPermission::W, 
                MapType::Direct, 
                VmaType::Elf,
                None
            )
        );

        kernel_memory_set.push(
            VirtMemoryAddr::new(
                (sbss as usize), 
                (ebss as usize), 
                MapPermission::R | MapPermission::W, 
                MapType::Direct, 
                VmaType::Elf,
                None
            )
        );

        kernel_memory_set.push(
            VirtMemoryAddr::new(
                (ekernel as usize), 
                (MEMORY_END as usize), 
                MapPermission::R | MapPermission::W, 
                MapType::Direct, 
                VmaType::PhysFrame,
                None
            )
        );
        info!("[kernel] Initail kernel finished!");
        kernel_memory_set
    }

    // 用户空间的页表需要做好映射
    pub fn new_user() -> Self {
        // 从内核中的页表里映射好了相关的数据
        let pt = SyncUnsafeCell::new(PageTable::new_user());
        Self {
            areas: VmaRange::new(),
            pt,
            cow_manager: CowManager::new(),
        }
    }

    pub fn token(&self) -> usize {
        self.pt.get_unchecked_mut().token()
    }

    pub fn activate(&self) {
        self.pt.get_unchecked_mut().activate();
    }

    // 分配一个vma空壳
    // 从vma_range中找到合适的start
    // TODO：还没考虑 back_file  后续应该要考虑修改构造vma的形式
    pub fn alloc_vma_anywhere(
        &self, 
        hint: usize, 
        len: usize, 
        map_permission: MapPermission,
        map_type: MapType
    ) -> Option<VirtMemoryAddr> {
        self.areas.find_anywhere(hint, len).map(|start_va| {
            VirtMemoryAddr::new(
                start_va,
                start_va + len,
                map_permission,
                map_type,
                VmaType::Mmap,
                None
            )
        })
    }

    // 分配 固定虚拟地址 的vma空壳
    // 非 Direct 类型
    // TODO： 还没考虑 back_file  后续应该要考虑修改构造vma的形式
    pub fn alloc_vma_fixed(
        &mut self,
        start: usize, 
        end: usize,
        map_permission: MapPermission,
        map_type: MapType,
    ) -> Option<VirtMemoryAddr> {
        self.areas.find_fixed(start, end, self.pt.get_unchecked_mut()).map(|start_va| {
            VirtMemoryAddr::new(
                start_va,
                end,
                map_permission,
                map_type,
                VmaType::Mmap,
                None
            )
        })
    }

    // 为 vma（分配物理页 + 做映射 + 插入memory_set）
    // TODO：是否需要传递含数据的物理页
    pub fn push(&mut self, vm_area: VirtMemoryAddr) {
        vm_area.map_all(self.pt.get_unchecked_mut());
        self.areas.insert_raw(vm_area);
    }

    // 懒分配，为 vma (插入memory_set)
    pub fn push_lazy(&mut self, vm_area: VirtMemoryAddr) {
        self.areas.insert_raw(vm_area);
    }

    pub fn mprotect(&mut self, start: usize, end: usize, new_flags: MapPermission) {
        self.areas.mprotect(start, end, new_flags, &mut self.pt.get_unchecked_mut())
    }

    pub fn find_vm_mut_by_vpn(&mut self, vpn: usize) -> Option<&mut VirtMemoryAddr> {
        if let Some((_, vma)) = 
            self.areas
                .segments
                .iter_mut()
                .find(
                    |(_, vma)|
                    vma.vma_range().contains(&vpn)
                ) {
                    Some(vma)
                }
        else {
            None
        }
    }

    pub fn find_vm_by_vaddr(&self, vaddr: usize) -> Option<& VirtMemoryAddr> {
        if let Some((_, vma)) = 
            self.areas
                .segments
                .iter()
                .find(|(_, vma)| vma.is_contain(vaddr)) {
                    Some(vma)
                }
        else {
            None
        }
    }

    pub fn handle_page_fault(&self, vaddr: usize, scause: Scause) {
        if let Some(vma) = self.find_vm_by_vaddr(vaddr) {
                vma.handle_page_fault(vaddr, self.pt.get_unchecked_mut())
            }
        // 对应的虚拟地址没有对应的虚拟地址空间！
        else {
            todo!()
        }
    }

    // 在fork, clone, exec 等系统调用中，用于创建一个新的地址空间。
    // 同时做好 COW （copy-on-write）
    pub fn from_user_lazily(another_ms: &Self) -> Self {
        let mut ms = MemeorySet::new_user();
        ms.cow_manager.from_other_cow(
            &another_ms.cow_manager, 
            &mut ms.pt.get_unchecked_mut()
        );
        for (_, vma) in another_ms
            .areas
            .segments
            .iter() {
                // 复制一模一样的虚拟逻辑段 TODO：有没有可能虚拟地址会重合？？？
                let new_vma = VirtMemoryAddr::from_another(&vma);
                for vpn in vma.vma_range() {
                    if let Some(page) = vma
                        .pma
                        .get_unchecked_mut()
                        .page_manager
                        .get(&vpn) {
                            // 这里存在 physical frame，所以要做一个特殊的映射
                            let old_pte = another_ms
                                .pt
                                .get_unchecked_mut()
                                .find_pte(vpn)
                                .unwrap();
                            let mut new_flags = old_pte.flags();
                            new_flags |= PTEFlags::COW;
                            new_flags.remove(PTEFlags::W);
                            old_pte.set_flags(new_flags);
                            let ppn = page.frame.ppn;
                            ms.pt.get_unchecked_mut().map_one(vpn, ppn, new_flags);
                            ms.cow_manager
                                .page_manager
                                .get_unchecked_mut()
                                .insert(vpn, page.clone());
                            another_ms.cow_manager
                                .page_manager
                                .get_unchecked_mut()
                                .insert(vpn, page.clone());
                        }
                    // 没有页，则可能是懒分配，或者是 Direct类型
                    else {
                        todo!()
                    }
                }
            ms.push_lazy(new_vma);
        }
        ms
    }

}