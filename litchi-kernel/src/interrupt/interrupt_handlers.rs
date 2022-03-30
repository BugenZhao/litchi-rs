use core::arch::asm;

use log::info;
use x86_64::structures::idt::InterruptStackFrame;

use crate::{print, task::Registers};

#[naked]
pub extern "x86-interrupt" fn reg_preserving_apic_timer(frame: InterruptStackFrame) {
    unsafe {
        asm!(
            "mov    qword ptr [rsp - 120], r15",
            "mov    qword ptr [rsp - 112], r14",
            "mov    qword ptr [rsp - 104], r13",
            "mov    qword ptr [rsp - 96], r12",
            "mov    qword ptr [rsp - 88], r11",
            "mov    qword ptr [rsp - 80], r10",
            "mov    qword ptr [rsp - 72], r9",
            "mov    qword ptr [rsp - 64], r8",
            "mov    qword ptr [rsp - 56], rsi",
            "mov    qword ptr [rsp - 48], rdi",
            "mov    qword ptr [rsp - 40], rbp",
            "mov    qword ptr [rsp - 32], rdx",
            "mov    qword ptr [rsp - 24], rcx",
            "mov    qword ptr [rsp - 16], rbx",
            "mov    qword ptr [rsp - 8],  rax",
            "lea    rdi, [rsp]",
            "lea    rsi, [rsp - 120]",
            "sub    rsp, 120",
            "call   {}",
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
            sym reg_preserving_inner,
            options(noreturn)
        )
    }
}

#[inline]
extern "C" fn reg_preserving_inner(_stack_frame: &InterruptStackFrame, _regs: &Registers) {
    print!(".");

    unsafe {
        super::LOCAL_APIC.lock().end_of_interrupt();
    }
}

#[naked]
pub extern "x86-interrupt" fn reg_preserving_print_hello(frame: InterruptStackFrame) {
    unsafe {
        asm!(
            "mov    qword ptr [rsp - 120], r15",
            "mov    qword ptr [rsp - 112], r14",
            "mov    qword ptr [rsp - 104], r13",
            "mov    qword ptr [rsp - 96], r12",
            "mov    qword ptr [rsp - 88], r11",
            "mov    qword ptr [rsp - 80], r10",
            "mov    qword ptr [rsp - 72], r9",
            "mov    qword ptr [rsp - 64], r8",
            "mov    qword ptr [rsp - 56], rsi",
            "mov    qword ptr [rsp - 48], rdi",
            "mov    qword ptr [rsp - 40], rbp",
            "mov    qword ptr [rsp - 32], rdx",
            "mov    qword ptr [rsp - 24], rcx",
            "mov    qword ptr [rsp - 16], rbx",
            "mov    qword ptr [rsp - 8],  rax",
            "lea    rdi, [rsp]",
            "lea    rsi, [rsp - 120]",
            "sub    rsp, 120",
            "call   {}",
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
            sym reg_preserving_print_hello_inner,
            options(noreturn)
        )
    }
}

#[inline]
extern "C" fn reg_preserving_print_hello_inner(
    _stack_frame: &InterruptStackFrame,
    _regs: &Registers,
) {
    info!("Hello from user!");

    unsafe {
        super::LOCAL_APIC.lock().end_of_interrupt();
    }
}
