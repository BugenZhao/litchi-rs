use alloc::vec::Vec;
use log::info;
use x86_64::{
    instructions,
    structures::paging::{FrameAllocator, FrameDeallocator, PhysFrame, Size4KiB},
};

use super::FRAME_ALLOCATOR;

pub struct RaiiFrameAllocator {
    allocated: Option<Vec<PhysFrame>>,
}

impl RaiiFrameAllocator {
    /// For the user program.
    pub fn new_traced() -> Self {
        Self {
            allocated: Some(Vec::new()),
        }
    }

    /// For the kernel.
    pub fn new_untraced() -> Self {
        Self { allocated: None }
    }
}

unsafe impl FrameAllocator<Size4KiB> for RaiiFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = instructions::interrupts::without_interrupts(|| {
            FRAME_ALLOCATOR
                .get()
                .expect("frame allocator not initialized")
                .lock()
                .allocate_frame()
        });

        if let Some(allocated) = self.allocated.as_mut() {
            if let Some(frame) = frame {
                allocated.push(frame);
            }
        }
        frame
    }
}

impl Drop for RaiiFrameAllocator {
    fn drop(&mut self) {
        instructions::interrupts::without_interrupts(|| {
            let mut inner = FRAME_ALLOCATOR
                .get()
                .expect("frame allocator not initialized")
                .lock();

            if let Some(allocated) = self.allocated.take() {
                info!("will deallocate {} frames", allocated.len());

                for frame in allocated {
                    unsafe { inner.deallocate_frame(frame) };
                }
            }
        });
    }
}
