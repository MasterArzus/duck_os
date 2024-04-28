use alloc::collections::BTreeMap;

use crate::{
    config::mm::{LOW_LIMIT, UPPER_LIMIT}, 
    mm::{page_table::PageTable, type_cast::MapPermission, vma::VirtMemoryAddr}
};

use super::{SplitOverlap, UnmapOverlap};

// #[derive(Debug)]
pub struct VmaRange {
    pub segments: BTreeMap<usize, VirtMemoryAddr>,
}

impl VmaRange {
    // 初始化
    pub fn new() -> VmaRange {
        VmaRange {
            segments: BTreeMap::new(),
        }
    }
    // 插入一段虚拟逻辑段，不检查 用于mmap的任意一个内存逻辑段
    pub fn insert_raw(&mut self, vma: VirtMemoryAddr) {
        self.segments.insert(vma.start_vaddr, vma);
    }

    // mmap_anywhere
    // 寻找到一个合适的空间，然后insert进去
    pub fn mmap_anywhere(
        &mut self,
        hint: usize,
        len: usize,
        f: impl FnOnce(usize) -> VirtMemoryAddr,
    ) -> Option<usize> {
        self.find_anywhere(hint, len).map(|start| {
            self.insert_raw(f(start));
            start
        })
    }

    // mmap_fixed
    pub fn find_fixed(
        &mut self, 
        start: usize, 
        end: usize,
        pt: &mut PageTable,
    ) -> Option<usize> {
        // TODO: 检查这个地址
        self.unmap(start, end, pt);
        Some(start)
    }
    // unmap一段区间，要检查 
    // 适用在mmap_fixed时，需要删除掉要等待分配区间的虚拟地址
    // 用在 munmap()函数中
    pub fn unmap(&mut self, start: usize, end: usize, pt: &mut PageTable) {
        if let Some((_, vma)) = self.segments
            .iter_mut()
            .find(|(_, vma)| 
                vma.is_overlap(start, end) == true
            ) {
                match vma.unmap_if_overlap(start, end, pt) {
                    UnmapOverlap::Split(right_vma) => {
                        self.segments.insert(right_vma.start_vaddr, right_vma);
                    }
                    _ => {}
                }
            }
    }

    // 用在mmap_protect函数中
    pub fn mprotect(&mut self, start: usize, end: usize, new_flags: MapPermission, pt: &mut PageTable) {
        if let Some((_, vma)) = self.segments
            .iter_mut()
            .find(|(_, vma)|
                vma.is_overlap(start, end) == true
            ) {
                match vma.split_if_overlap(start, end, new_flags, pt) {
                    SplitOverlap::ShrinkLeft(right_vma) => {
                        self.segments.insert(right_vma.start_vaddr, right_vma);
                    }
                    SplitOverlap::ShrinkRight(right_vma) => {
                        self.segments.insert(right_vma.start_vaddr, right_vma);
                    }
                    SplitOverlap::Split(middle_vma, right_vma) => {
                        self.segments.insert(right_vma.start_vaddr, right_vma);
                        self.segments.insert(middle_vma.start_vaddr, middle_vma);
                    }
                    _ => {}
                }
            }
    }

    // 查找空的空间
    // 如果hint为0,则从最下面LOW_LIMIT开始分配空间
    // 如果hint不为0,则之前会保证其会在一个相对合理的位置，最后实在不行就分配在最高的那个vma的上面。
    // TODO：这里应该有一个固定空间用来分配
    pub fn find_anywhere(&self, hint: usize, len: usize) -> Option<usize> {
        let mut last_end = hint.max(LOW_LIMIT);
        for (start, vma) in self.segments.iter() {
            if last_end + len <= *start {
                return Some(last_end);
            }
            last_end = last_end.max(vma.end_vaddr);
        }
        if last_end + len <= UPPER_LIMIT {
            Some(last_end)
        } else {
            None
        }
    }

}