#![no_std]
#![feature(core_intrinsics)]
#![feature(alloc_error_handler)]

extern crate alloc;

mod heap;
pub mod print;
pub mod syscall;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    syscall::sys_exit();
}

extern "C" {
    fn main();
}

#[no_mangle]
pub extern "C" fn _user_main() {
    heap::init();
    unsafe { main() };
    syscall::sys_exit();
}
