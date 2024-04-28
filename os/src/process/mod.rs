use alloc::sync::Arc;

use crate::{fs::{dentry::path_to_dentry, fd_table::init_fd_allocator}, process::{hart::cpu::init_cpu_locals, schedule::init_schedule}};

use self::{pcb::PCB, pid::init_pid_allocator};

pub mod context;
pub mod hart;
pub mod pcb;
pub mod pid;
pub mod switch;
pub mod trap;
pub mod schedule;
pub mod kstack;
pub mod loader;

// lazy_static! {
//     pub static ref ORIGIN_TASK: Arc<PCB> = Arc::new(
//         PCB::elf_data_to_pcb("file_name", &[0])
//     );
// }

pub static mut ORIGIN_TASK: Option<Arc<PCB>> = None;
// pub static ORIGIN_TASK: Option<Arc<SpinLock<PCB>>> = None;

pub fn init_origin_task() {
    // 先拿到elf的data数据
    let path = "/hellostd";
    let dentry = path_to_dentry(path);
    if dentry.is_none() {
        panic!("No file:{} in file system.", path);
    }
    let inode = Arc::clone(&dentry.as_ref().unwrap().metadata().inner.lock().d_inode);
    let data = inode.read_all();
    // 然后再根据这些数据构造pcb
    init_pid_allocator();
    init_fd_allocator();
    init_cpu_locals();
    unsafe {
        ORIGIN_TASK = Some(Arc::new(PCB::elf_data_to_pcb(path, &data)));
    }
    println!("Origin task initialization finished!");
    init_schedule();
}