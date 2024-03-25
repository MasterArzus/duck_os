//! 设备驱动模块
//! 


pub trait BlockDevice: Send + Sync {
    fn read_block(&self, block_id: usize, buf: &mut [u8]);
    fn write_block(&self, block_id: usize, buf: &[u8]);
}

pub trait CharDevice: Send + Sync {
    fn getchar(&self) -> u8;
    fn puts(&self, char: &[u8]);
}