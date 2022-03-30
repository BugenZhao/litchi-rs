use litchi_user_common::syscall::{syscall, Syscall};

pub fn sys_print_hello(name: &'static str) {
    unsafe { syscall(Syscall::PrintHello { name }) }
}

pub fn sys_exit() -> ! {
    unsafe {
        syscall(Syscall::Exit);
        core::intrinsics::unreachable()
    }
}
