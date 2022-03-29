use core::arch::asm;

use align_data::{include_aligned, Align4K};
use litchi_common::elf_loader::{ElfLoader, LoaderConfig};
use log::info;
use x86_64::{
    registers::{
        self,
        segmentation::{Segment, SegmentSelector},
    },
    structures::idt::InterruptStackFrameValue,
    VirtAddr,
};

use crate::{gdt::GDT, memory::PageTableWrapper};

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

#[repr(C)]
#[derive(Debug)]
struct TaskFrame {
    pub es: u64,
    pub ds: u64,
    pub regs: Registers,
    pub frame: InterruptStackFrameValue,
}

impl TaskFrame {
    unsafe fn pop(&self) -> ! {
        // Manually set ds & es here, since I don't know how to write in inline assembly :(
        registers::segmentation::DS::set_reg(SegmentSelector(self.ds as u16));
        registers::segmentation::ES::set_reg(SegmentSelector(self.es as u16));

        info!("loaded ds = {}, es = {}", self.ds, self.es);

        asm!(
            "mov    rsp, {}",
            "add    rsp, 16", // skip es & ds
            "add    rsp, 120",
            "mov    r15, qword ptr [rsp - 120]",
            "mov    r14, qword ptr [rsp - 112]",
            "mov    r13, qword ptr [rsp - 104]",
            "mov    r12, qword ptr [rsp - 96]",
            "mov    r11, qword ptr [rsp - 88]",
            "mov    r10, qword ptr [rsp - 80]",
            "mov    r9,  qword ptr [rsp - 72]",
            "mov    r8,  qword ptr [rsp - 64]",
            "mov    rsi, qword ptr [rsp - 56]",
            "mov    rdi, qword ptr [rsp - 48]",
            "mov    rbp, qword ptr [rsp - 40]",
            "mov    rdx, qword ptr [rsp - 32]",
            "mov    rcx, qword ptr [rsp - 24]",
            "mov    rbx, qword ptr [rsp - 16]",
            "mov    rax, qword ptr [rsp - 8]",
            "iretq",
            in(reg) self,
            options(noreturn)
        )
    }
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

    let code_segment = GDT.user_code_selector.0 as u64;
    let data_segment = GDT.user_data_selector.0 as u64;

    let user_trap_frame = TaskFrame {
        es: data_segment,
        ds: data_segment,
        regs: Registers::default(),
        frame: InterruptStackFrameValue {
            instruction_pointer: VirtAddr::from_ptr(entry_point),
            code_segment,
            cpu_flags: 0x0000_0200, // enable interrupts
            stack_pointer: VirtAddr::new(USER_STACK_TOP),
            stack_segment: data_segment,
        },
    };

    unsafe {
        user_trap_frame.pop();
    }
}
