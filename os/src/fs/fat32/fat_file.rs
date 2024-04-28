//! fat32文件系统 在磁盘上管理 File 和 抽象出来的内存File

use core::cmp::{max, min};

use alloc::{sync::{Arc, Weak}, vec::Vec};

use crate::{config::{fs::SECTOR_SIZE, mm::PAGE_SIZE}, fs::{file::{File, FileMeta, SeekFrom}, info::{OpenFlags, TimeSpec}}};

use super::{block_cache::get_block_cache, data, fat::{alloc_cluster, find_all_cluster, free_cluster, FatInfo}, fat_dentry::Position, fat_inode::NxtFreePos, utility::cluster_to_sector};

// TODO：这里有一个问题要仔细想一想？
/*
    这里实现的是磁盘上的文件读写。在实际读写文件时，首先通过open函数打开文件，即应该是把文件内容加载到内存中，
    然后通过调用 trait File中的函数进行读写，Titanix中是利用了 page，
*/
// 磁盘上的文件，
pub struct FatDiskFile {
    pub fat_info: Arc<FatInfo>,
    clusters: Vec<usize>,
    pub size: usize,
}

impl FatDiskFile {
    pub fn empty(fat_info: Arc<FatInfo>) -> Self {
        Self {
            fat_info: Arc::clone(&fat_info),
            clusters: Vec::new(),
            size: 0,
        }
    }

    pub fn init(fat_info: Arc<FatInfo>, pos: Position) -> Self {
        let mut file = Self::empty(Arc::clone(&fat_info));
        file.cal_cluster_size(pos);
        file
    }

    fn cal_cluster_size(&mut self, pos: Position) {
        let cluster = find_all_cluster(self.fat_info.clone(), pos.data_cluster);
        self.clusters.clone_from(&cluster);
        if self.size == 0 {
            self.size = self.clusters.len() * self.fat_info.sec_per_clus * SECTOR_SIZE;
        }
    }

    pub fn first_cluster(&self) -> usize {
        self.clusters[0]
    }

    pub fn last_cluster(&self) -> usize {
        *self.clusters.last().unwrap()
    }

    // 此时默认了不用进行 cluster 的重新计算
    pub fn modify_size(&mut self, diff_size: isize, pos: Position) -> usize {
        if diff_size < 0 && self.size as isize + diff_size > 0 {
            let new_sz = (self.size as isize + diff_size) as usize;
            let clus_num = (new_sz + self.fat_info.sec_per_clus * SECTOR_SIZE - 1)
                / (self.fat_info.sec_per_clus * SECTOR_SIZE);
            while self.clusters.len() > clus_num {
                let end_clu = self.clusters.pop().unwrap();
                if self.clusters.len() > 0 {
                    let pre_clu = *self.clusters.last().unwrap();
                    free_cluster(end_clu, Some(pre_clu), Arc::clone(&self.fat_info));
                } else {
                    free_cluster(end_clu, None, Arc::clone(&self.fat_info));
                }
            }
            self.size = new_sz;
        } else if diff_size > 0 {
            let new_sz = (self.size as isize + diff_size) as usize;
            let clus_num = (new_sz + self.fat_info.sec_per_clus * SECTOR_SIZE - 1)
                / (self.fat_info.sec_per_clus * SECTOR_SIZE);
            while self.clusters.len() < clus_num {
                let end_clu = *self.clusters.last().unwrap();
                let new_clu: usize;
                if self.clusters.is_empty() {
                    new_clu = alloc_cluster(None, Arc::clone(&self.fat_info)).unwrap();
                } else {
                    new_clu = alloc_cluster(Some(end_clu), Arc::clone(&self.fat_info)).unwrap();
                }
                self.clusters.push(new_clu);
            }
            self.size = new_sz;
        }
        self.cal_cluster_size(pos);
        self.size
    }

