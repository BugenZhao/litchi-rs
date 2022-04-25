use uefi::prelude::BootServices;
use uefi::table::boot::{AllocateType, MemoryType};
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

pub struct BootFrameAllocator<'a>(&'a BootServices);

impl<'a> BootFrameAllocator<'a> {
    pub fn new(boot_services: &'a BootServices) -> Self {
        Self(boot_services)
    }
}

unsafe impl<'a> FrameAllocator<Size4KiB> for BootFrameAllocator<'a> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let phys_addr = self
            .0
            .allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 1)
            .ok()?;
        let frame =
            PhysFrame::from_start_address(PhysAddr::new(phys_addr)).expect("frame not aligned");

        Some(frame)
    }
}
