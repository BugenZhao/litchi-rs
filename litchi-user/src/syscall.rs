use litchi_user_common::syscall::{syscall, Syscall};
use x86_64::VirtAddr;

pub fn sys_print_hello(name: &'static str) {
    unsafe { syscall(Syscall::PrintHello { name }) }
}

pub fn sys_exit() -> ! {
    unsafe {
        syscall(Syscall::Exit);
        core::intrinsics::unreachable()
    }
}

pub fn sys_extend_heap(top: VirtAddr) {
    unsafe { syscall(Syscall::ExtendHeap { top }) }
}
