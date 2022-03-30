use linked_list_allocator::LockedHeap;
use litchi_user_common::heap::USER_HEAP_BASE_ADDR;
use x86_64::structures::paging::{PageSize, Size4KiB};

use crate::syscall::sys_extend_heap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub(crate) fn init() {
    const HEAP_PAGES: usize = 2048; // 8 MiB
    const HEAP_SIZE: usize = HEAP_PAGES * (Size4KiB::SIZE as usize);

    sys_extend_heap(USER_HEAP_BASE_ADDR + HEAP_SIZE);

    unsafe {
        ALLOCATOR
            .lock()
            .init(USER_HEAP_BASE_ADDR.as_u64() as usize, HEAP_SIZE);
    }
}
