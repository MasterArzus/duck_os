use alloc::{collections::BTreeMap, sync::{Arc, Weak}};
use spin::Mutex;

use crate::{config::mm::PAGE_SIZE_BITS, mm::{pma::Page, type_cast::PagePermission}};

use super::inode::Inode;

pub struct PageCache {
    // (file_offset, page)
    pub pages: Mutex<BTreeMap<usize, Arc<Page>>>,
}

impl PageCache {
    pub fn new() -> Self {
        Self { pages: Mutex::new(BTreeMap::new()) }
    }

    fn to_offset(file_offset: usize) -> usize {
        file_offset << PAGE_SIZE_BITS
    }
    
    pub fn find_page(&self, file_offset: usize, inode: Weak<dyn Inode>) -> Arc<Page> {
        let page = match self.pages.lock().get(&Self::to_offset(file_offset)) {
            Some(page) => Arc::clone(page),
            None => Self::find_page_from_disk(&self, file_offset, inode),
        };
        page
    }

    fn find_page_from_disk(&self, offset: usize, inode: Weak<dyn Inode>) -> Arc<Page> {
        let page = Page::new_disk_page(PagePermission::R, inode, offset);
        let page_arc = Arc::new(page);
        self.pages.lock().insert(Self::to_offset(offset), page_arc.clone());
        page_arc
    }
}