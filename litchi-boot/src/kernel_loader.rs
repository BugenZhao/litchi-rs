use core::intrinsics::copy_nonoverlapping;

use itertools::{EitherOrBoth, Itertools};
use log::info;
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageSize, PageTableFlags, PhysFrame,
        Size1GiB, Size4KiB,
    },
    PhysAddr, VirtAddr,
};
use xmas_elf::{header, program, ElfFile};

pub type KernelEntry = *const extern "C" fn() -> !;

pub struct KernelLoader<'a, A> {
    elf: ElfFile<'static>,

    page_table: OffsetPageTable<'static>,

    allocator: &'a mut A,
}

impl<'a, A> KernelLoader<'a, A>
where
    A: FrameAllocator<Size4KiB>,
{
    pub fn new(input: &'static [u8], allocator: &'a mut A) -> Self {
        // UEFI maps vmem with a zero offset.
        let page_table = unsafe {
            let frame = allocate_zeroed_frame(allocator);
            let p4_table = &mut *(frame.start_address().as_u64() as *mut _);
            OffsetPageTable::new(p4_table, VirtAddr::zero())
        };

        let elf = ElfFile::new(input).expect("failed to parse elf");
        Self::check_dynamic(&elf);

        Self {
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

    pub fn load(mut self) -> (OffsetPageTable<'static>, VirtAddr, KernelEntry) {
        let file_base = PhysAddr::new(self.elf.input.as_ptr() as u64);

        for segment in self
            .elf
            .program_iter()
            .filter(|p| p.get_type().expect("bad type") == program::Type::Load && p.mem_size() > 0)
        {
            info!("begin to map segment {:x?}", segment);

            // TODO: use correct flags
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

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

        for page in Page::<Size1GiB>::range_inclusive(
            Page::containing_address(VirtAddr::zero()),
            Page::containing_address(VirtAddr::new(0xffffffff)),
        ) {
            let frame = PhysFrame::from_start_address(PhysAddr::new(page.start_address().as_u64()))
                .unwrap();

            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

            unsafe {
                self.page_table
                    .map_to(page, frame, flags, self.allocator)
                    .expect("failed to map page")
                    .flush();

                info!("mapped {:?} to {:?}", page, frame);
            }
        }

        let kernel_stack_top = VirtAddr::new(0x666700000000u64);
        let stack_page = Page::containing_address(kernel_stack_top);
        for i in 0..20 {
            let page = stack_page - i;
            let frame = allocate_zeroed_frame(self.allocator);

            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

            unsafe {
                self.page_table
                    .map_to(page, frame, flags, self.allocator)
                    .expect("failed to map page")
                    .flush();

                info!("mapped {:?} to {:?}", page, frame);
            }
        }

        (
            self.page_table,
            kernel_stack_top,
            entry_point as KernelEntry,
        )
    }
}

fn allocate_zeroed_frame(allocator: &mut impl FrameAllocator<Size4KiB>) -> PhysFrame<Size4KiB> {
    let frame = allocator
        .allocate_frame()
        .expect("failed to allocate frame");
    let ptr = frame.start_address().as_u64() as *mut u8;
    unsafe {
        core::ptr::write_bytes(ptr, 0, Size4KiB::SIZE as usize);
    }
    frame
}
