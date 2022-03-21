#![no_main]
#![no_std]
#![feature(abi_efiapi)]

use log::info;
use uefi::{prelude::*, proto::console::text::Color};

#[entry]
fn efi_main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).expect("failed to init services");

    system_table
        .stdout()
        .set_color(Color::Magenta, Color::Black)
        .expect("failed to set color");

    info!("Hello, litchi boot!");

    loop {
        x86_64::instructions::hlt();
    }
}
