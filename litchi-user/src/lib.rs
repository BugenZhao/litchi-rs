#![no_std]
#![feature(core_intrinsics)]
#![feature(default_alloc_error_handler)]

extern crate alloc;

pub mod heap;
pub mod syscall;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
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
