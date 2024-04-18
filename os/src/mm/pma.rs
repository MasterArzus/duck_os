//! 物理地址空间模块

/*
    1. pma 数据结构
        1）page_manager (使用BTree来管理)
        2）back_file (mmap中有关的数据结构)
    2. page 数据结构
        1）frames（页的物理内存）
        2) permission (页的访问权限)
        3) file_info (用在page cache中的数据结构)
    3. pma的区间伸缩问题
        这里统一交给frames的物理页号去处理，参考maturin
    4. 功能函数
        1）clone_as_fork (这一部分的东西其实还没有看，所以这个暂定)
        2）区间伸缩函数 shrink_left | shrink_right | split
        3）读写函数
        4）同步函数（用在page cache中，暂且不考虑）
*/

use alloc::{collections::BTreeMap, sync::{Arc, Weak}};
use spin::Mutex;

use crate::{config::{fs::SECTOR_SIZE, mm::PAGE_SIZE}, fs::inode::Inode};

use super::{
    address::{align_down, byte_array, get_mut, get_ref, phys_to_ppn, ppn_to_phys, virt_to_vpn},
    allocator::frame::{alloc_frame, FrameTracker}, 
    type_cast::PagePermission,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DataState {
    Init,
    Sync,
    Dirty,
}

const DATA_SIZE: usize = PAGE_SIZE / SECTOR_SIZE;

pub struct DiskFileInfo {
    inode: Weak<dyn Inode>,
    file_offset: usize,
    data_state: [DataState; DATA_SIZE],
}

impl DiskFileInfo {
    pub fn new(inode: Weak<dyn Inode>, f_offset: usize) -> Self {
        Self {
            inode,
            file_offset: f_offset,
            data_state: [DataState::Init; DATA_SIZE],
        }
    }

    pub fn change_data_state(&mut self, state: DataState, idx: usize) {
        self.data_state[idx] = state;
    }
}

pub struct Page {
    pub frame: FrameTracker,
    pub permission: PagePermission,
    pub disk_file: Option<Mutex<DiskFileInfo>>,
}

impl Page {
    pub fn set_permission(&mut self, per: PagePermission) {
        self.permission = per;
    } 

    pub fn new(per: PagePermission) -> Self {
        Self{
            frame: alloc_frame().unwrap(),
            permission: per,
            disk_file: None,
        }
    }

    pub fn new_disk_page(per: PagePermission, inode: Weak<dyn Inode>, offset: usize) -> Self {
        Self {
            frame: alloc_frame().unwrap(),
            permission: per,
            disk_file: Some(Mutex::new(DiskFileInfo::new(inode, offset))),
        }
    }

    pub fn new_from_page(ppn: usize, permission: PagePermission) -> Self {
        let new_frame = alloc_frame().unwrap();
        byte_array(ppn_to_phys(new_frame.ppn))
            .copy_from_slice(&byte_array(ppn_to_phys(ppn)));
        Self {
            frame: new_frame,
            permission,
            disk_file: None,
        }
    }
    
    pub fn get_ref_from_page<T>(&self) -> &'static T {
        get_ref(ppn_to_phys(self.frame.ppn))
    }

    pub fn get_mut_from_page<T>(&self) -> &'static mut T {
        get_mut(ppn_to_phys(self.frame.ppn))
    }

    pub fn page_byte_array(&self) -> &'static mut [u8] {
        byte_array(phys_to_ppn(self.frame.ppn))
    }

    fn to_sec_idx(page_offset: usize) -> usize {
        page_offset / PAGE_SIZE 
    }
    // 一个页面的读写
    // page_offset: 为页面中的offset值
    pub fn read(&self, page_offset: usize, buf: &mut [u8]) {
        if page_offset > PAGE_SIZE {
            panic!()
        }
        let len: usize = buf.len().min(PAGE_SIZE - page_offset);
        for idx in Self::to_sec_idx(page_offset)..DATA_SIZE {
            let mut disk_file_lock = self.disk_file.as_ref().unwrap().lock();
            if disk_file_lock.data_state[idx] == DataState::Init {
                disk_file_lock.inode.upgrade().unwrap().read(
                    disk_file_lock.file_offset + idx * SECTOR_SIZE,
                    &mut self.page_byte_array()[idx*SECTOR_SIZE..(idx+1)*SECTOR_SIZE],
                );
                disk_file_lock.change_data_state(DataState::Sync, idx);
            }
            drop(disk_file_lock);
        }
        buf.copy_from_slice(&self.page_byte_array()[page_offset..page_offset+len]);
    }

    pub fn write(&self, page_offset: usize, buf: &[u8]) {
        if page_offset > PAGE_SIZE {
            panic!()
        }
        let len: usize = buf.len().min(PAGE_SIZE - page_offset);
        let start = page_offset;
        let end = page_offset + len;
        for idx in Self::to_sec_idx(page_offset)..DATA_SIZE {
            let mut disk_file_lock = self.disk_file.as_ref().unwrap().lock();
            if disk_file_lock.data_state[idx] == DataState::Init {
                disk_file_lock.inode.upgrade().unwrap().read(
                    disk_file_lock.file_offset + idx * SECTOR_SIZE,
                    &mut self.page_byte_array()[idx*SECTOR_SIZE..(idx+1)*SECTOR_SIZE],
                );
                disk_file_lock.change_data_state(DataState::Dirty, idx);
            }
        }
        // TODO: 小心copy_from_slice这个函数，不会检查两个切片的大小，我这里没有检查，区间可能会爆掉！
        self.page_byte_array()[start..end].copy_from_slice(buf);
    }

    pub fn sync() {

    }
}

