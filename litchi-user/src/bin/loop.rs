#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

use litchi_user::println;

extern crate litchi_user;

#[no_mangle]
extern "C" fn main() {
    println!("welcome litchi user program");
    for _ in 0..10000000 {}
    println!("goodbye litchi user program");
}
