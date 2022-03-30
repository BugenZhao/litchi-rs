use alloc::vec::Vec;
use core::{
    any::type_name_of_val,
    sync::atomic::{AtomicBool, Ordering},
};
use linked_list_allocator::LockedHeap;
use log::info;
use size_format::SizeFormatterBinary;
use x86_64::{
    structures::paging::{Page, PageSize, PageTableFlags, Size4KiB},
    VirtAddr,
};

use crate::memory::KERNEL_PAGE_TABLE;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

static INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn init() {
    const HEAP_BASE: VirtAddr = VirtAddr::new_truncate(0x4444_0000_0000);
    const HEAP_PAGES: usize = 8192; // 32 MiB
    const HEAP_SIZE: usize = HEAP_PAGES * (Size4KiB::SIZE as usize);

    let heap_base_page = Page::from_start_address(HEAP_BASE).unwrap();
    for i in 0..HEAP_PAGES {
        let page = heap_base_page + i as u64;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        unsafe {
            KERNEL_PAGE_TABLE.allocate_and_map_to(page, flags).unwrap();
        }
    }

    info!(
        "allocated heap at {:?} of {}B",
        HEAP_BASE.as_ptr::<()>(),
        SizeFormatterBinary::new(HEAP_SIZE as u64)
    );

    unsafe {
        ALLOCATOR
            .lock()
            .init(HEAP_BASE.as_u64() as usize, HEAP_SIZE);
    }
    INITIALIZED.store(true, Ordering::SeqCst);

    let test_vec = (0u16..).take(4096).collect::<Vec<_>>();
    assert!(test_vec.as_ptr() >= HEAP_BASE.as_ptr());
    for (i, num) in test_vec.into_iter().enumerate() {
        assert_eq!(i as u16, num);
    }

    info!(
        "allocator of `{}` initialized",
        type_name_of_val(&ALLOCATOR)
    );
}

pub fn initialized() -> bool {
    INITIALIZED.load(Ordering::SeqCst)
}
