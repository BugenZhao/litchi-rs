mod global;
mod raii;

use log::info;
use spin::Mutex;
use x86_64::structures::paging::FrameAllocator;

use self::global::{GlobalFrameAllocator, FRAME_ALLOCATOR};
use crate::BOOT_INFO;

pub use raii::RaiiFrameAllocator;

pub fn init() {
    let boot_info = BOOT_INFO.get().expect("boot info not set");
    FRAME_ALLOCATOR.call_once(|| Mutex::new(GlobalFrameAllocator::new(boot_info)));

    {
        let mut allocator = FRAME_ALLOCATOR.get().unwrap().lock();
        let _test_frame = allocator
            .allocate_frame()
            .expect("failed to allocate test frame");
    }

    info!("initialized frame allocator from boot info");
}
