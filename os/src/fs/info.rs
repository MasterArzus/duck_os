//! 存放各种定义的模块

use bitflags::bitflags;

use crate::timer::{current_time_ms, current_time_ns};

// https://man7.org/linux/man-pages/man7/inode.7.html 手册中是8进制
#[derive(PartialEq, Clone, Copy)]
pub enum InodeModeMask {
    S_IFMT = 0xF000,  // bit mask for the file type bit field
    S_IFSOCK = 0xC000, //  socket
    S_IFLNK = 0xA000,  // symbolic link
    S_IFREG = 0x8000,  // regular file0
    S_IFBLK = 0x6000,  // block device
    S_IFDIR = 0x4000,  // directory
    S_IFCHR = 0x2000,  // character devxice
    S_IFIFO = 0x1000,  // FIFO
    
    // 剩下的低12位和组别有关，暂时不实现
}

#[derive(PartialEq)]
pub enum InodeMode {
    Socket,
    Link,
    Regular,
    Block,
    Directory,
    Char,
    FIFO,
}

// https://man7.org/linux/man-pages/man3/timespec.3type.html
/*
秒（Second）：通常用 s 表示，是国际标准的时间单位。
毫秒（Millisecond）：1 毫秒等于 0.001 秒，通常用 ms 表示。
微秒（Microsecond）：1 微秒等于 0.000001 秒，通常用 μs 表示，也可以用 us 表示。
纳秒（Nanosecond）：1 纳秒等于 0.000000001 秒，通常用 ns 表示。 */

#[derive(Clone, Copy, Default)]
pub struct TimeSpec {
    pub tv_sec: usize, /* 秒 */
    pub tv_nsec: usize, /*Nanoseconds 0 ~ 999'999'999 */
}

impl TimeSpec {
    pub fn new() -> TimeSpec {
        let current_time = current_time_ms();
        Self {
            tv_sec: current_time / 1000,
            tv_nsec: current_time_ns(),
        }
    }

    pub fn update(&mut self) {
        self.tv_sec =  current_time_ms() / 1000;
        self.tv_nsec = current_time_ns();
    }
}


// https://man7.org/linux/man-pages/man2/open.2.html
bitflags! {
    pub struct OpenFlags: u32 {
        // 只读模式
        const O_RDONLY = 0;
        // 只写模式
        const O_WRONLY = 1 << 0;
        // 读写模式
        const O_RDWR = 1 << 1;
        // 如果文件不存在，则创建文件
        const O_CREAT = 1 << 6;
        // 与 O_CREAT 一起使用，如果文件已经存在，则打开失败
        const O_EXCL = 1 << 7;
        // 如果文件存在，并且以写方式打开，则将文件截断为零长度
        const O_TRUNC = 1 << 9;
        // 在写入文件时始终追加到文件末尾
        const O_APPEND = 1 << 10;
        // 非阻塞模式
        const O_NONBLOCK = 1 << 11;
        // 同步写模式，要求每次写操作都同步到存储介质上
        const O_SYNC = 1 << 12;
        // 目录
        const O_DIRECTORY = 1 << 16;

        const O_NOFOLLOW = 1 << 17;
        const O_CLOEXEC = 1 << 19;
    }
}



bitflags! {
    // TODO: 不确定，要根据具体的情况来设计。
    // 由 openat系统调用决定！
    pub struct FileMode: u8 {
        const FMODE_READ = 1 << 0;
        const FMODE_WRITE = 1 << 1;
    }
}


