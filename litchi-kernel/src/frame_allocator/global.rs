use alloc::collections::VecDeque;

use litchi_common::BootInfo;
use spin::{Mutex, Once};
use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

use crate::heap;

type UsableFrameIterator = impl Iterator<Item = PhysFrame>;

fn extract_iter_from_boot_info(boot_info: &'static BootInfo) -> UsableFrameIterator {
    boot_info.usable_memory_ranges().flat_map(|desc| {
        let start = PhysFrame::from_start_address(PhysAddr::new(desc.phys_start))
            .expect("phys frame not aligned");
        let end = start + desc.page_count;

        PhysFrame::<Size4KiB>::range(start, end)
    })
}

pub(super) struct GlobalFrameAllocator {
    iter: UsableFrameIterator,

    deallocated: Option<VecDeque<PhysFrame>>,
}

impl GlobalFrameAllocator {
    pub(super) fn new(boot_info: &'static BootInfo) -> Self {
        Self {
            iter: extract_iter_from_boot_info(boot_info),
            deallocated: None,
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for GlobalFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        self.iter
            .next()
            .or_else(|| self.deallocated.as_mut().and_then(|de| de.pop_front()))
    }
}

impl FrameDeallocator<Size4KiB> for GlobalFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        let de = self.deallocated.get_or_insert_with(|| {
            assert!(
                heap::initialized(),
                "cannot deallocate frame when kernel heap allocator not initialized"
            );
            VecDeque::new()
        });

        de.push_back(frame);
    }
}

pub(super) static FRAME_ALLOCATOR: Once<Mutex<GlobalFrameAllocator>> = Once::new();
