use core::intrinsics::copy_nonoverlapping;

use itertools::{EitherOrBoth, Itertools};
use log::info;
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageSize, PageTableFlags, PhysFrame,
        Size4KiB, Translate,
    },
    VirtAddr,
};
use xmas_elf::{header, program, ElfFile};

pub type EntryPoint = *const extern "C" fn() -> !;

#[derive(Debug, Clone)]
pub struct LoaderConfig {
    pub stack_top: VirtAddr,

    pub stack_pages: u64,

    pub userspace: bool,
}

pub struct ElfLoader<'a, A> {
    config: &'a LoaderConfig,

    elf: ElfFile<'static>,

    page_table: &'a mut OffsetPageTable<'static>,

    allocator: &'a mut A,
}

impl<'a, A> ElfLoader<'a, A>
where
    A: FrameAllocator<Size4KiB>,
{
    pub fn new(
        config: &'a LoaderConfig,
        input: &'static [u8],
        allocator: &'a mut A,
        page_table: &'a mut OffsetPageTable<'static>,
    ) -> Self {
        let elf = ElfFile::new(input).expect("failed to parse elf");
        Self::check_dynamic(&elf);

        Self {
            config,
            elf,
            page_table,
            allocator,
        }
    }

    fn check_dynamic(elf: &ElfFile) {
        if elf.header.pt2.type_().as_type() == header::Type::SharedObject {
            unimplemented!("loading a shared object / pie executable is not supported");
        }
    }

    pub fn load(self) -> EntryPoint {
        // TODO: This requires the target page table can access the elf input.
        let file_base = self
            .page_table
            .translate_addr(VirtAddr::from_ptr(self.elf.input.as_ptr()))
            .expect("failed to translate file base");
        assert!(
            file_base.is_aligned(Size4KiB::SIZE),
            "this elf is not 4K aligned"
        );

        // TODO: use correct flags
        let flags = {
            let mut base = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
            if self.config.userspace {
                base |= PageTableFlags::USER_ACCESSIBLE;
            }
            base
        };

        for segment in self
            .elf
            .program_iter()
            .filter(|p| p.get_type().expect("bad type") == program::Type::Load && p.mem_size() > 0)
        {
            info!("begin to map segment {:x?}", segment);

            let file_start = file_base + segment.offset();
            let file_end = file_start + segment.file_size();

            let mem_start = VirtAddr::new(segment.virtual_addr());
            let mem_end = mem_start + segment.mem_size();

            let start_page = Page::<Size4KiB>::containing_address(mem_start);
            let end_page = Page::containing_address(mem_end - 1u64);

            let start_frame = PhysFrame::containing_address(file_start);
            let end_frame = PhysFrame::containing_address(file_end - 1u64);

            if segment.mem_size() > segment.file_size() {
                for pair in Page::range_inclusive(start_page, end_page)
                    .zip_longest(PhysFrame::range_inclusive(start_frame, end_frame))
                {
                    let (page, new_frame) = match pair {
                        EitherOrBoth::Both(page, frame) => unsafe {
                            let new_frame = allocate_zeroed_frame(self.allocator);
                            let size_in_frame =
                                core::cmp::min(frame.size(), file_end - frame.start_address())
                                    as usize;
                            copy_nonoverlapping(
                                frame.start_address().as_u64() as *const u8,
                                new_frame.start_address().as_u64() as *mut u8,
                                size_in_frame,
                            );
                            (page, new_frame)
                        },
                        EitherOrBoth::Left(page) => {
                            let new_frame = allocate_zeroed_frame(self.allocator);
                            (page, new_frame)
                        }
                        EitherOrBoth::Right(_frame) => unreachable!(),
                    };

                    unsafe {
                        self.page_table
                            .map_to(page, new_frame, flags, self.allocator)
                            .expect("failed to map page")
                            .flush();

                        info!("mapped bss {:?} to {:?}", page, new_frame);
                    }
                }
            } else {
                for pair in Page::range_inclusive(start_page, end_page)
                    .zip_longest(PhysFrame::range_inclusive(start_frame, end_frame))
                {
                    let (page, frame) = match pair {
                        EitherOrBoth::Both(page, frame) => (page, frame),
                        EitherOrBoth::Left(_page) => panic!("frame not enough"),
                        EitherOrBoth::Right(_frame) => break,
                    };

                    unsafe {
                        self.page_table
                            .map_to(page, frame, flags, self.allocator)
                            .expect("failed to map page")
                            .flush();

                        info!("mapped {:?} to {:?}", page, frame);
                    }
                }
            }

            info!("mapped this segment")
        }

        let entry_point = self.elf.header.pt2.entry_point();
        info!("entry point at 0x{:x}", self.elf.header.pt2.entry_point());

        let stack_page = Page::containing_address(self.config.stack_top);
        for i in 0..=self.config.stack_pages {
            let page = stack_page - i;
            let frame = allocate_zeroed_frame(self.allocator);

            let stack_flags = if i == self.config.stack_pages {
                // Make the bottom page unwritable.
                flags - PageTableFlags::WRITABLE
            } else {
                flags
            };

            unsafe {
                self.page_table
                    .map_to(page, frame, stack_flags, self.allocator)
                    .expect("failed to map page")
                    .flush();

                info!("mapped {:?} to {:?}", page, frame);
            }
        }

        entry_point as EntryPoint
    }
}

pub fn allocate_zeroed_frame(allocator: &mut impl FrameAllocator<Size4KiB>) -> PhysFrame<Size4KiB> {
    let frame = allocator
        .allocate_frame()
        .expect("failed to allocate frame");
    let ptr = frame.start_address().as_u64() as *mut u8;
    unsafe {
        core::ptr::write_bytes(ptr, 0, Size4KiB::SIZE as usize);
    }
    frame
}
