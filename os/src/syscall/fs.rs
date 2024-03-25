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