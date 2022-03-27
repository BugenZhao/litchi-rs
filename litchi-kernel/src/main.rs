#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

mod qemu;
mod serial_logger;

use core::panic::PanicInfo;

use litchi_boot::BootInfo;
use log::{error, info};
use spin::Once;

use crate::qemu::{exit, ExitCode};

static mut TEST_BSS: &mut [u8] = &mut [0; 10000];

static BOOT_INFO: Once<&'static BootInfo> = Once::new();

#[no_mangle]
pub extern "C" fn kernel_main(boot_info: *const BootInfo) {
    let a = &mut [1, 2, 3];
    for i in a.iter_mut() {
        *i += 1;
    }

    unsafe {
        for byte in TEST_BSS.iter_mut() {
            assert_eq!(*byte, 0, "bss check failed");
            *byte = 233;
        }
    }

    serial_logger::init().expect("failed to init serial logger");
    info!("Hello, the Litchi kernel!");

    BOOT_INFO.call_once(|| unsafe { &(*boot_info) });
    info!("boot info: {:#?}", BOOT_INFO.get().unwrap());

    exit(ExitCode::Success);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{}", info);
    exit(ExitCode::Failed);
}