    // TODO: 再检查一下
    pub fn read(&mut self, data: &mut [u8], offset: usize) -> usize {
        let st = min(offset, self.size);
        let ed = min(offset + data.len(), self.size);
        let st_cluster = st / (self.fat_info.sec_per_clus * SECTOR_SIZE);
        let ed_cluster = (ed + self.fat_info.sec_per_clus * SECTOR_SIZE - 1)
            / (self.fat_info.sec_per_clus * SECTOR_SIZE);
        for clu_id in st_cluster..ed_cluster {
            let cluster_id = self.clusters[clu_id];
            let sector_id = cluster_to_sector(Arc::clone(&self.fat_info), cluster_id);
            for j in 0..self.fat_info.sec_per_clus {
                let off = clu_id * self.fat_info.sec_per_clus + j;
                let sector_st = off * SECTOR_SIZE;
                let sector_ed = sector_st + SECTOR_SIZE;
                if sector_ed <= st || sector_st >= ed {
                    continue;
                }
                let cur_st = max(sector_st, st);
                let cur_ed = min(sector_ed, ed);
                let mut tmp_data: [u8; SECTOR_SIZE] = [0; SECTOR_SIZE];
                get_block_cache(sector_id + j, Arc::clone(&self.fat_info.dev.as_ref().unwrap()))
                    .lock()
                    .read(0, |sector: &Sector|{
                        // tmp_data = sector.data;
                        tmp_data.copy_from_slice(sector);
                });
                for i in cur_st..cur_ed {
                    data[i - st] = tmp_data[i - sector_st];
                }
            }
        }
        ed - st
    }

    // TODO: 再检查一下
    pub fn write(&mut self, data: &mut [u8], offset: usize, pos: Position) -> usize {
        let st = min(offset, self.size);
        let ed = min(offset + data.len(), self.size);
        if self.size < ed {
            self.modify_size((ed - self.size) as isize, pos);
        }
        let st_cluster = st / (self.fat_info.sec_per_clus * SECTOR_SIZE);
        let ed_cluster = (ed + self.fat_info.sec_per_clus * SECTOR_SIZE - 1)
            / (self.fat_info.sec_per_clus * SECTOR_SIZE);
        for clu_id in st_cluster..ed_cluster {
            let cluster_id = self.clusters[clu_id];
            let sector_id = cluster_to_sector(Arc::clone(&self.fat_info), cluster_id);
            for j in 0..self.fat_info.sec_per_clus {
                let off = clu_id * self.fat_info.sec_per_clus + j;
                let sector_st = off * SECTOR_SIZE;
                let sector_ed = sector_st + SECTOR_SIZE;
                if sector_ed <= st || sector_st >= ed {
                    continue;
                }
                let cur_st = max(sector_st, st);
                let cur_ed = min(sector_ed, ed);
                let mut tmp_data: [u8; SECTOR_SIZE] = [0; SECTOR_SIZE];
                if cur_st != sector_st || cur_ed != sector_ed {
                    get_block_cache(sector_id + j, Arc::clone(&self.fat_info.dev.as_ref().unwrap()))
                    .lock()
                    .read(0, |sector: &Sector|{
                        // tmp_data = sector.data;
                        tmp_data.copy_from_slice(sector);
                    });
                }
                for i in cur_st..cur_ed {
                    data[i - st] = tmp_data[i - sector_st];
                }
                get_block_cache(sector_id + j, Arc::clone(&self.fat_info.dev.as_ref().unwrap()))
                    .lock()
                    .write(0, |sector: &mut Sector|{
                        // sector.data = tmp_data;
                        sector.copy_from_slice(&tmp_data);
                    })
            }
        }
        ed - st
    }

    pub fn read_all(&mut self) -> Vec<u8> {
        let mut data_vec = Vec::new();
        data_vec.resize(self.size, 0);
        self.read(&mut data_vec, 0);
        data_vec
    }

}

type Sector = [u8; SECTOR_SIZE];

// fat 抽象出来的内存上的文件
pub struct FatMemFile {
    pub meta: FileMeta,
}

