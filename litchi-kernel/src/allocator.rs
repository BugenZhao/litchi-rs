use alloc::{
    alloc::{GlobalAlloc, Layout},
    vec::Vec,
};
use core::{any::type_name_of_val, ptr::null_mut};
use linked_list_allocator::LockedHeap;
use log::info;
use size_format::SizeFormatterBinary;
use x86_64::{
    structures::paging::{Page, PageSize, PageTableFlags, Size4KiB},
    VirtAddr,
};

use crate::memory::allocate_and_map_to;

struct Dummy;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should be never called")
    }
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init() {
    const HEAP_BASE: VirtAddr = VirtAddr::new_truncate(0x4444_0000_0000);
    const HEAP_PAGES: usize = 8192; // 32 MiB
    const HEAP_SIZE: usize = HEAP_PAGES * (Size4KiB::SIZE as usize);

    let heap_base_page = Page::from_start_address(HEAP_BASE).unwrap();
    for i in 0..HEAP_PAGES {
        let page = heap_base_page + i as u64;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        unsafe {
            allocate_and_map_to(page, flags);
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
