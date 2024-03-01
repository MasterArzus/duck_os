//! 物理地址空间模块

/*
    1. pma 数据结构
        1）page_manager (使用BTree来管理)
        2）back_file (mmap中有关的数据结构)
    2. page 数据结构
        1）frames（页的物理内存）
        2) permission (页的访问权限)
        3) file_info (用在page cache中的数据结构)
    3. pma的区间伸缩问题
        这里统一交给frames的物理页号去处理，参考maturin
*/