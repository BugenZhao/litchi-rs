use litchi_common::BootInfo;
use log::info;
use spin::{Mutex, Once};
use x86_64::{
    structures::paging::{FrameAllocator, PhysFrame, Size4KiB},
    PhysAddr,
};

use crate::BOOT_INFO;

type UsableFrameIterator = impl Iterator<Item = PhysFrame>;

fn extract_iter_from_boot_info(boot_info: &'static BootInfo) -> UsableFrameIterator {
    boot_info.usable_memory_ranges().flat_map(|desc| {
        let start = PhysFrame::from_start_address(PhysAddr::new(desc.phys_start))
            .expect("phys frame not aligned");
        let end = start + desc.page_count;

        PhysFrame::<Size4KiB>::range(start, end).into_iter()
    })
}

pub struct BootInfoFrameAllocator {
    iter: UsableFrameIterator,
}

impl BootInfoFrameAllocator {
    pub fn new(boot_info: &'static BootInfo) -> Self {
        Self {
            iter: extract_iter_from_boot_info(boot_info),
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        self.iter.next()
    }
}

static FRAME_ALLOCATOR: Once<Mutex<BootInfoFrameAllocator>> = Once::new();

pub fn init() {
    let boot_info = BOOT_INFO.get().expect("boot info not set");
    FRAME_ALLOCATOR.call_once(|| Mutex::new(BootInfoFrameAllocator::new(boot_info)));

    let _test_frame = FRAME_ALLOCATOR
        .get()
        .unwrap()
        .lock()
        .allocate_frame()
        .expect("failed to allocate test frame");

    info!("initialized frame allocator from boot info");
}
