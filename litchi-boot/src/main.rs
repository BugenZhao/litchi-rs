#![no_main]
#![no_std]
#![feature(abi_efiapi)]

extern crate alloc;

use core::arch::asm;

use align_data::{include_aligned, Align4K};
use alloc::vec;
use log::info;
use uefi::{prelude::*, proto::console::text::Color};
use x86_64::{
    registers::control::{Cr3, Cr3Flags},
    structures::paging::PhysFrame,
    PhysAddr,
};

use crate::{frame_allocator::BootFrameAllocator, kernel_loader::KernelLoader};

pub mod frame_allocator;
pub mod kernel_loader;

static KERNEL_ELF_BYTES: &[u8] = include_aligned!(
    Align4K,
    "../../target/x86_64-unknown-litchi/debug/litchi-kernel"
);

#[entry]
fn efi_main(handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
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

    let (mut page_table, kernel_stack_top, kernel_entry) = kernel_loader.load();

    unsafe {
        Cr3::write(
            PhysFrame::from_start_address(PhysAddr::new(
                page_table.level_4_table() as *const _ as u64
            ))
            .expect("page table is not aligned"),
            Cr3Flags::empty(),
        );
    }

    info!("loaded kernel page table");

    let mmap_size = system_table.boot_services().memory_map_size().map_size;
    let mmap_buf = vec![0u8; mmap_size * 2].leak();

    uefi::alloc::exit_boot_services();
    let (_system_table, _iter) = system_table
        .exit_boot_services(handle, mmap_buf)
        .expect("failed to exit boot services");

    // Note: we can not use log & alloc anymore.

    unsafe {
        asm!("mov rsp, {}; call {}", in(reg) kernel_stack_top.as_u64(), in(reg) kernel_entry);
    }

    Status::SUCCESS
}
