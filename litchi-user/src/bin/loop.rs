#![no_std]
#![no_main]

use litchi_user::syscall;

extern crate litchi_user;

#[no_mangle]
extern "C" fn main() {
    syscall::sys_print_hello("welcome litchi user program");
    for _ in 0..10000000 {}
    syscall::sys_print_hello("goodbye litchi user program");
}
