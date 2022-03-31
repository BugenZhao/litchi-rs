use litchi_common::elf_loader::allocate_zeroed_frame;
use log::debug;

use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, PhysFrame, Size1GiB,
        Size4KiB,
    },
    PhysAddr, VirtAddr,
};

pub fn create_kernel_page_table(
    allocator: &mut impl FrameAllocator<Size4KiB>,
) -> (PhysFrame, OffsetPageTable<'static>) {
    let frame = allocate_zeroed_frame(allocator);

    // UEFI maps vmem with a zero offset.
    let mut page_table = unsafe {
        let p4_table = &mut *(frame.start_address().as_u64() as *mut _);
        OffsetPageTable::new(p4_table, VirtAddr::zero())
    };

    // Map 0-4 GiB
    for page in Page::<Size1GiB>::range_inclusive(
        Page::containing_address(VirtAddr::zero()),
        Page::containing_address(VirtAddr::new(0xffffffff)),
    ) {
        let frame =
            PhysFrame::from_start_address(PhysAddr::new(page.start_address().as_u64())).unwrap();

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        unsafe {
            page_table
                .map_to(page, frame, flags, allocator)
                .expect("failed to map page")
                .flush();

            debug!("mapped {:?} to {:?}", page, frame);
        }
    }

    (frame, page_table)
}
