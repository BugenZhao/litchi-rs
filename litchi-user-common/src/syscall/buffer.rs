use core::{
    arch::asm,
    mem::{size_of, MaybeUninit},
};

use super::{Syscall, SYSCALL_BUFFER_BYTES, SYSCALL_IN_ADDR};
use spin::Mutex;

lazy_static::lazy_static! {
    pub static ref SYSCALL_IN_BUFFER: Mutex<SyscallInBuffer> = Mutex::new(SyscallInBuffer::new());
}

pub struct SyscallInBuffer(&'static mut [u8; SYSCALL_BUFFER_BYTES]);

impl SyscallInBuffer {
    fn new() -> Self {
        let buffer = unsafe {
            core::slice::from_raw_parts_mut(SYSCALL_IN_ADDR.as_mut_ptr(), SYSCALL_BUFFER_BYTES)
        }
        .try_into()
        .unwrap();

        Self(buffer)
    }

    pub(super) unsafe fn call(&mut self, syscall: Syscall) {
        core::ptr::copy_nonoverlapping(
            &syscall as *const _ as *const u8,
            self.0.as_mut_ptr(),
            size_of::<Syscall>(),
        );

        asm!("int 114"); // SYSCALL_INTERRUPT
    }

    pub(super) unsafe fn get(&self) -> Syscall {
        let mut syscall = MaybeUninit::uninit();
        core::ptr::copy_nonoverlapping(
            self.0.as_ptr(),
            syscall.as_mut_ptr() as *mut u8,
            size_of::<Syscall>(),
        );

        syscall.assume_init()
    }
}
