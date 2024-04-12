use alloc::sync::Arc;
use lazy_static::lazy_static;

use self::pcb::PCB;

pub mod context;
pub mod hart;
pub mod pcb;
pub mod pid;
pub mod switch;
pub mod trap;
pub mod schedule;
pub mod kstack;
pub mod loader;

lazy_static! {
    pub static ref ORIGIN_TASK: Arc<PCB> = Arc::new(
        PCB::elf_data_to_pcb("file_name", &[0])
    );
}