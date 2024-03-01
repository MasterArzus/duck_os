//! 虚拟地址 逻辑段 一段访问权限相同，位置相邻的地址空间。
//！或者可以看作是多个页 pages

/*
    1. 数据结构 vma
        1) pma, 包括当前的物理空间页（管理器） + 可能的back_file。
            而page是页，有frames + flags + file_info（页cache相关的信息）
        2) vma的类型
           elf、user_stack、mmap、user_heap
        3）page_table(可以不要，从地址空间传下来)
        4）start 和 end（用于区间变化操作）
        5）mmap的port，这个和文件的相同
        6）mmap的flag 种类

    2. 功能
        1）new
        2）from_another (用于fork)
        3）page_fault
        4) map 和 unmap（在创建vma之后，需要映射到物理地址，可以懒分配或者正常分配）
        5）copy_data
            (待定，在Titanix中，这个是用做map_elf的，但是在maturin中，加载elf的部分则单独放在了
            loader模块，所以maturin中没有这个函数。因为我暂时对这个东西不了解，所以先不管它。而且这个函数
            肯定是用在装载文件，例如第一次在内核中加载一个初始化的elf和之后通过sys_exec加载的elf文件)
            maturin装载这一部分的代码我还没有看，所以我不知道如何处理?!?!?!?!?!?
        6）大致没了
*/