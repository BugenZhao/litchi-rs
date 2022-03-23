#![no_main]
#![no_std]
#![feature(abi_efiapi)]

extern crate alloc;

use log::info;
use uefi::{prelude::*, proto::console::text::Color};

use crate::{frame_allocator::BootFrameAllocator, kernel_loader::KernelLoader};

pub mod frame_allocator;
pub mod kernel_loader;

static KERNEL_ELF_BYTES: &[u8] =
    include_bytes!("../../target/x86_64-unknown-none/debug/litchi-kernel");

#[entry]
fn efi_main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).expect("failed to init services");
    unsafe {
        uefi::alloc::init(system_table.boot_services());
    }

    system_table
        .stdout()
        .set_color(Color::Magenta, Color::Black)
        .expect("failed to set color");

    info!("Hello, litchi boot!");

    let mut allocator = BootFrameAllocator::new(system_table.boot_services());
    let kernel_loader = KernelLoader::new(KERNEL_ELF_BYTES, &mut allocator);

    let _kernel_entry = kernel_loader.load();

    // uefi::alloc::exit_boot_services();
    // system_table.exit_boot_services(image, mmap_buf);

    // unsafe { (*kernel_entry)() }

    #[allow(unreachable_code)]
    loop {
        x86_64::instructions::hlt();
    }
}
