//! fat32文件系统对 VFS File System 的具体实现

use alloc::{string::ToString, sync::Arc};
use spin::mutex::Mutex;

use crate::{config::fs::{BOOT_SECTOR_ID, FSINFO_SECTOR_ID, SECTOR_SIZE}, driver::BlockDevice, fs::{dentry::Dentry, file_system::{FSFlags, FSType, FileSystem, FileSystemMeta}, inode::Inode}};

use super::{bpb::load_bpb, fat::FatInfo, fat_dentry::FatDentry, fat_inode::FatInode, fsinfo::{load_fsinfo, FSInfo}, utility::{fat_sector, init_map}};


pub struct Fat32FileSystem {
    pub meta: FileSystemMeta,
}

impl FileSystem for Fat32FileSystem {
    fn metadata(&self) -> &FileSystemMeta {
        &self.meta
    }

    fn root_dentry(&self) -> Arc<dyn Dentry> {
        self.meta.root_dentry.clone()
    }
}

impl Fat32FileSystem {
    // 初始化文件系统
    pub fn new(
        mount_point: &str,
        dev_name: &str,
        dev: Arc<dyn BlockDevice>,
        flags: FSFlags,
    ) -> Self {
        // 1. 读 bpb
        let mut boot_sec_data: [u8; SECTOR_SIZE] = [0; SECTOR_SIZE];
        // 这个数据只需要读一次即可，所以不需要使用cache
        dev.read_block(BOOT_SECTOR_ID, &mut boot_sec_data[..]);
        let map = init_map();
        let bpb = load_bpb(map.clone(), boot_sec_data);
        if !bpb.is_valid() {
            todo!();
        };
        // 2. 读 fsinfo
        let mut fsinfo_sec_data: [u8; SECTOR_SIZE] = [0; SECTOR_SIZE];
        // TODO：这个部分的代码要修改，应该使用缓存方式的读写，因为这个数据会被修改
        dev.read_block(FSINFO_SECTOR_ID, &mut fsinfo_sec_data[..]);
        let fsinfo = load_fsinfo(map.clone(), fsinfo_sec_data);
        FSINFO.lock().from_another(fsinfo);
        // 3. 读 fat
        let fat_info = Arc::new(FatInfo::init(
            fat_sector(&bpb), 
            bpb.BPB_FATSz32 as usize,
            bpb.BPB_BytsPerSec as usize,
            bpb.BPB_SecPerClus as usize,
            bpb.BPB_NumFATs as usize,
            Some(dev.clone()))
        );
        // 4. 读data块的位置，构建root_dentry和root_inode
        // let data_start =  data_sector(&bpb);
        let root_inode = FatInode::new_from_root(fat_info.clone());
        let r_inode: Arc<dyn Inode> = Arc::new(root_inode);
        let root_dentry = FatDentry::new_from_root(fat_info.clone(), mount_point, Arc::clone(&r_inode));
        root_dentry.meta.inner.lock().d_inode = Arc::clone(&r_inode);
        let r_dentry: Arc<dyn Dentry> = Arc::new(root_dentry);
        // root_dentry.metadata().inner.lock().d_inode = Some(root_inode.clone());
        // // 5. 让root_dentry 加载所有的子结点
        r_dentry.load_all_child(Arc::clone(&r_dentry));
        Self {
            meta: FileSystemMeta {
                f_dev: dev_name.to_string(),
                f_type: FSType::VFAT,
                f_flags: flags,
                root_dentry: r_dentry,
                root_inode: r_inode,
            }
        }
    }
}

pub static FSINFO: Mutex<FSInfo> = Mutex::new(FSInfo::empty());