use core::ops::Deref;

use lazy_static::lazy_static;
use litchi_user_common::syscall::SYSCALL_INTERRUPT;
use log::info;

use x86_64::{
    instructions, set_general_handler, structures::idt::InterruptDescriptorTable, PrivilegeLevel,
};

use crate::gdt::IstIndex;

mod io_apic;
mod local_apic;
mod macros;
mod trap_handlers;
mod user_handlers;

pub const USER_INTERRUPT_OFFSET: u8 = 32;
pub const IO_APIC_INTERRUPT_OFFSET: u8 = 128;

pub type RawUserInterrupt = u8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum UserInterrupt {
    ApicTimer = USER_INTERRUPT_OFFSET,
    ApicError = USER_INTERRUPT_OFFSET + 19,
    ApicSpurious = USER_INTERRUPT_OFFSET + 31,

    Syscall = SYSCALL_INTERRUPT,

    #[allow(unused)]
    SerialIn = IO_APIC_INTERRUPT_OFFSET + 4,
}

impl From<RawUserInterrupt> for UserInterrupt {
    fn from(raw: RawUserInterrupt) -> Self {
        unsafe { core::mem::transmute(raw) }
    }
}

impl UserInterrupt {
    fn as_index(self) -> usize {
        self as u8 as _
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = new_idt();
}

fn new_idt() -> InterruptDescriptorTable {
    use trap_handlers::*;
    use user_handlers::*;

    let mut idt = InterruptDescriptorTable::new();

    // default unhandled
    set_general_handler!(&mut idt, unhandled);

    // Breakpoint
    idt.breakpoint.set_handler_fn(breakpoint);

    // Double fault
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault)
            .set_stack_index(IstIndex::DoubleFault as u16);
    }

    // Page fault
    unsafe {
        idt.page_fault
            .set_handler_fn(page_fault)
            .set_privilege_level(PrivilegeLevel::Ring3)
            .set_stack_index(IstIndex::UserInterrupt as u16);
    }

    // APIC Timer
    unsafe {
        idt[UserInterrupt::ApicTimer.as_index()]
            .set_handler_fn(apic_timer)
            .set_stack_index(IstIndex::UserInterrupt as u16);
    }

    // Serial in
    unsafe {
        idt[UserInterrupt::SerialIn.as_index()]
            .set_handler_fn(serial_in)
            .set_stack_index(IstIndex::UserInterrupt as u16);
    }

    // Syscall
    unsafe {
        idt[UserInterrupt::Syscall.as_index()]
            .set_handler_fn(syscall)
            .set_privilege_level(PrivilegeLevel::Ring3)
            .set_stack_index(IstIndex::UserInterrupt as u16);
    }

    idt
}

pub fn init() {
    IDT.load();
    info!("loaded idt at {:p}", IDT.deref());

    local_apic::enable();
    info!("enabled local apic with timer");

    io_apic::enable_irqs();
    info!("initialized and enabled io apic");
}

#[allow(dead_code)]
pub fn enable() {
    instructions::interrupts::enable();
    info!("enabled interrupts");
}

#[allow(dead_code)]
pub fn disable() {
    instructions::interrupts::disable();
    info!("disabled interrupts");
}
