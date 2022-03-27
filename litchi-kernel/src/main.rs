#![no_std]
#![no_main]

use core::panic::PanicInfo;

static mut TEST_BSS: &mut [u8] = &mut [0; 4096];

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // this function is the entry point, since the linker looks for a function
    // named `_start` by default

    let a = &mut [1, 2, 3];
    for i in a.iter_mut() {
        *i += 1;
    }

    unsafe {
        for byte in TEST_BSS.iter_mut() {
            if *byte != 0 {
                loop {}
            }
            *byte = 233;
        }
    }

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
