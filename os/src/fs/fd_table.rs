//! 文件描述符表

use alloc::{ sync::Arc, vec::Vec};
use bitmap_allocator::{BitAlloc, BitAlloc4K};
use hashbrown::HashMap;
use spin::Mutex;

use crate::config::fs::MAX_FD;

use super::{file::File, info::OpenFlags, stdio::{Stderr, Stdin, Stdout, STDERR, STDIN, STDOUT}};

type FdAllocatorImpl = BitAlloc4K;

trait FdAllocator {
    fn init(&mut self);
    fn alloc_fd(&mut self) -> Option<usize>;
    fn dealloc_fd(&mut self, fd: usize);
}

impl FdAllocator for FdAllocatorImpl {
    fn init(&mut self) {
        self.insert(3..MAX_FD);    
    }

    fn alloc_fd(&mut self) -> Option<usize> {
        self.alloc()
    }

    fn dealloc_fd(&mut self, fd: usize) {
        self.dealloc(fd);
    }
}

pub static FD_ALLOCATOR: Mutex<FdAllocatorImpl> = Mutex::new(FdAllocatorImpl::DEFAULT);

pub fn init_fd_allocator() {
    FD_ALLOCATOR.lock().init();
}

pub fn alloc_fd() -> Option<usize> {
    FD_ALLOCATOR.lock().alloc_fd()
}

pub fn dealloc_fd(fd: usize) {
    FD_ALLOCATOR.lock().dealloc_fd(fd);
}

// 这里的fd没有实现RAII，之后根据需求再判断要不要实现
// TODO: 这里的fd_table完全可以更换为 hash table，而不采用BTreeMap
pub struct FdTable {
    pub fd_table: HashMap<usize, FdInfo>,
}

impl FdTable {
    pub fn insert(&mut self, fd: usize, fd_info: FdInfo) {
        self.fd_table.insert(fd, fd_info);
    }

    pub fn init_fdtable() -> Self {
        let mut fd_table = HashMap::new();
        fd_table.insert(
            STDIN, 
            FdInfo::new(Arc::new(Stdin), OpenFlags::O_RDONLY)
        );
        fd_table.insert(
            STDOUT, 
            FdInfo::new(Arc::new(Stdout), OpenFlags::O_WRONLY)
        );
        fd_table.insert(
            STDERR, 
            FdInfo::new(Arc::new(Stderr), OpenFlags::O_WRONLY)
        );
        Self { fd_table }
    }

    pub fn close_exec(&mut self) {
        let mut remove_idx: Vec<usize> = Vec::new();
        for (idx, _) in self.fd_table.iter() {
            if self.fd_table.get(idx)
                .unwrap()
                .flags.contains(OpenFlags::O_CLOEXEC) {
                remove_idx.push(*idx);
            }
        }
        for idx in remove_idx {
            self.fd_table.remove(&idx);
        }
    }
}

#[derive(Clone)]
pub struct FdInfo {
    pub file: Arc<dyn File>,
    pub flags: OpenFlags,
}

impl FdInfo {
    pub fn new(file: Arc<dyn File>, flags: OpenFlags) -> Self {
        Self { file, flags }
    }
}

