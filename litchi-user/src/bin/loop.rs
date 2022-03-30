#![no_std]
#![no_main]

use litchi_user::syscall;

extern crate litchi_user;

#[no_mangle]
extern "C" fn main() {
    syscall::sys_print_hello("litchi user program 233");
    for _ in 0..2000000 {}
    syscall::sys_print_hello("litchi user program 666");
    loop {}
}
