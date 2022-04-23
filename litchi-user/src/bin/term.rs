#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use litchi_user::{
    println,
    syscall::{sys_get_task_id, sys_open, sys_read},
};

#[no_mangle]
extern "C" fn main() {
    let id = sys_get_task_id();
    let term = sys_open("/device/term").unwrap();

    let buf = &mut [0u8; 256];
    while let Ok(len) = sys_read(term, buf) {
        let read = &buf[..len];
        let str = String::from_utf8_lossy(read);
        println!("Task {}: from terminal: {}", id, str);
    }
}
