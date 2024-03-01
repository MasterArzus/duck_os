//! 内存模块
/*
暂时先处理 堆、页帧、虚拟逻辑段、地址之间转换、地址空间、物理地址（pma）；
*/
mod page_table;
mod vma;
mod allocator;
mod memory_set;
mod address;
mod pma;
mod vma_range;


pub fn init_mm() {
    allocator::heap::init_heap();    
}