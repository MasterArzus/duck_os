//! 进程信息控制块

use alloc::{sync::Weak, string::{String, ToString}, sync::Arc, vec::Vec};
use spin::mutex::Mutex;

use crate::{fs::fd_table::FdTable, mm::memory_set::mem_set::MemeorySet};

use super::{context::TaskContext, kstack::Kstack, loader::load_elf, pid::{alloc_pid, Pid}, trap::context::TrapContext};

pub struct PCB {
    // 进程相关
    pub tgid: usize, // 组标识符，外部可见的pid
    pub pid: Pid, // 唯一标识符，内部可见的pid
    pub kernel_stack: Kstack,
    pub vm: Arc<Mutex<MemeorySet>>,
    pub fd_table: Arc<Mutex<FdTable>>,
    pub inner: Arc<Mutex<PCBInner>>,
}

pub struct PCBInner {
    pub cwd: String,
    pub ppid: usize,
    pub exit_code: i32,
    pub parent: Option<Weak<PCB>>,
    pub child: Vec<Arc<PCB>>,
    pub task_cx: TaskContext,
    pub status: TaskStatus,
}

unsafe impl Send for PCBInner {}

impl PCB {
    pub fn elf_data_to_pcb(file_name: &str, data: &[u8]) -> Self {
        let mut vm = MemeorySet::new_user();
        let (entry_point, user_stack, _) = load_elf(data, &mut vm, Vec::new(), Vec::new());
        let kernel_stack = Kstack::init_kernel_stack();
        let ks_top = kernel_stack.push_trap_cx(TrapContext::init_trap_cx(entry_point, user_stack));
        let pid = alloc_pid().unwrap();
        
        Self {
            tgid: pid.value,
            pid,
            kernel_stack,
            vm: Arc::new(Mutex::new(vm)),
            fd_table: Arc::new(Mutex::new(FdTable::init_fdtable())),
            inner: Arc::new(Mutex::new(PCBInner {
                cwd: file_name.to_string(),
                ppid: 0,
                exit_code: 0,
                parent: None,
                child: Vec::new(),
                task_cx: TaskContext::init_task_cx(ks_top),
                status: TaskStatus::Ready,
            }))
        }
    }

    pub fn from_clone() {

    }

    /* Function: 将当前的进程修改为另一个进程。
       TODO：不完善，缺少很多细节
     */
    pub fn from_exec(&self, data: &[u8], args_vec: Vec<String>, envs_vec: Vec<String>) {
        self.vm.lock().clear_user_space();
        self.fd_table.lock().close_exec();
        let mut vm_lock = self.vm.lock();
        let (entry, user_stack, stack_layout) = load_elf(data, &mut vm_lock, args_vec, envs_vec);
        let kernel_stack = self.kernel_stack.push_trap_cx(
            TrapContext::exec_trap_cx(entry, user_stack, stack_layout.unwrap())
        );
        self.inner.lock().task_cx = TaskContext::init_task_cx(kernel_stack);
    }

    pub fn task_cx_ptr(&self) -> *const TaskContext {
        let inner = self.inner.lock();
        &inner.task_cx
    }

    pub fn set_status(&self, status: TaskStatus) {
        self.inner.lock().status = status;
    }

    pub fn status(&self) -> TaskStatus {
        self.inner.lock().status
    }

    pub fn set_exit_code(&self, exit_code: i32) {
        self.inner.lock().exit_code = exit_code;
    }

    pub fn clear_child(&self) {
        self.inner.lock().child.clear();
    }
}

#[derive(Clone, Copy)]
pub enum TaskStatus {
    Running, // 运行状态
    Ready, // 就绪状态
    Dead, // 还没被父进程回收
    Interruptible, // 等待某些事件发生，会被挂起
    Exit, // 已经被回收了
}