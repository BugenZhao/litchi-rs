#![no_std]
#![no_main]

use litchi_user::{
    println,
    syscall::{sys_get_task_id, sys_sleep},
};

#[no_mangle]
extern "C" fn main() {
    let id = sys_get_task_id();
    let sleep_slices = 5;

    println!("Task {}: hello", id);
    sys_sleep(sleep_slices);
    println!(
        "Task {}: goodbye after sleeping {} slices",
        id, sleep_slices
    );
}
