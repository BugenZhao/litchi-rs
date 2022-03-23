#![no_std]
#![no_main]

use core::panic::PanicInfo;

// static mut _TEST_BSS: &mut [u8] = &mut [0; 4096];

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // this function is the entry point, since the linker looks for a function
    // named `_start` by default

    // unsafe {
    //     for byte in _TEST_BSS.iter_mut() {
    //         *byte = 233;
    //     }
    // }

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
