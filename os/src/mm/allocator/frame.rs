/*！
   页帧模块，通过包装 bitmap_allocator 实现。    
*/

/*
    1. 位图分配器用来分配 bit
    2. 页帧分配器就分配 FrameTracker
    3. 页帧分配器的功能
        1） alloc
        2) dealloc
        3) alloc_continue
    4. 一个全局的页帧分配器
        1）初始化，全新
        2）设置分配的区间
        3）alloc
        4) alloc_continue
        5) dealloc
*/