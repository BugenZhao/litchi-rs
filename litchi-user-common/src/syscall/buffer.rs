use core::{
    arch::asm,
    marker::PhantomData,
    mem::{size_of, MaybeUninit},
};

use super::{Syscall, SyscallResponse, SYSCALL_BUFFER_BYTES, SYSCALL_IN_ADDR, SYSCALL_OUT_ADDR};
use spin::Mutex;
use x86_64::VirtAddr;

pub struct In;
pub struct Out;

lazy_static::lazy_static! {
    pub static ref SYSCALL_IN_BUFFER: Mutex<SyscallBuffer<In>> = Mutex::new(SyscallBuffer::new(SYSCALL_IN_ADDR));
    pub static ref SYSCALL_OUT_BUFFER: Mutex<SyscallBuffer<Out>> = Mutex::new(SyscallBuffer::new(SYSCALL_OUT_ADDR));
}

pub struct SyscallBuffer<T> {
    buffer: &'static mut [u8; SYSCALL_BUFFER_BYTES],

    _phantom: PhantomData<T>,
}

impl<T> SyscallBuffer<T> {
    fn new(base: VirtAddr) -> Self {
        let buffer =
            unsafe { core::slice::from_raw_parts_mut(base.as_mut_ptr(), SYSCALL_BUFFER_BYTES) }
                .try_into()
                .unwrap();

        Self {
            buffer,
            _phantom: PhantomData,
        }
    }

    unsafe fn put<I>(&mut self, item: I) {
        core::ptr::copy_nonoverlapping(
            &item as *const _ as *const u8,
            self.buffer.as_mut_ptr(),
            size_of::<I>(),
        );
    }

    unsafe fn get<I>(&self) -> I {
        let mut item = MaybeUninit::uninit();
        core::ptr::copy_nonoverlapping(
            self.buffer.as_ptr(),
            item.as_mut_ptr() as *mut u8,
            size_of::<I>(),
        );

        item.assume_init()
    }
}

impl SyscallBuffer<In> {
    pub(super) unsafe fn call(&mut self, syscall: Syscall) {
        self.put(syscall);
        asm!("int 114"); // SYSCALL_INTERRUPT
    }

    pub(super) unsafe fn get_syscall(&self) -> Syscall<'static> {
        self.get()
    }
}

impl SyscallBuffer<Out> {
    pub(super) unsafe fn response(&mut self, response: SyscallResponse) {
        self.put(response);
    }

    pub(super) unsafe fn get_response(&self) -> SyscallResponse {
        self.get()
    }
}
