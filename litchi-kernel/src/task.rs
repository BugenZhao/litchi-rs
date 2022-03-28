use core::arch::asm;

use align_data::{include_aligned, Align4K};
use litchi_common::elf_loader::{ElfLoader, LoaderConfig};
use log::info;
use x86_64::VirtAddr;

use crate::memory::PageTableWrapper;

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct Registers {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
}

static EMBEDDED_USER_BIN: &[u8] = include_aligned!(
    Align4K,
    "../../target/x86_64-unknown-litchi-user/debug/loop"
);

pub fn init() {
    const USER_STACK_TOP: u64 = 0x1889_0000_0000;

    let page_table = PageTableWrapper::new_user();
    let loader_config = LoaderConfig {
        stack_top: VirtAddr::new(USER_STACK_TOP),
        stack_pages: 10,
        userspace: true,
    };

    let entry_point = page_table.with_allocator(|frame_allocator, page_table| {
        ElfLoader::new(
            &loader_config,
            EMBEDDED_USER_BIN,
            frame_allocator,
            page_table,
        )
        .load()
    });
    info!("loaded embedded user binary, entry point {:p}", entry_point);

    page_table.load();
    info!("loaded user page table");

    unsafe {
        asm!("mov rsp, {}; call {}",
            in(reg) USER_STACK_TOP,
            in(reg) entry_point,
        );
    }
}
