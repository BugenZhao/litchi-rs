use litchi_user_common::syscall::{syscall, Syscall};

pub fn sys_print_hello(name: &'static str) {
    unsafe { syscall(Syscall::PrintHello { name }) }
}
