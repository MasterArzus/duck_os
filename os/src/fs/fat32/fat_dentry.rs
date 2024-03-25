//! fat32文件系统对 VFS Dentry 的具体实现


use alloc::{string::{String, ToString}, sync::Arc, vec::Vec};
use spin::mutex::Mutex;

use crate::{config::fs::ROOT_CLUSTER_NUM, fs::{dentry::{cwd_and_path, dentry_name, path_plus_name, Dentry, DentryMeta, DentryMetaInner, DENTRY_CACHE}, info::{InodeMode, OpenFlags}, inode::Inode}};

use super::{block_cache::get_block_cache, data::{parse_child, DirEntry}, fat::{find_all_cluster, FatInfo}, fat_inode::{FatInode, NxtFreePos}, utility::cluster_to_sector, DirEntryStatus};

// 目录项的位置信息 （自身的cluster——通常在父目录中， offset——dentry的编号，内容的所在的cluster）
// 如果self_cluster == 0，说明没有父目录，即是根目录
#[derive(Debug, Clone, Copy)]
pub struct Position {
    // 对应目录项所在的位置
    pub self_cluster: usize,
    pub self_sector: usize,
    pub offset: usize,
    // 目录中的内容所在的位置
    pub data_cluster: usize,
}

impl Position {
    pub fn new_from_root() -> Self {
    /*  根目录中的信息：
        因为没有父目录，所以父目录有关的信息都为零
        根目录的data起始簇是2, 当前的空闲簇为2, 空闲的sector为0, 空闲的dentry位置为0
    */
        Self {
            self_cluster: 0,
            self_sector: 0,
            offset: 0,
            data_cluster: ROOT_CLUSTER_NUM,
        }
    }
    
    pub fn new_from_nxtpos(pos: NxtFreePos, data_clu: usize) -> Self {
        Self {
            self_cluster: pos.cluster,
            self_sector: pos.sector,
            offset: pos.offset,
            data_cluster: data_clu,
        }
    }
}

pub struct FatDentry {
    pub meta: DentryMeta,
    pub pos: Position,
    pub fat_info: Arc<FatInfo>,
}

impl Dentry for FatDentry {
    fn metadata(&self) -> &DentryMeta {
        &self.meta
    }

    // Assumption: path是合法的，format过的
    // function: 创建子inode和子目录，同时将数据都写在了磁盘上。
    // return: 子目录 Arc<dyn Dentry>
    // 在openat函数和mkdir函数中均有使用
    fn mkdir(&self, path: &str, mode: InodeMode) -> Arc<dyn Dentry> {
        let inode = Arc::clone(&self.meta.inner.lock().d_inode);
        let child_inode = FatInode::mkdir(
            Arc::clone(&inode), 
            path, 
            mode, 
            self.fat_info.clone());
        Arc::new(FatDentry::new_from_inode(child_inode,self.fat_info.clone(), path))
    }

    // TODO: 这个函数和 mkdir 暂时不知道有什么区别，所以实现一样！
    fn mknod(&self, path: &str, mode: InodeMode, dev_id: Option<usize>) -> Arc<dyn Dentry> {
        let inode = Arc::clone(&self.meta.inner.lock().d_inode);
        let child_inode = FatInode::mknod(
            Arc::clone(&inode), 
            path, 
            mode, 
            self.fat_info.clone(),
            dev_id,
        );
        Arc::new(FatDentry::new_from_inode(child_inode,self.fat_info.clone(), path))
    }

    // Assumption: name是单个名字
    // function：此时flags == CREATE, 所以需要创建对应的文件
    // 在openat函数中使用。
    fn open(&self, this: Arc<dyn Dentry>, name: &str, flags: OpenFlags) -> Option<Arc<dyn Dentry>> {
        // if let Some(inode) = &self.metadata().inner.lock().d_inode {
        
        let child_dentry: Arc<dyn Dentry>;
        if flags.contains(OpenFlags::O_DIRECTORY) {
            child_dentry = self.mkdir(
                &path_plus_name(&self.path(), name), 
                InodeMode::Directory);
        } else {
            child_dentry = self.mknod(
                &path_plus_name(&self.path(), name), 
                InodeMode::Regular,
                None,
            );
        }       
        self.meta.inner.lock().d_child.push(Arc::clone(&child_dentry));
        child_dentry.metadata().inner.lock().d_parent = Some(Arc::downgrade(&this));
        Some(child_dentry)
    }

