#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

use litchi_user::{
    println,
    syscall::{sys_get_task_id, sys_yield},
};

#[no_mangle]
extern "C" fn main() {
    let id = sys_get_task_id();
    println!("Task {}: hello, litchi user program", id);
    sys_yield();
    for _ in 0..10000000 {}
    println!("Task {}: goodbye, litchi user program", id);
}
