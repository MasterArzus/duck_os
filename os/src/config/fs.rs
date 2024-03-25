//!
//! 

// Default: 一个扇区的大小是512字节。其实这个大小应该从bpb中读出来的。
pub const SECTOR_SIZE: usize = 512;

// Default: boot_sector_id 是 0
pub const BOOT_SECTOR_ID: usize = 0;

// Default FSInfo sector id 是 1
pub const FSINFO_SECTOR_ID: usize = 1;

// cache manager 的大小
pub const SECTOR_CACHE_SIZE: usize = 16;

// 根目录data cluster的位置
pub const ROOT_CLUSTER_NUM: usize = 2;

// 一个扇区能存放Dirdentry的最大个数
pub const MAX_DIRENT_PER_SECTOR: usize = 16;