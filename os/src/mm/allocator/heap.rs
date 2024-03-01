/*！
    堆分配器，使用buddy_system中的Locked_Heap.
    TODO： 暂时先这样，之后可能要更改锁的类型
*/

use buddy_system_allocator::LockedHeap;
use crate::config::mm::KERNEL_HEAP_SIZE;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::new();

// Initial the heap
pub fn init_heap() {
    // warning: 这里其实可以使用usize,但是要考虑不同机器上的usize大小
    static mut HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}
