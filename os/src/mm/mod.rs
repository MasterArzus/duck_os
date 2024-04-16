//! 内存模块
/*
暂时先处理 堆、页帧、虚拟逻辑段、地址之间转换、地址空间、物理地址（pma）；
*/
mod page_table;
pub mod vma;
pub mod allocator;
pub mod memory_set;
pub mod address;
pub mod pma;
mod vma_range;
pub mod cow;
/// 
pub mod type_cast;

/// 
pub fn init_mm() {
    allocator::heap::init_heap();
    allocator::frame::init_frame_allocator();
}