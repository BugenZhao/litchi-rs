use core::{fmt::Debug, intrinsics::copy_nonoverlapping};

use log::info;
use spin::Mutex;
use x86_64::{
    instructions,
    registers::{
        self,
        control::{Cr3, Cr3Flags},
    },
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageSize, PageTable, PageTableFlags,
        PhysFrame,
    },
};

use crate::{
    frame_allocator::{BootInfoFrameAllocator, FRAME_ALLOCATOR},
    BOOT_INFO,
};

pub struct PageTableWrapper {
    frame: PhysFrame,

    inner: Mutex<OffsetPageTable<'static>>,
}

impl PageTableWrapper {
    fn from_frame(frame: PhysFrame) -> Self {
        let boot_info = BOOT_INFO.get().expect("boot info not set");

        let l4_table = frame.start_address().as_u64() as *mut PageTable;
        let inner = unsafe {
            let l4_table = l4_table.as_mut().unwrap();
            OffsetPageTable::new(l4_table, boot_info.phys_offset)
        };

        Self {
            frame,
            inner: Mutex::new(inner),
        }
    }

    fn kernel() -> Self {
        let current_frame = registers::control::Cr3::read().0;

        Self::from_frame(current_frame)
    }

    pub fn new_user() -> Self {
        let frame = instructions::interrupts::without_interrupts(|| {
            FRAME_ALLOCATOR
                .get()
                .expect("frame allocator not initialized")
                .lock()
                .allocate_frame()
                .expect("failed to allocate frame for new page table")
        });

        // Copy mapping for kernel.
        // TODO: This requires memory space used for kernel should not overlap with users.
        unsafe {
            copy_nonoverlapping(
                KERNEL_PAGE_TABLE.frame.start_address().as_u64() as *const u8,
                frame.start_address().as_u64() as *mut _,
                frame.size() as usize,
            );
        }

        Self::from_frame(frame)
    }

    pub fn load(&self) {
        unsafe {
            Cr3::write(self.frame, Cr3Flags::empty());
        }
    }

    pub fn with_allocator<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut BootInfoFrameAllocator, &mut OffsetPageTable<'static>) -> R,
    {
        instructions::interrupts::without_interrupts(|| {
            let mut frame_allocator = FRAME_ALLOCATOR
                .get()
                .expect("frame allocator not initialized")
                .lock();
            let mut page_table = self.inner.lock();

            f(&mut *frame_allocator, &mut *page_table)
        })
    }

    pub unsafe fn map_to<S: PageSize + Debug>(
        &self,
        page: Page<S>,
        frame: PhysFrame<S>,
        flags: PageTableFlags,
    ) where
        OffsetPageTable<'static>: Mapper<S>,
    {
        self.with_allocator(|frame_allocator, page_table| {
            page_table
                .map_to(page, frame, flags, &mut *frame_allocator)
                .expect("failed to map frame")
                .flush();
        })
    }

    pub unsafe fn allocate_and_map_to(&self, page: Page, flags: PageTableFlags) -> PhysFrame {
        self.with_allocator(|frame_allocator, page_table| {
            let frame = frame_allocator.allocate_frame().expect("no enough memory");

            page_table
                .map_to(page, frame, flags, &mut *frame_allocator)
                .expect("failed to map frame")
                .flush();

            frame
        })
    }
}

lazy_static::lazy_static! {
    pub static ref KERNEL_PAGE_TABLE: PageTableWrapper = PageTableWrapper::kernel();
}

pub fn init() {
    lazy_static::initialize(&KERNEL_PAGE_TABLE);

    info!("prepared page table")
}
