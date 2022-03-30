use litchi_user_common::syscall::{syscall, Syscall};

pub fn sys_print_hello() {
    unsafe { syscall(Syscall::PrintHello) }
}
