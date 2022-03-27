#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![feature(type_alias_impl_trait)]

mod frame_allocator;
mod gdt;
mod interrupts;
mod qemu;
mod serial_log;

use core::panic::PanicInfo;

use litchi_boot::BootInfo;
use log::{error, info};
use spin::Once;
use x86_64::instructions;

use crate::qemu::{exit, ExitCode};

static BOOT_INFO: Once<&'static BootInfo> = Once::new();

#[allow(unreachable_code)]
#[no_mangle]
pub extern "C" fn kernel_main(boot_info: *const BootInfo) {
    serial_log::init().expect("failed to init serial logger");
    info!("Hello, the Litchi kernel!");

    BOOT_INFO.call_once(|| unsafe { &(*boot_info) });
    info!("boot info: {:#?}", BOOT_INFO.get().unwrap());

    memory_check();

    gdt::init();
    interrupts::init();
    frame_allocator::init();

    interrupts::enable();
    instructions::interrupts::int3();
    loop {
        instructions::hlt();
    }

    exit(ExitCode::Success);
}

fn memory_check() {
    static mut TEST_BSS: &mut [u8] = &mut [0; 10000];

    unsafe {
        for byte in TEST_BSS.iter_mut() {
            assert_eq!(*byte, 0, "bss check failed");
            *byte = 233;
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{}", info);
    exit(ExitCode::Failed);
}

#[allow(unconditional_recursion)]
#[allow(dead_code)]
fn stack_overflow() {
    stack_overflow();
}
