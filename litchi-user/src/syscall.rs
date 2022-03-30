use litchi_user_common::syscall::{syscall, Syscall};
use x86_64::VirtAddr;

pub fn sys_print(args: core::fmt::Arguments) {
    unsafe { syscall(Syscall::Print { args }) };
}

pub fn sys_extend_heap(top: VirtAddr) {
    unsafe { syscall(Syscall::ExtendHeap { top }) };
}

pub fn sys_get_task_id() -> u64 {
    unsafe { syscall(Syscall::GetTaskId) }
        .into_get_task_id()
        .unwrap()
}

pub fn sys_yield() {
    unsafe { syscall(Syscall::Yield) };
}

pub fn sys_exit() -> ! {
    unsafe {
        syscall(Syscall::Exit);
        core::intrinsics::unreachable()
    }
}