    fn load_child(&self, this: Arc<dyn Dentry>) {
        let dev = Arc::clone(self.fat_info.dev.as_ref().expect("Block device is None"));
        let clusters: Vec<usize> = find_all_cluster(self.fat_info.clone(), self.data_cluster());
        // 2. 分别从其中的cluster读出所需要的数据
        let mut dir_pos: Vec<(DirEntry, Position)> = Vec::new();
        'outer: for current_cluster in clusters.iter() {
            let start_sector = cluster_to_sector(self.fat_info.clone(), *current_cluster);
            for sec_id in start_sector..start_sector + self.fat_info.sec_per_clus {
                for num in 0..16usize {
                    let dir = get_block_cache(sec_id, dev.clone())
                        .lock()
                        .read(num * core::mem::size_of::<DirEntry>(), |dir: &DirEntry| {
                            *dir
                        });
                    if dir.status() == DirEntryStatus::Empty {
                        break 'outer;
                    } else {
                        // TODO: 这里还有一些dir的情况没有考虑到！
                        let pos = Position {
                            self_cluster: *current_cluster,
                            self_sector: sec_id,
                            offset: num * core::mem::size_of::<DirEntry>(),
                            data_cluster: 0,
                        };
                        dir_pos.push((dir, pos));
                    }
                }
            }
        }
        // 3. 解析相关数据并转换为inode和dentry
        let childs = parse_child(&dir_pos, self.fat_info.clone());
        for child in childs.into_iter() {
            let name = child.meta.inner.lock().d_name.clone();
            // child.meta.inner.lock().d_path = self.meta.inner.lock().d_path.clone();
            // child.meta.inner.lock().d_path.push_str(&name);
            let cwd = self.meta.inner.lock().d_path.clone();
            child.meta.inner.lock().d_path = cwd_and_path(&name, &cwd);
            child.meta.inner.lock().d_parent = Some(Arc::downgrade(&this));
            // 维护好关系
            let child_rc: Arc<dyn Dentry> = Arc::new(child);
            self.meta.inner.lock().d_child.push(Arc::clone(&child_rc));
        }
    }
    
    // TODO: 不确定这种写法能不能正确的运行???? 如果不行,则要替换成每次只load一层.
    fn load_all_child(&self, this: Arc<dyn Dentry>) {
        let fa = this.clone();
        if fa.metadata()
            .inner
            .lock()
            .d_inode
            .metadata().i_mode != InodeMode::Directory {
            return;
        }
        fa.load_child(fa.clone());
        for child in &fa.metadata().inner.lock().d_child {
            child.load_all_child(Arc::clone(child));
        }
    }

    fn unlink(&self, child: Arc<dyn Dentry>) {
        let child_name = child.metadata().inner.lock().d_name.clone();
        let mut id: Option<usize> = None;
        for (idx, inode) in self.meta.inner.lock().d_child.iter().enumerate() {
            if child_name == inode.metadata().inner.lock().d_name {
                id = Some(idx);
                break;
            }
        }
        if id.is_none() {
            todo!()
        }
        DENTRY_CACHE.lock().remove(&child.metadata().inner.lock().d_path);
        child.metadata().inner.lock().d_inode.delete_data();
        self.meta.inner.lock().d_child.remove(id.unwrap());
    }
}

impl FatDentry {
    pub fn new_from_root(fat_info: Arc<FatInfo>, mount_point: &str, inode: Arc<dyn Inode> ) -> Self {
        Self {
            meta: DentryMeta {
                inner: Mutex::new(DentryMetaInner {
                    d_name: mount_point.to_string(),
                    d_path: mount_point.to_string(),
                    d_inode: inode,
                    d_parent: None,
                    d_child: Vec::new(),
                })
            },
            pos: Position { 
                /*  根目录中的信息：
                    因为没有父目录，所以父目录有关的信息都为零
                    根目录的data起始簇是2, 当前的空闲簇为2, 空闲的sector为0, 空闲的dentry位置为0
                */
                self_cluster: 0, 
                self_sector: 0, 
                offset: 0, 
                data_cluster: ROOT_CLUSTER_NUM, 
            },
            fat_info,
        }
    }

    fn data_cluster(&self) -> usize {
        self.pos.data_cluster
    }

    pub fn new_from_inode(inode: FatInode, fat_info: Arc<FatInfo>, path: &str) -> Self {
        let pos = inode.pos;
        Self {
            meta: DentryMeta {
                inner: Mutex::new(DentryMetaInner {
                    d_name: dentry_name(path).to_string(),
                    d_path: path.to_string(),
                    d_inode: Arc::new(inode),
                    d_parent: None,
                    d_child: Vec::new(),
                })
            },
            pos,
            fat_info,
        }
    }

    pub fn path(&self) -> String {
        self.meta.inner.lock().d_path.clone()
    }
}


    
