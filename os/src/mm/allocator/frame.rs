/*！
   页帧模块，通过包装 bitmap_allocator 实现。    
*/

/*
    1. 位图分配器用来分配 bit
    2. 页帧分配器就分配 FrameTracker
    3. 页帧分配器的功能
        1） alloc
        2) dealloc
        3) alloc_continue
    4. 一个全局的页帧分配器
        1）初始化，全新
        2）设置分配的区间
        3）alloc
        4) alloc_continue
        5) dealloc
*/

use core::fmt::{self, Formatter, Debug};

use alloc::vec::Vec;
use spin::Mutex;
use bitmap_allocator::BitAlloc;

use crate::{config::mm::MEMORY_END, mm::address::{byte_array, phys_to_ppn, phys_to_ppn_next, ppn_to_phys}};

// 16M * 4K = 64G，所以可以分配64G的内存
// 如果再小一点，只有4G的选项可以选择，这又有点小
type FrameAllocatorImpl = bitmap_allocator::BitAlloc16M;

pub struct FrameTracker {
    pub ppn: usize,
}

impl FrameTracker {
    pub fn new(ppn: usize) -> Self {
        let bytes_array = byte_array(ppn_to_phys(ppn));
        for i in bytes_array {
            *i = 0;
        }
        Self { ppn }
    }
}

impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker: ppn={:#x}", self.ppn))
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        dealloc_frame(self.ppn)
    }
}

trait FrameAllocator {
    fn init(&mut self, start_ppn: usize, end_ppn: usize);
    // 返回页帧号 ppn
    fn alloc_frame(&mut self) -> Option<usize>;
    fn dealloc_frame(&mut self, ppn: usize);
    // 返回起始页帧号 ppn
    fn alloc_contiguous_frame(&mut self, num: usize) -> Option<usize>;
}

impl FrameAllocator for FrameAllocatorImpl {
    // range is [start_ppn, end_ppn)
    fn init(&mut self, start_ppn: usize, end_ppn: usize) {
        self.insert(start_ppn..end_ppn)
    }

    fn alloc_frame(&mut self) -> Option<usize> {
        self.alloc()
    }

    fn dealloc_frame(&mut self, ppn: usize) {
        self.dealloc(ppn)
    }

    fn alloc_contiguous_frame(&mut self, num: usize) -> Option<usize> {
        self.alloc_contiguous(num, 0)
    }
}

// TODO: 现在是一把大锁，以后估计要换锁。
pub static FRAME_ALLOCATOR: Mutex<FrameAllocatorImpl> = Mutex::new(FrameAllocatorImpl::DEFAULT);

pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }    
    let start_ppn = phys_to_ppn_next(ekernel as usize);
    let end_ppn = phys_to_ppn(MEMORY_END);
    FRAME_ALLOCATOR
        .lock()
        .init(start_ppn, end_ppn);
}

pub fn alloc_frame() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .lock()
        .alloc_frame()
        .map(FrameTracker::new)
}

pub fn alloc_contiguous_frame(num: usize) -> Vec<FrameTracker> {
    let mut frames: Vec<FrameTracker> = Vec::new();
    FRAME_ALLOCATOR
        .lock()
        .alloc_contiguous_frame(num)
        .map(|ppn_start|{
            for ppn in ppn_start..ppn_start+num {
                frames.push(FrameTracker::new(ppn))
            }
        });
    frames
}

pub fn dealloc_frame(ppn: usize) {
    FRAME_ALLOCATOR.lock().dealloc_frame(ppn);
}