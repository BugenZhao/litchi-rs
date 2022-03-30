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
        mapper::TranslateResult, FrameAllocator, Mapper, OffsetPageTable, Page, PageSize,
        PageTable, PageTableFlags, PhysFrame, Size4KiB, Translate,
    },
    VirtAddr,
};

use crate::{frame_allocator::RaiiFrameAllocator, BOOT_INFO};

pub struct PageTableWrapper {
    frame: PhysFrame,

    inner: Mutex<OffsetPageTable<'static>>,

    allocator: Mutex<RaiiFrameAllocator>,
}

impl core::fmt::Debug for PageTableWrapper {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "PageTable at {:?}",
            self.frame.start_address()
        ))
    }
}

impl PageTableWrapper {
    fn new(frame: PhysFrame, allocator: RaiiFrameAllocator) -> Self {
        let boot_info = BOOT_INFO.get().expect("boot info not set");

        let l4_table = frame.start_address().as_u64() as *mut PageTable;
        let inner = unsafe {
            let l4_table = l4_table.as_mut().unwrap();
            OffsetPageTable::new(l4_table, boot_info.phys_offset)
        };

        Self {
            frame,
            inner: Mutex::new(inner),
            allocator: Mutex::new(allocator),
        }
    }

    fn kernel() -> Self {
        let current_frame = registers::control::Cr3::read().0;

        Self::new(current_frame, RaiiFrameAllocator::new_untraced())
    }

    pub fn new_user() -> Self {
        let mut allocator = RaiiFrameAllocator::new_traced();

        let frame = allocator
            .allocate_frame()
            .expect("failed to allocate frame for new page table");

        // Copy mapping for kernel.
        // TODO: This requires memory space used for kernel should not overlap with users.
        unsafe {
            copy_nonoverlapping(
                KERNEL_PAGE_TABLE.frame.start_address().as_u64() as *const u8,
                frame.start_address().as_u64() as *mut _,
                frame.size() as usize,
            );
        }

        Self::new(frame, allocator)
    }

    pub fn load(&self) {
        unsafe {
            Cr3::write(self.frame, Cr3Flags::empty());
        }
    }

    pub fn is_current(&self) -> bool {
        Cr3::read().0 == self.frame
    }

    pub fn with_allocator<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut RaiiFrameAllocator, &mut OffsetPageTable<'static>) -> R,
    {
        instructions::interrupts::without_interrupts(|| {
            let mut allocator = self.allocator.lock();
            let mut page_table = self.inner.lock();

            f(&mut *allocator, &mut *page_table)
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

    pub unsafe fn allocate_and_map_to(
        &self,
        page: Page,
        flags: PageTableFlags,
    ) -> Option<PhysFrame> {
        self.with_allocator(|frame_allocator, page_table| {
            let frame = frame_allocator.allocate_frame()?;

            page_table
                .map_to(page, frame, flags, &mut *frame_allocator)
                .expect("failed to map frame")
                .flush();

            Some(frame)
        })
    }

    pub fn check_user_accessible(&self, base: *const (), len: usize) -> bool {
        if len == 0 {
            return true;
        }

        self.with_allocator(|_, page_table| {
            let base = VirtAddr::from_ptr(base);
            let base_page = Page::<Size4KiB>::containing_address(base);
            let end_page = Page::containing_address(base + (len - 1));

            for page in Page::range_inclusive(base_page, end_page) {
                let check_addr = page.start_address();

                match page_table.translate(check_addr) {
                    TranslateResult::Mapped { flags, .. }
                        if flags.contains(PageTableFlags::USER_ACCESSIBLE) => {}

                    TranslateResult::Mapped { .. }
                    | TranslateResult::NotMapped
                    | TranslateResult::InvalidFrameAddress(_) => return false,
                }
            }

            true
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
