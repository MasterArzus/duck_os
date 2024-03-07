//! 系统调用模块



pub fn sys_mmap(
    addr: usize,
    length: usize,
    prot: i32,
    flags: i32,
    fd: usize,
    offset: usize,
) -> usize {
    
    
    // 先 alloc 一个空壳vma
    // 再 push_lazily进去

    
    0
}

pub fn sys_mprotect(addr: usize, len: usize, prot: i32) -> usize {
    // 首先调用process中的memory_set

    // 然后调用memory_set中的 VmaRange modify()函数
    // let new_flags = MmapProt::from(prot);
    0
}