//! memory_set模块

/*
    1. 懒分配
        1）maturin: 在插入虚拟逻辑段的时候，使用push_lazy函数，插入时分配None的Frame
                    同时写入物理地址为0的pte。之后在page_fault中，使用get_frame来得到frame,
                    如果没有分配，则进行分配。
        2）Tiantix: 同样在插入虚拟逻辑段的时候，使用push_lazy函数，插入的时候直接不做映射。在发生
                    page_fault时，让对应的page_fault进行分配，并做好映射。
    TODO： 这里还有很多的细节不懂的，还是需要花时间去看代码！！
    2. 数据结构
        1) page_table
        2) areas 不同vma的机和，使用BTreeMap管理
        3）heap_range（与brk系统调用有关，可以用上那个区间管理的东西）
    3. 函数功能
        1）new 和 new_from_other(fork有关)
        2）token
        3）通过vpn找vm_area
        4) 插入vma，主要就是两个push函数
        5）读写（其实就是使用下层的
        6）page_fault
        7）克隆地址空间
        还有一些莫名奇妙的函数，反正先不管吧，那些函数也是需要去看大量代码才能理解的，先实现一些通用的功能。
*/