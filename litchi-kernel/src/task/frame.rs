use core::arch::asm;

use log::debug;
use x86_64::{
    registers::{
        self,
        segmentation::{Segment, SegmentSelector},
    },
    structures::idt::InterruptStackFrameValue,
    PrivilegeLevel,
};

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
pub struct TaskFrame {
    pub es: u64,
    pub ds: u64,
    pub regs: Registers,
    pub frame: InterruptStackFrameValue,
}

impl TaskFrame {
    pub fn is_user(&self) -> bool {
        SegmentSelector(self.frame.code_segment as u16).rpl() == PrivilegeLevel::Ring3
    }

    pub unsafe fn pop(self) -> ! {
        // Manually set ds & es here, since I don't know how to write in inline assembly :(
        registers::segmentation::DS::set_reg(SegmentSelector(self.ds as u16));
        registers::segmentation::ES::set_reg(SegmentSelector(self.es as u16));

        debug!("loaded ds = {}, es = {}", self.ds, self.es);

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
            in(reg) &self,
            options(noreturn)
        )
    }
}
