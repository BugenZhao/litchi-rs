#![no_main]
#![no_std]
#![feature(abi_efiapi)]
#![feature(int_roundings)]

extern crate alloc;

use alloc::vec::Vec;
use core::arch::asm;

use litchi_common::elf_loader::{ElfLoader, LoaderConfig};
use litchi_common::BootInfo;
use log::info;
use uefi::prelude::*;
use uefi::proto::console::text::Color;
use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::VirtAddr;

use crate::frame_allocator::BootFrameAllocator;
use crate::page_table::create_kernel_page_table;

mod file_system;
mod frame_allocator;
mod page_table;

const KERNEL_PATH: &str = "litchi-kernel";

const KERNEL_STACK_TOP: u64 = 0x6667_0000_0000;
const KERNEL_STACK_PAGES: u64 = 20;

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

    info!("Hello, the Litchi bootloader!");

    let file = file_system::open(system_table.boot_services(), KERNEL_PATH);
    let kernel_elf_bytes = file_system::read(system_table.boot_services(), file);
    info!(
        "loaded kernel file `{}` at {:p}",
        KERNEL_PATH, kernel_elf_bytes
    );

    let mut allocator = BootFrameAllocator::new(system_table.boot_services());
    let (page_table_frame, mut page_table) = create_kernel_page_table(&mut allocator);
    info!("created kernel page table");

    let loader_config = LoaderConfig {
        stack_top: VirtAddr::new(KERNEL_STACK_TOP),
        stack_pages: KERNEL_STACK_PAGES,
        userspace: false,
    };
    let kernel_loader = ElfLoader::new(
        &loader_config,
        kernel_elf_bytes,
        &mut allocator,
        &mut page_table,
    );

    let kernel_entry = kernel_loader.load();
    info!("loaded kernel elf, entry {:p}", kernel_entry);

    unsafe {
        Cr3::write(page_table_frame, Cr3Flags::empty());
    }
    info!("loaded kernel page table");

    let mmap_size = system_table.boot_services().memory_map_size().map_size;
    let mmap_buf = alloc::vec![0u8; mmap_size * 2].leak();
    let memory_descriptors = Vec::with_capacity(128);

    info!("exit boot services & call the kernel entry");

    system_table
        .stdout()
        .set_color(Color::Yellow, Color::Black)
        .expect("failed to set color");

    uefi::alloc::exit_boot_services();
    let (system_table, mem_desc_iter) = system_table
        .exit_boot_services(handle, mmap_buf)
        .expect("failed to exit boot services");

    // Note: we can not use log & alloc anymore.
    let cap = memory_descriptors.capacity();
    let mut memory_descriptors = memory_descriptors;
    for mem_desc in mem_desc_iter {
        assert!(memory_descriptors.len() < cap);
        memory_descriptors.push(mem_desc);
    }

    let boot_info = BootInfo {
        name: "litchi",
        kernel_entry: VirtAddr::from_ptr(kernel_entry),
        kernel_stack_top: VirtAddr::new(KERNEL_STACK_TOP),
        kernel_page_table: page_table_frame,
        system_table,
        phys_offset: VirtAddr::zero(),
        memory_descriptors,
    };

    unsafe {
        asm!("mov rsp, {}; call {}",
            in(reg) KERNEL_STACK_TOP,
            in(reg) kernel_entry,
            in("rdi") &boot_info as *const _,
            options(noreturn)
        );
    }
}
