use core::sync::atomic::{AtomicU64, Ordering};

use linked_list_allocator::LockedHeap;
use litchi_user_common::heap::USER_HEAP_BASE_ADDR;
use x86_64::structures::paging::{PageSize, Size4KiB};
use x86_64::VirtAddr;

use crate::syscall::sys_extend_heap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

static HEAP_TOP: AtomicU64 = AtomicU64::new(USER_HEAP_BASE_ADDR.as_u64());

fn extend_additional(size: usize) {
    let old_top = HEAP_TOP.fetch_add(size as u64, Ordering::SeqCst);
    sys_extend_heap(VirtAddr::new(old_top) + size);
}

pub(crate) fn init() {
    const HEAP_PAGES: usize = 2048; // 8 MiB
    const HEAP_SIZE: usize = HEAP_PAGES * (Size4KiB::SIZE as usize);

    extend_additional(HEAP_SIZE);

    unsafe {
        ALLOCATOR
            .lock()
            .init(USER_HEAP_BASE_ADDR.as_u64() as usize, HEAP_SIZE);
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("alloc error: {:?}", layout);
}
