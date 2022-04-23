#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![feature(type_alias_impl_trait)]
#![feature(type_name_of_val)]
#![feature(naked_functions)]
#![feature(asm_sym)]
#![feature(trait_alias)]
#![feature(let_else)]
#![feature(proc_macro_hygiene)]
#![feature(stmt_expr_attributes)]

extern crate alloc;

mod acpi;
mod frame_allocator;
mod gdt;
mod heap;
mod interrupt;
mod kernel_task;
mod memory;
mod qemu;
mod serial_log;
mod syscall;
mod task;

use core::panic::PanicInfo;

use litchi_common::BootInfo;
use log::{error, info};
use spin::Once;
use x86_64::instructions;

use crate::qemu::{exit, ExitCode};

static BOOT_INFO: Once<&'static BootInfo> = Once::new();

#[no_mangle]
pub extern "C" fn _kernel_main(boot_info: *const BootInfo) -> ! {
    // Initialize serial logger
    serial_log::init();
    info!("Hello, the Litchi kernel!");

    // Store the global boot info
    BOOT_INFO.call_once(|| unsafe { &(*boot_info) });
    info!("boot info: {:?}", BOOT_INFO.get().unwrap());

    // Check BSS
    memory_check();

    // Initialize memories
    gdt::init();
    frame_allocator::init();
    memory::init();
    heap::init();

    // Initialize interrupts
    acpi::init();
    interrupt::disable();
    interrupt::init();

    // Test interrupts
    instructions::interrupts::int3();

    kernel_task::init();

    task::load();
    task::run();
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
