use alloc::sync::Arc;

use crate::driver::BLOCK_DEVICE;

use self::file_system::{FSFlags, FSType, FILE_SYSTEM_MANAGER};

pub mod fat32;
mod file;
pub mod file_system;
pub mod dentry;
pub mod inode;
mod info;
pub mod fd_table;
pub mod page_cache;
pub mod stdio;


pub const AT_FDCWD: isize = -100;

pub fn init() {
    FILE_SYSTEM_MANAGER
        .mount(
            "/",
         "/dev/vda2",
        Some(Arc::clone(&BLOCK_DEVICE.lock().as_ref().unwrap())), 
        FSType::VFAT, 
        FSFlags::MS_NOSUID,
    );
}