//！ 加载模块 —— 目前只实现静态加载

use alloc::{string::String, sync::Arc, vec::Vec};

use crate::{config::mm::{USER_STACK_BOTTOM, USER_STACK_SIZE}, mm::{address::vaddr_offset, memory_set::{mem_set::MemeorySet, page_fault::UStackPageFaultHandler}, type_cast::MapPermission, vma::{MapType, VirtMemoryAddr, VmaType}}};
use self::stack::{StackInfo, StackLayout};

pub mod dynamic;
pub mod stack;

pub fn check_magic(elf: &xmas_elf::ElfFile) -> bool {
    let mut ans: bool = true;
    let magic_num:[u8; 4] = [0x7f, 0x45, 0x4c, 0x46];
    for i in 0..magic_num.len() {
       if magic_num[i] != elf.header.pt1.magic[i] {
            ans = false;
       }
    }
    ans
}

// 返回值：(entry_point, ustack_sp, StackLayout)
pub fn load_elf(data: &[u8], vm: &mut MemeorySet, args: Vec<String>, envs: Vec<String>) -> (usize, usize, Option<StackLayout>) {
    let elf = xmas_elf::ElfFile::new(&data).unwrap();
    // 检查魔数
    if !check_magic(&elf) {
        panic!()
    }
    // 开始映射
    let ph_count = elf.header.pt2.ph_count();
    for i in 0..ph_count {
        let ph = elf.program_header(i).unwrap();
        if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
            let start_va = ph.virtual_addr() as usize;
            let end_va = start_va + ph.mem_size() as usize;
            let mut map_permission = MapPermission::U;
            let ph_flags = ph.flags();
            if ph_flags.is_read() {
                map_permission |= MapPermission::R;
            }
            if ph_flags.is_write() {
                map_permission |= MapPermission::W;
            }
            if ph_flags.is_execute() {
                map_permission |= MapPermission::X;
            }
            // TODO：优化的地方：首先可以对elf的内容进行lazy复制
            // 其次如果elf的内容已经在内存中，且是不可写，则可以共享page
            vm.push(VirtMemoryAddr::new(
                start_va,
                end_va, 
                map_permission, 
                MapType::Framed,
                VmaType::Elf,
                None,),
                Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                vaddr_offset(start_va)
            );
        }
    }
    // 映射用户栈
    let user_stack_bottom = USER_STACK_BOTTOM;
    let user_stack_top = user_stack_bottom - USER_STACK_SIZE;
    vm.push_no_map(VirtMemoryAddr::new(
        user_stack_top, 
        user_stack_bottom, 
        MapPermission::U | MapPermission::R | MapPermission::W, 
        MapType::Framed, 
        VmaType::UserStack,
        Some(Arc::new(UStackPageFaultHandler {}))
    ));

    let mut stack_layout: Option<StackLayout> = None;
    // 需要构建user stack中的内容
    if !args.is_empty() && envs.is_empty() {
        // 传递auxv的相关值
        let mut stack_info = StackInfo::empty();
        stack_info.init_arg(args, envs);
        stack_info.init_auxv(&elf);

        let (sp, layout) = stack_info.build_stack(user_stack_bottom - 1);
        stack_layout = Some(layout);
    }
    (0, 0, stack_layout)
    
}