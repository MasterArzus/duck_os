//! 专门处理多种不同的 page_fault

/*
    1. page_fault种类
        1） sbrk
        2） mmap
        3） user_stack
        4)  user_heap
*/

use alloc::sync::Arc;
use riscv::register::scause::Scause;

use crate::mm::{address::{byte_array, ppn_to_phys, virt_to_vpn, VirtAddr}, allocator::frame::alloc_frame, page_table::PageTable, pma::Page, type_cast::{PTEFlags, PagePermission}, vma::VirtMemoryAddr};

use super::mem_set::MemeorySet;

pub trait PageFaultHandler: Send + Sync {
    // 懒分配：已经插入了对应的vma，只是没有做映射和物理帧分配
    // 所以只需要 映射 + 将分配的物理帧插入对应的 vma 中
    fn handler_page_fault(
        &self,
        vma: &VirtMemoryAddr,
        vaddr: VirtAddr,
        ms: Option<&MemeorySet>,
        pt: &mut PageTable,
    ) {}

    // TODO: 这个部分需要去参考手册，目前不懂
    fn is_legal(&self, scause: Scause) -> bool {
        todo!()
    }
}

#[derive(Clone)]
pub struct UStackPageFaultHandler {}

impl PageFaultHandler for UStackPageFaultHandler {
    fn handler_page_fault(
            &self,
            vma: &VirtMemoryAddr,
            vaddr: VirtAddr,
            _ms: Option<&MemeorySet>,
            pt: &mut PageTable,
        ) {
        let page = Page::new(PagePermission::from(vma.map_permission));
        vma.pma
            .get_unchecked_mut()
            .page_manager
            .insert(
                virt_to_vpn(vaddr), 
                Arc::new(page),
            );
        vma.map_all(pt);
        pt.activate();
    }

    fn is_legal(&self, scause: Scause) -> bool {
        todo!()
    }
}

#[derive(Clone)]
pub struct MmapPageFaultHandler {}

// TODO：有点复杂，暂时不完成，需要完成文件的回写。
impl PageFaultHandler for MmapPageFaultHandler {
    fn handler_page_fault(
            &self,
            vma: &VirtMemoryAddr,
            vaddr: VirtAddr,
            _ms: Option<&MemeorySet>,
            pt: &mut PageTable,
        ) {
        
    }

    fn is_legal(&self, scause: Scause) -> bool {
        todo!()
    }
}


#[derive(Clone)]
pub struct CowPageFaultHandler {}

impl PageFaultHandler for CowPageFaultHandler {
    fn handler_page_fault(
            &self,
            vma: &VirtMemoryAddr,
            vaddr: VirtAddr,
            ms: Option<&MemeorySet>,
            pt: &mut PageTable,
        ) {
        let pte = pt.find_pte(vaddr).unwrap();
        debug_assert!(pte.flags().contains(PTEFlags::COW));
        debug_assert!(!pte.flags().contains(PTEFlags::W));

        let mut flags = pte.flags() | PTEFlags::W;
        flags.remove(PTEFlags::COW);

        let page = ms
            .unwrap()
            .cow_manager
            .page_manager
            .get_unchecked_mut()
            .get(&virt_to_vpn(vaddr))
            .cloned()
            .unwrap();

        // 复制这个page 
        // 这里有一个暴力的做法：不管是不是最后一个指向这个页，统一的复制再创造一个新页。
        let new_page = Page::new_from_page(page.frame.ppn, page.permission);
        let vpn = virt_to_vpn(vaddr);
        pt.unmap(vpn);
        pt.map_one(vpn, new_page.frame.ppn, flags);
        pt.activate();
        ms.unwrap().cow_manager
            .page_manager
            .get_unchecked_mut()
            .remove(&vpn);

        // vma.pma.get_unchecked_mut().push_pma_page(vpn, page);
        ms.unwrap()
            .find_vm_by_vaddr(vaddr)
            .unwrap()
            .pma
            .get_unchecked_mut()
            .page_manager
            .insert(vpn, page);
        
    }

    fn is_legal(&self, scause: Scause) -> bool {
        todo!()
    }
}
