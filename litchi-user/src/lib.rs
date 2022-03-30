#![no_std]
#![feature(core_intrinsics)]

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
    unsafe { main() };
    syscall::sys_exit();
}
