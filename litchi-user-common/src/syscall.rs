use x86_64::{
    structures::paging::{PageSize, Size4KiB},
    VirtAddr,
};

use self::buffer::SYSCALL_IN_BUFFER;

pub const SYSCALL_INTERRUPT: u8 = 114;

pub const SYSCALL_IN_ADDR: VirtAddr = VirtAddr::new_truncate(0x1333_0000_0000);
pub const SYSCALL_OUT_ADDR: VirtAddr = VirtAddr::new_truncate(0x1334_0000_0000);
pub const SYSCALL_BUFFER_PAGES: u64 = 10;
pub const SYSCALL_BUFFER_BYTES: usize = (SYSCALL_BUFFER_PAGES * Size4KiB::SIZE) as usize;

mod buffer;

#[derive(Debug)]
pub enum Syscall<'a> {
    Print { args: core::fmt::Arguments<'a> },
    ExtendHeap { top: VirtAddr },
    Exit,
}

#[derive(Debug)]
pub enum SyscallResponse {
    PrintHello { task_id: u64 },
}

pub unsafe fn syscall(syscall: Syscall) {
    SYSCALL_IN_BUFFER.lock().call(syscall)
}

pub unsafe fn get_syscall() -> Syscall<'static> {
    SYSCALL_IN_BUFFER.lock().get()
}
