//! 简化组合版 superblock + vfsmount


/*
    1.数据结构
        1）dev: 设备标识符
        2) type: 文件系统类型
        3) flags: 挂载标志
        4) root: 目录挂载点（Dentry）
        5) inode: 文件系统根inode
        6) dirty: 回写链表（待定）
        7) mnt_parent: 父文件系统（待定）

    2. 功能
        1）得到根 inode

    3. 一个全局的管理器
        负责挂载和解挂载，同时负责找到根文件系统
        2) mount unmount
*/

use alloc::{collections::BTreeMap, string::{String, ToString}, sync::Arc};
use bitflags::bitflags;
use crate::fs::dentry::DENTRY_CACHE;
use spin::mutex::Mutex;

use crate::driver::BlockDevice;

use super::{dentry::Dentry, fat32::fat_fs::Fat32FileSystem, file::File, inode::Inode};

#[derive(Clone, Copy)]
pub enum FSType {
    VFAT,
    EXT2,
    Proc,
    Dev,
    Tmpfs,
}
// https://man7.org/linux/man-pages/man2/mount.2.html
bitflags!  {
    pub struct FSFlags: u16 {
        const MS_RDONLY = 1 << 0; //只读挂载文件系统
        const MS_NOSUID = 1 << 1; //禁止设置文件的 SUID 和 SGID 位
        const MS_NODEV = 1 << 2; // 禁止访问设备文件
        const MS_NOEXEC = 1 << 3; // 禁止在文件系统上执行可执行文件
        const MS_SYNCHRONOUS = 1 << 4; // 同步挂载，即对文件系统的写操作立即同步到磁盘
        const MS_REMOUNT = 1 << 5; // 重新挂载文件系统，允许修改挂载标志
        const MS_MANDLOCK = 1 << 6; // 启用强制锁定
        const MS_DIRSYNC = 1 << 7; // 同步目录更新
        const MS_NOATIME = 1 << 10; // 不更新访问时间
        const MS_BIND = 1 << 12; // 绑定挂载，即创建目录或文件的镜像
        const MS_MOVE = 1 << 13; // 原子移动挂载点
    }
}

pub struct FileSystemMeta {
    pub f_dev: String,
    pub f_type: FSType,
    pub f_flags: FSFlags,
    pub root_dentry: Arc<dyn Dentry>,
    pub root_inode: Arc<dyn Inode>,
    /*
    pub mnt_parent: Option<Arc<dyn FileSystem>>,
    pub is_root_mnt: bool,
    pub dirty_inode: Vec<Inode>,
     */
}

pub trait FileSystem: Send + Sync {
    fn root_dentry(&self) -> Arc<dyn Dentry>;
    fn metadata(&self) -> &FileSystemMeta;
}

pub struct FileSystemManager {
    // (mounting point name, FileSystem)
    // 可以换成 hashmap
    pub manager: Mutex<BTreeMap<String, Arc<dyn FileSystem>>>,
}

impl FileSystemManager {
    pub fn new() -> FileSystemManager {
        FileSystemManager { 
            manager: Mutex::new(BTreeMap::new()), 
        }
    }

    // 返回根文件系统的引用
    pub fn root_fs(&self) -> Arc<dyn FileSystem> {
        self.manager.lock().get("/").unwrap().clone()
    }

    pub fn root_dentry(&self) -> Arc<dyn Dentry> {
        self.manager.lock().get("/").unwrap().root_dentry()
    }

    pub fn mount(
        &self,
        mount_point: &str,
        dev_name: &str,
        device: Option<Arc<dyn BlockDevice>>,
        fs_type: FSType,
        flags: FSFlags,
    ) {
        let fs: Arc<dyn FileSystem> = match fs_type {
            FSType::VFAT => {
                Arc::new(Fat32FileSystem::new(
                    mount_point, 
                    dev_name, 
                    Arc::clone(&device.unwrap()), 
                    flags,
                ))
            }
            _ => {
                todo!()
            }
        };
        DENTRY_CACHE.lock().insert(
            mount_point.to_string(), 
            fs.metadata().root_dentry.clone()
        );
        FILE_SYSTEM_MANAGER.manager.lock().insert(
            mount_point.to_string(),
            Arc::clone(&fs),
        );
    }

    // 找到fs，和fs中的meta, 移除inode_cache, fs_manager中的数据。
    pub fn unmount(&self, mount_point: &str) {
        let mut fs_manager = FILE_SYSTEM_MANAGER.manager.lock();
        let fs_op = fs_manager.get(mount_point);
        if fs_op.is_none() {
            todo!();
        }

        DENTRY_CACHE.lock().remove(mount_point);
        fs_manager.remove(mount_point);
    }

}

lazy_static::lazy_static! {
    pub static ref FILE_SYSTEM_MANAGER: FileSystemManager = FileSystemManager::new(); 
}