pub struct PhysMemoryAddr {
    // (key: vpn, value: page)
    // 这个设计目前看起来有点鸡肋，只是做一做增删查改的工作
    // 对于page中的permission基本上没有什么改动
    pub page_manager: BTreeMap<usize, Arc<Page>>,
    pub backen_file: Option<usize>,
}

impl PhysMemoryAddr {
    pub fn new() -> Self {
        Self {
            page_manager: BTreeMap::new(),
            backen_file: None,
        }
    }

    // 处理page_manager增删的函数
    // TODO: 很多边界情况没有考虑

    // 删除一个页面
    pub fn pop_pma_page(&mut self, vpn: usize) {
        if !self.page_manager.contains_key(&vpn) {
            todo!()
        }
        self.page_manager.remove(&vpn);
    }

    // 增加一个页面
    pub fn push_pma_page(&mut self, vpn: usize, page: Arc<Page>) {
        self.page_manager.insert(vpn, page);
    }
    

    // 处理区间伸缩问题的相关函数
    // 就是将相关区间的frame释放，同时将None从中移除
    pub fn shrink_left(&mut self, new_start: usize, old_start: usize) {
        // 不用检查区间的情况，这些都交给上一层 vma中处理
        // TODO： 如何解决判断区间页的问题？我该怎么去除页？会不会有保证一定是页首地址？
        let old_vpn = virt_to_vpn(old_start);
        let new_vpn = virt_to_vpn(new_start);
        for vpn in old_vpn..=new_vpn {
            self.pop_pma_page(vpn);
        }
    }

    pub fn shrink_right(&mut self, new_end: usize, old_end: usize) {
        let old_vpn = virt_to_vpn(old_end);
        let new_vpn = virt_to_vpn(new_end);
        for vpn in new_vpn..=old_vpn {
            self.pop_pma_page(vpn);
        }
    }

    // 返回右边剩下的, 删除中间的，留下左边的。
    // 这里的绝对地址也需要再次修改！
    pub fn split(&mut self, left_end: usize, right_start: usize, _start: usize, end: usize) -> Self {
        let mut right_page_manager:BTreeMap<usize, Arc<Page>> = BTreeMap::new();
        for vpn in virt_to_vpn(right_start)..virt_to_vpn(end) {
            right_page_manager.insert(vpn, self.page_manager.remove(&vpn).unwrap()); 
        }
        for vpn in virt_to_vpn(left_end)..virt_to_vpn(right_start) {
            self.pop_pma_page(vpn);
        }
        Self {
            page_manager: right_page_manager,
            backen_file: None,
        }
    }

    // 得到其中的页面 TODO: 仍然需要修改，直接返回page有点不好，返回地址比较好
    pub fn get_pma_page_ppn(&mut self, vpn: usize) -> Option<usize> {
        Some(self.page_manager.get(&vpn).unwrap().frame.ppn)
    }

    // 读写相关的函数
    // 相关函数的接口形式可以参考maturin
    pub fn read_write_pma_page(
        &mut self, 
        offset: usize,
        len: usize,
        mut op: impl FnMut(usize, &mut [u8])
    ) -> usize {
        let mut start = offset;
        let mut len = len;
        let mut finished = 0usize;
        while len > 0 {
            let start_align = align_down(start);
            let page_offset = start - start_align;
            let n = (PAGE_SIZE - page_offset).min(len);
            let vpn = virt_to_vpn(start_align);
            
            let page = self.page_manager.get_mut(&vpn).unwrap();
            op(finished, &mut self.page_manager.get(&vpn).unwrap().page_byte_array()[page_offset..page_offset+n]);
            start += n;
            len -= n;
            finished += n;
        }
        finished
    }

    pub fn read_pma_page(&mut self, offset: usize, dst:&mut [u8]) -> usize {
        self.read_write_pma_page(offset, dst.len(), |finished: usize, src: &mut [u8]|{
            dst[finished..finished + src.len()].copy_from_slice(src)
        })
    }

    pub fn write_pmd_page(&mut self, offset: usize, src: &mut [u8]) -> usize {
        self.read_write_pma_page(offset, src.len(), |finished: usize, dst: &mut [u8]| {
            dst.copy_from_slice(&src[finished..finished + dst.len()]);
        })
    }
    // fork相关的函数


    // 同步的相关函数(暂时不用考虑)
}