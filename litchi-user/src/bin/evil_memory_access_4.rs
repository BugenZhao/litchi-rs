#![no_std]
#![no_main]

// Read invalid memory to make kernel page fault.

extern crate alloc;
extern crate litchi_user;

use alloc::slice;

use litchi_user::syscall::sys_print;

#[no_mangle]
extern "C" fn main() {
    let kernel_slice = unsafe { slice::from_raw_parts(0x999900000000 as *mut u8, 16) };

    let str = unsafe { core::str::from_utf8_unchecked(kernel_slice) };

    sys_print(str);
}
