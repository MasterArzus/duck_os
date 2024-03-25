//! file 模块
//! 

/*
    file是抽象出来的，不是物理存储介质中的file。
    这个概念用于进程，表示进程打开该文件时，文件的整体状态。

    1. 数据结构
        1）dentry：对应的目录项
        2）mode：文件打开的模式
        3）pos：文件当前的位移量（文件指针）
        4）file：绑定的文件？？？

    2. 功能函数
        1）llseek：更新偏移量指针
        2）read / write：读写
        3）ioctl: io的相关控制
        4) fsync
        5) 其他的我也不知道了。
*/

use alloc::sync::Arc;
use spin::mutex::Mutex;

use super::{dentry::Dentry, info::{FileMode, OpenFlags}};


pub struct FileMeta {
    pub f_mode: FileMode,
    pub inner: Mutex<FileMetaInner>,
    // pub file: Option<Weak<dyn File>>
}

pub struct FileMetaInner {
    pub f_dentry: Arc<dyn Dentry>,
    pub f_pos: usize,
}

pub trait File {
    fn metadata(&self) -> &FileMeta;

    fn read(&self, buf: &mut [u8], flags: OpenFlags);
    fn write(&self, buf: &[u8], flags: OpenFlags);

}