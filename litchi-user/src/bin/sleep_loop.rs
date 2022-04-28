#![no_std]
#![no_main]

use litchi_user::println;
use litchi_user::syscall::{sys_get_task_id, sys_sleep};

#[no_mangle]
extern "C" fn main() {
    let id = sys_get_task_id();
    let sleep_slices = 50;

    loop {
        println!("[Task 0x{:x}] I'm running.", id);
        sys_sleep(sleep_slices);
    }
}
