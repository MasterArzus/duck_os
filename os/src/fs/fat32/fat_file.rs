//! fat32文件系统 在磁盘上管理 File
//! 

use core::cmp::{max, min};

use alloc::{sync::Arc, vec::Vec};

use crate::config::fs::SECTOR_SIZE;

use super::{block_cache::get_block_cache, fat::{alloc_cluster, find_all_cluster, free_cluster, FatInfo}, fat_dentry::Position, utility::cluster_to_sector};

// TODO：不知道这里的meta需不需要Arc？
// TODO：这里有一个问题要仔细想一想？
/*
    这里实现的是磁盘上的文件读写。在实际读写文件时，首先通过open函数打开文件，即应该是把文件内容加载到内存中，
    然后通过调用 trait File中的函数进行读写，Titanix中是利用了 page，
*/
pub struct FatFile {
    pub fat_info: Arc<FatInfo>,
    clusters: Vec<usize>,
    size: usize,
}

impl FatFile {
    pub fn empty(fat_info: Arc<FatInfo>) -> Self {
        Self {
            fat_info: Arc::clone(&fat_info),
            clusters: Vec::new(),
            size: 0,
        }
    }

    pub fn init(fat_info: Arc<FatInfo>, pos: Position) -> Self {
        let mut file = Self::empty(fat_info.clone());
        file.cal_cluster_size(pos);
        file
    }

    pub fn cal_cluster_size(&mut self, pos: Position) {
        let cluster = find_all_cluster(self.fat_info.clone(), pos.data_cluster);
        self.clusters.clone_from(&cluster);
        if self.size == 0 {
            self.size = self.clusters.len() * self.fat_info.sec_per_clus * SECTOR_SIZE;
        }
    }

    pub fn first_cluster(&self) -> usize {
        self.clusters[0]
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
                        tmp_data = sector.data;
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
                        tmp_data = sector.data;
                    });
                }
                for i in cur_st..cur_ed {
                    data[i - st] = tmp_data[i - sector_st];
                }
                get_block_cache(sector_id + j, Arc::clone(&self.fat_info.dev.as_ref().unwrap()))
                    .lock()
                    .write(0, |sector: &mut Sector|{
                        sector.data = tmp_data;
                    })
            }
        }
        ed - st
    }

}

#[repr(C)]
pub struct Sector {
    pub data: [u8; SECTOR_SIZE],
}

