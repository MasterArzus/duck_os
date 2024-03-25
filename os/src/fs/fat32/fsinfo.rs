//! File System Information(FSInfo) Structure

use alloc::{collections::BTreeMap, string::String, sync::Arc};

#[allow(non_snake_case)]
#[derive(Clone, Copy, Default)]
// 有两个数据不记录，因为为0. 没什么用
pub struct FSInfo {
    pub FSI_LeadSig: u32,
    pub FSI_StrucSig: u32,
    pub FSI_Free_Count: u32,
    pub FSI_Nxt_Free: u32,
    pub FSI_TrailSig: u32,
}

impl FSInfo {
    pub const fn empty() -> Self {
        Self {
            FSI_LeadSig: 0, 
            FSI_StrucSig: 0, 
            FSI_Free_Count: 0, 
            FSI_Nxt_Free: 0, 
            FSI_TrailSig: 0 
        }
    }

    pub fn from_another(&mut self, an: Self) {
        self.FSI_LeadSig = an.FSI_LeadSig;
        self.FSI_StrucSig = an.FSI_StrucSig;
        self.FSI_Free_Count = an.FSI_Free_Count;
        self.FSI_Nxt_Free = an.FSI_Nxt_Free;
        self.FSI_TrailSig = an.FSI_TrailSig;
    }

    pub fn update(&mut self, free_count: u32, nxt_free: u32) {
        self.FSI_Free_Count = free_count;
        self.FSI_Nxt_Free = nxt_free;
    }

    pub fn free_count(&self) -> usize {
        self.FSI_Free_Count as usize
    }

    pub fn nxt_free(&self) -> usize {
        self.FSI_Free_Count as usize
    }

    pub fn alloc_cluster(&mut self) -> usize {
        let nxt_free = self.nxt_free();
        self.FSI_Free_Count -= 1;
        self.FSI_Nxt_Free += 1;
        nxt_free
    }

    pub fn free_cluster(&mut self) {
        self.FSI_Free_Count += 1;
    }

}

pub fn load_fn<T: Copy>(dst: &mut T, src: &[u8], offset: usize, size: usize) {
    unsafe {
        core::ptr::copy_nonoverlapping(&src[offset], dst as *mut _ as *mut u8, size);
    }
}

pub fn load_fsinfo(map: Arc<BTreeMap<String, (usize, usize)>>, data: [u8; 512]) -> FSInfo {
    let mut fs_info = FSInfo::default();

    macro_rules! load {
        ($a: expr, $b: expr) => {
            if let Some((offset, size)) = map.get($b) {
                load_fn(&mut $a, &data, *offset, *size);
            }
        };
    }

    load!(fs_info.FSI_LeadSig, "FSI_LeadSig");
    load!(fs_info.FSI_StrucSig, "FSI_StrucSig");
    load!(fs_info.FSI_Free_Count, "FSI_Free_Count");
    load!(fs_info.FSI_Nxt_Free, "FSI_Nxt_Free");
    load!(fs_info.FSI_TrailSig, "FSI_TrailSig");

    fs_info

}

impl FSInfo {
    // free cluster 的数量
    pub fn set_free_count(&mut self, new_free_count: usize) {
        self.FSI_Free_Count = new_free_count as u32;
    }

    // 下一个空闲的cluster number
    pub fn next_free(&mut self, new_free: usize) {
        self.FSI_Nxt_Free = new_free as u32;
    }
}
