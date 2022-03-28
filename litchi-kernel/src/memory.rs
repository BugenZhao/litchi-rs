use log::info;
use spin::Mutex;
use x86_64::{
    instructions, registers,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame,
    },
};

use crate::{
    frame_allocator::{BootInfoFrameAllocator, FRAME_ALLOCATOR},
    BOOT_INFO,
};

lazy_static::lazy_static! {
    pub static ref PAGE_TABLE: Mutex<OffsetPageTable<'static>> = Mutex::new(new_page_table());
}

fn new_page_table() -> OffsetPageTable<'static> {
    let boot_info = BOOT_INFO.get().expect("boot info not set");

    let frame = registers::control::Cr3::read().0;
    let l4_table = frame.start_address().as_u64() as *mut PageTable;
    unsafe {
        let l4_table = l4_table.as_mut().unwrap();
        OffsetPageTable::new(l4_table, boot_info.phys_offset)
    }
}

pub fn init() {
    lazy_static::initialize(&PAGE_TABLE);

    info!("prepared page table")
}

pub fn with_allocator_and_page_table<F, R>(f: F) -> R
where
    F: FnOnce(&mut BootInfoFrameAllocator, &mut OffsetPageTable) -> R,
{
    instructions::interrupts::without_interrupts(|| {
        let mut frame_allocator = FRAME_ALLOCATOR
            .get()
            .expect("frame allocator not initialized")
            .lock();
        let mut page_table = PAGE_TABLE.lock();

        f(&mut *frame_allocator, &mut *page_table)
    })
}

pub unsafe fn map_to(page: Page, frame: PhysFrame, flags: PageTableFlags) {
    with_allocator_and_page_table(|frame_allocator, page_table| {
        page_table
            .map_to(page, frame, flags, &mut *frame_allocator)
            .expect("failed to map frame")
            .flush();
    })
}

pub unsafe fn allocate_and_map_to(page: Page, flags: PageTableFlags) -> PhysFrame {
    with_allocator_and_page_table(|frame_allocator, page_table| {
        let frame = frame_allocator.allocate_frame().expect("no enough memory");

        page_table
            .map_to(page, frame, flags, &mut *frame_allocator)
            .expect("failed to map frame")
            .flush();

        frame
    })
}
