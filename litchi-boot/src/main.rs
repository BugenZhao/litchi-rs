#![no_main]
#![no_std]
#![feature(abi_efiapi)]

use log::info;
use uefi::{prelude::*, proto::console::text::Color};
use xmas_elf::ElfFile;

static KERNEL_ELF_BYTES: &[u8] =
    include_bytes!("../../target/x86_64-unknown-none/debug/litchi-kernel");

#[entry]
fn efi_main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).expect("failed to init services");

    system_table
        .stdout()
        .set_color(Color::Magenta, Color::Black)
        .expect("failed to set color");

    info!("Hello, litchi boot!");

    let kernel_elf = ElfFile::new(KERNEL_ELF_BYTES).expect("failed to parse kernel elf");
    info!("Kernel size: {}", KERNEL_ELF_BYTES.len());
    info!("Kernel ELF header: {}", kernel_elf.header);

    loop {
        x86_64::instructions::hlt();
    }
}
