#![no_std]
#![no_main]

extern crate alloc;
extern crate litchi_user;

use alloc::slice;
use litchi_user::println;

#[no_mangle]
extern "C" fn main() {
    let kernel_slice = unsafe { slice::from_raw_parts_mut(0x233300000000 as *mut u8, 16) };

    // TODO: fix this previlege cheating
    println!("try to read: {:?}", kernel_slice);

    for i in kernel_slice.iter_mut() {
        *i = 233;
    }

    println!("try to write: {:?}", kernel_slice);
}
