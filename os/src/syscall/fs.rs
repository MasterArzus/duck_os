//!
//! 

/* 
pub fn sys_uname() -> {
    
    let uname = UnameInfo::uname();

}

*/

/*
pub fn sys_dup(oldfd: usize) {
    let file = get_file_from_oldfd(oldfd);
    let new_fd = alloc_fd();
    fdtable.insert(fild, new_fd);
}

pub fn sys_dup3() {
    和上面的基本上一样
}

pub fn sys_chdir(path) {
    1. 先解析一下path  cwd_and_path()
    2. 在对相关的inode做一些时间上的更新
    3. 最后就是 current_process().cwd = path;
}

pub fn sys_getcwd() {
    1. 得到 current_process.cwd()
    2. 然后处理一下指针和buf之间的关系，将cwd的内容放入buf中。
}

pub fn sys_fstat() {
    现在只考虑文件
    1. 得到对应的inode
    2. 然后构造 stat
    3. 从inode中获取信息，并填入 stat 中
    4. 这一块的代码略微有点小多。
}

*/

use crate::process::hart::cpu::{get_cpu_id, get_cpu_local};

use super::error::SyscallResult;

pub fn sys_write(fd: usize, buf: usize, len: usize) -> SyscallResult {
    println!("[sys_write]: fd {}, len {}", fd, len);
    let fd_table = get_cpu_local(get_cpu_id())
        .current_pcb()
        .as_ref()
        .unwrap()
        .fd_table
        .clone();
    let file_info = {
        let locked_fd_table = fd_table.lock();
        locked_fd_table.fd_table.get(&fd).cloned()
    };
    if file_info.is_none() {
        panic!("The fd {} has no file", fd);
    } else {
        let file_info_unwrap = file_info.unwrap();
    // if file_info_unwrap.flags
        let flags = file_info_unwrap.flags.clone();
        let buf = unsafe { core::slice::from_raw_parts(buf as *const u8, len)};
        let ret = file_info_unwrap.file.write(buf, flags);
        Ok(ret.unwrap())
    }
}