impl FatMemFile {
    pub fn init(meta: FileMeta) -> Self {
        Self { meta }
    }
}

impl File for FatMemFile {
    fn metadata(&self) -> &FileMeta {
        &self.meta
    }

    fn seek(&self, seek: SeekFrom) {
        let mut meta_lock = self.meta.inner.lock();
        match seek {
            SeekFrom::Current(pos) => {
                if pos < 0 {
                    meta_lock.f_pos -= pos.abs() as usize;
                } else {
                    meta_lock.f_pos += pos as usize;
                }
            },
            SeekFrom::End(pos) => {
                // TODO: 不知道怎么处理，不知道这个data_len是什么东西！
                meta_lock.f_pos = pos as usize;
            },
            SeekFrom::Start(pos) => {
                meta_lock.f_pos += pos;
            }
        }
    }

    fn trucate(&self, size: usize) {
        
    }

    // 将文件的offset(pos)之后的数据读入buf中
    // offset >> PAGE_SIZE: page的索引值； 后几位：page中的offset
    // TODO：需要修改这里的data_len
    fn read(&self, buf: &mut [u8], _flags: OpenFlags) -> Option<usize> {
        let data_len = 0;
        let pos = self.meta.inner.lock().f_pos;
        let page_cache = Arc::clone(self.meta.page_cache.as_ref().unwrap());
        let inode = Arc::downgrade(&self.meta.f_inode);

        let max_len = buf.len().min(data_len - pos);
        let mut buf_offset = 0 as usize;
        let mut file_offset = pos;
        let mut total_len = 0usize;
        
        loop {
            let page = page_cache.find_page(file_offset, Weak::clone(&inode));
            let page_offset = file_offset % PAGE_SIZE;
            let mut byte = PAGE_SIZE - page_offset;
            if total_len + byte > max_len {
                let old_byte = byte;
                byte = max_len - total_len;
                total_len += old_byte;
            } else {
                total_len += byte;
            }
            page.read(page_offset, &mut buf[buf_offset..buf_offset+byte]);
            buf_offset += byte;
            file_offset += byte;
            if total_len > max_len {
                break;
            }
        }
        // TODO: 没搞懂这个东西的逻辑
        self.meta.f_inode.metadata().inner.lock().i_atime = TimeSpec::new();
        self.meta.inner.lock().f_pos = file_offset;
        // TODO: 不一定是这个值，这里没有仔细思考而随意设置的一个值
        Some(max_len)
    }

    // TODO: 多个进程访问一个文件的问题？
    fn write(&self, buf: &[u8], _flags: OpenFlags) -> Option<usize> {
        let pos = self.meta.inner.lock().f_pos;
        let page_cache = Arc::clone(self.meta.page_cache.as_ref().unwrap());
        let inode = Arc::downgrade(&self.meta.f_inode);

        let mut buf_offset = 0 as usize;
        let mut file_offset = pos;
        let mut total_len = 0usize;
        
        loop {
            let data_len = 0;
            let max_len = buf.len().min(data_len - pos);
            
            let page = page_cache.find_page(file_offset, Weak::clone(&inode));
            let page_offset = file_offset % PAGE_SIZE;
            let mut byte = PAGE_SIZE - page_offset;
            if total_len + byte > max_len {
                let old_byte = byte;
                byte = max_len - total_len;
                total_len += old_byte;
            } else {
                total_len += byte;
            }
            page.write(page_offset, &buf[buf_offset..buf_offset+byte]);
            buf_offset += byte;
            file_offset += byte;
            if total_len > max_len {
                break;
            }
        }
        // TODO: 没搞懂这个东西的逻辑
        let mut inner_lock = self.meta.f_inode.metadata().inner.lock();
        inner_lock.i_atime = TimeSpec::new();
        inner_lock.i_ctime = inner_lock.i_atime;
        inner_lock.i_mtime = inner_lock.i_atime;
        self.meta.inner.lock().f_pos = file_offset;
        // TODO: 随意设置的一个值
        Some(total_len)
    }
}


