use log::info;
use spin::{Mutex, Once};
use x86_64::{
    registers,
    structures::paging::{Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame},
};

use crate::{frame_allocator::FRAME_ALLOCATOR, BOOT_INFO};

pub static PAGE_TABLE: Once<Mutex<OffsetPageTable>> = Once::new();

pub fn init() {
    let boot_info = BOOT_INFO.get().expect("boot info not set");
    PAGE_TABLE.call_once(|| {
        let frame = registers::control::Cr3::read().0;
        let l4_table = frame.start_address().as_u64() as *mut PageTable;
        unsafe {
            let l4_table = l4_table.as_mut().unwrap();
            Mutex::new(OffsetPageTable::new(l4_table, boot_info.phys_offset))
        }
    });

    info!("prepared page table")
}

pub unsafe fn map_to(page: Page, frame: PhysFrame, flags: PageTableFlags) {
    let mut frame_allocator = FRAME_ALLOCATOR
        .get()
        .expect("frame allocator not initialized")
        .lock();
    let mut page_table = PAGE_TABLE.get().expect("page table not initialized").lock();

    page_table
        .map_to(page, frame, flags, &mut *frame_allocator)
        .expect("failed to map heap frame")
        .flush();
}
