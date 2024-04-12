//! 处理当前核中的信息

use alloc::{sync::Arc, vec::Vec};

use crate::{config::task::MAX_CORE_NUM, mm::memory_set::mem_set::kernel_space_activate, process::{context::TaskContext, pcb::{TaskStatus, PCB}, schedule::{pop_task_from_schedule, push_task_to_schedule}, switch::__switch, ORIGIN_TASK}};

use super::env::Env;

pub struct CpuLocal {
    // pub id: usize,
    pub current: Option<Arc<PCB>>, 
    pub env: Env,
    pub idle_cx: TaskContext,
}

impl CpuLocal {
    pub const fn empty() -> Self {
        Self {
            current: None,
            env: Env::empty(),
            idle_cx: TaskContext::empty(),
        }
    }

    pub fn idle_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_cx as *mut _
    }

    pub fn current_pcb(&self) -> Option<Arc<PCB>> {
        self.current.as_ref().map(Arc::clone)
    }

    pub fn take_current_pcb(&mut self) -> Option<Arc<PCB>> {
        self.current.take()
    }
}

pub fn run_task() {
    let cpu_id = get_cpu_id();
    loop {
        if let Some(task) = pop_task_from_schedule() {
            let cpu_local = get_mut_cpu_local(cpu_id);
            let idle_task_cx_ptr = cpu_local.idle_cx_ptr();
            let next_task_cx_ptr = task.task_cx_ptr();
            task.set_status(TaskStatus::Running);
            task.vm.lock().activate(); // 切换用户进程地址空间
            cpu_local.current = Some(task);
            unsafe {
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
            kernel_space_activate(); // 切换回内核地址空间
            // if let Some(task) = CPULOCALS[cpu_id].lock().take_current_pcb() {
            //     match task.status() {
            //         TaskStatus::Ready => {
            //             push_task_to_schedule(task);
            //         },
            //         TaskStatus::Dead => {

            //         }
            //         TaskStatus::Interruptible => {

            //         },  
            //         _ => {

            //         }
            //     }
            // }
        }
    }
}

// 暂停当前的任务，但是不移出来
// TODO: 其实可以移出来
pub fn suspend_current_task() {
    let cpu_id = get_cpu_id();
    let cpu_local = get_mut_cpu_local(cpu_id);
    if let Some(task) = cpu_local.take_current_pcb() {
        task.set_status(TaskStatus::Ready);
        let current_task_cx_ptr = task.task_cx_ptr() as *mut TaskContext;
        let idle_task_cx_ptr = cpu_local.idle_cx_ptr();
        push_task_to_schedule(task);
        unsafe {
            __switch(current_task_cx_ptr, idle_task_cx_ptr);
        }
    }
}

pub fn exit_current_task(exit_code: i32) {
    let cpu_id = get_cpu_id();
    let cpu_local = get_mut_cpu_local(cpu_id);
    let task = cpu_local.current_pcb().unwrap();
    task.set_status(TaskStatus::Dead);
    task.set_exit_code(exit_code);

    let task_inner = task.inner.lock();
    // 这里上锁要注意，先尝试拿儿子的锁，如果拿不到，就不去拿INIT_PROC的锁,否则可能会死锁
    for child in &task_inner.child {
        loop {
            if let Some(mut child_inner) = child.inner.try_lock() {
                if let Some(mut origin_task_inner) = ORIGIN_TASK.clone().inner.try_lock() {
                    child_inner.parent = Some(Arc::downgrade(&ORIGIN_TASK));
                    child_inner.ppid = 0;
                    origin_task_inner.child.push(child.clone());
                }
            }
        }
    }
    task.clear_child();
    // 剩下的步骤有：释放文件描述符、内存中的数据、发送信号
    // 这里是利用了rust语言的特性，当在wait函数中，task会被drop，且Arc的计数为1
    // 所有的资源都会被自动释放，不需要手动去释放文件描述符和内存中的数据。但是也可以自己手动释放。
}

#[inline]
pub fn get_cpu_id() -> usize {
    let mut cpu_id: usize;
    unsafe {
        core::arch::asm!("mv {0}, tp", out(reg) cpu_id);
    }
    cpu_id
}

// 每个核都只能访问对应的cpu_local，所以不会出现数据竞争的情况。不需要加锁！
pub static mut CPULOCALS: Vec<CpuLocal> = Vec::new();

pub fn init_cpu_locals() {
    for _ in 0..MAX_CORE_NUM {
        unsafe { CPULOCALS.push(CpuLocal::empty()); }
    }
}

pub fn get_mut_cpu_local(cpu_id: usize) -> &'static mut CpuLocal{
    unsafe {
        &mut CPULOCALS[cpu_id]
    }
}
