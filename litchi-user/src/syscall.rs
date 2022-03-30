use litchi_user_common::syscall::{syscall, Syscall, SyscallResponse};
use x86_64::VirtAddr;

pub fn sys_print(args: core::fmt::Arguments) {
    unsafe { syscall(Syscall::Print { args }) };
}

pub fn sys_extend_heap(top: VirtAddr) {
    unsafe { syscall(Syscall::ExtendHeap { top }) };
}

pub fn sys_get_task_id() -> u64 {
    let response = unsafe { syscall(Syscall::GetTaskId) };
    match response {
        SyscallResponse::GetTaskId { task_id } => task_id,
        _ => unreachable!(),
    }
}

pub fn sys_exit() -> ! {
    unsafe {
        syscall(Syscall::Exit);
        core::intrinsics::unreachable()
    }
}
