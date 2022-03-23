#![no_main]
#![no_std]
#![feature(abi_efiapi)]

extern crate alloc;

use core::arch::asm;

use align_data::{include_aligned, Align4K};
use alloc::slice;
use log::info;
use uefi::{prelude::*, proto::console::text::Color};
use x86_64::{
    registers::control::{Cr3, Cr3Flags},
    structures::paging::{PhysFrame, Translate},
    PhysAddr, VirtAddr,
};

use crate::{frame_allocator::BootFrameAllocator, kernel_loader::KernelLoader};

pub mod frame_allocator;
pub mod kernel_loader;

static KERNEL_ELF_BYTES: &[u8] = include_aligned!(
    Align4K,
    "../../target/x86_64-unknown-none/debug/litchi-kernel"
);

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

    let (mut page_table, kernel_stack_top, kernel_entry) = kernel_loader.load();

    // uefi::alloc::exit_boot_services();
    // system_table.exit_boot_services(image, mmap_buf);

    unsafe {
        Cr3::write(
            PhysFrame::from_start_address(PhysAddr::new(
                page_table.level_4_table() as *const _ as u64
            ))
            .unwrap(),
            Cr3Flags::empty(),
        );
    }

    info!("loaded kernel page table");

    unsafe {
        let es = kernel_entry as *const u8;
        let mem = slice::from_raw_parts(es, 128);
        info!("mem after entry point: \n{:x?}", mem);
    }

    let phys = page_table.translate_addr(VirtAddr::new(kernel_entry as u64));
    info!("phys {:?}", phys);

    unsafe {
        let ptr = KERNEL_ELF_BYTES.as_ptr().offset(0x220);
        info!(
            "mem after ptr {:p}: \n{:x?}",
            ptr,
            slice::from_raw_parts(ptr, 0x20)
        );
    }

    unsafe {
        asm!("mov rsp, {}; call {}", in(reg) kernel_stack_top.as_u64(), in(reg) kernel_entry);
    }

    Status::SUCCESS
}
