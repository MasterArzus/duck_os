pub mod fat32;
mod file;
pub mod file_system;
mod dentry;
mod inode;
mod info;


pub const AT_FDCWD: isize = -100;