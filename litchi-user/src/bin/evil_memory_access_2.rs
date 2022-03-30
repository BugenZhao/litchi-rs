#![no_std]
#![no_main]

// Read kernel memory by cheating the print syscall. However we format it in userspace, so this leads to page fault.

extern crate alloc;
extern crate litchi_user;

use alloc::slice;
use litchi_user::println;

#[no_mangle]
extern "C" fn main() {
    let kernel_slice = unsafe { slice::from_raw_parts(0x233300000000 as *mut u8, 16) };

    println!("try to read: {:?}", kernel_slice);
}
