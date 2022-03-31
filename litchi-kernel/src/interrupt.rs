use core::ops::Deref;

use acpi::platform::interrupt::{InterruptSourceOverride, IoApic as IoApicInfo};
use alloc::vec::Vec;
use lazy_static::lazy_static;
use litchi_user_common::syscall::SYSCALL_INTERRUPT;
use log::info;
use spin::Mutex;
use x2apic::{
    ioapic::{IoApic, IrqFlags, IrqMode, RedirectionTableEntry},
    lapic::{self, LocalApic},
};
use x86_64::{
    instructions, set_general_handler, structures::idt::InterruptDescriptorTable, PrivilegeLevel,
};

use crate::{acpi::ACPI, gdt::IstIndex};

mod macros;
mod trap_handlers;
mod user_handlers;

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

    // Syscall
    unsafe {
        idt[UserInterrupt::Syscall.as_index()]
            .set_handler_fn(syscall)
            .set_privilege_level(PrivilegeLevel::Ring3)
            .set_stack_index(IstIndex::UserInterrupt as u16);
    }

    idt
}

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
    Serial = IO_APIC_INTERRUPT_OFFSET + 4,
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
    static ref LOCAL_APIC: Mutex<LocalApic> = Mutex::new(new_local_apic());
}

fn new_local_apic() -> LocalApic {
    lapic::LocalApicBuilder::new()
        .error_vector(UserInterrupt::ApicError.as_index())
        .spurious_vector(UserInterrupt::ApicSpurious.as_index())
        .timer_vector(UserInterrupt::ApicTimer.as_index())
        .timer_initial(10_000_000 * 100)
        .set_xapic_base(ACPI.apic_info.local_apic_address) // or lapic::xapic_base()
        .build()
        .expect("failed to build lapic")
}

pub fn init() {
    IDT.load();
    info!("loaded idt at {:p}", IDT.deref());

    unsafe {
        LOCAL_APIC.lock().enable();
    }
    info!("enabled local apic with timer");

    init_io_apic();
    info!("initialized io apic");
}

struct IoApicWrapper {
    inner: IoApic,

    info: &'static IoApicInfo,
}

impl IoApicWrapper {
    fn handle(&self, global_system_interrupt: u32) -> bool {
        global_system_interrupt >= self.info.global_system_interrupt_base
            && global_system_interrupt < (self.info.global_system_interrupt_base + 24)
    }
}

struct IoApics {
    io_apics: Vec<IoApicWrapper>,

    overrides: Vec<&'static InterruptSourceOverride>,
}

impl IoApics {
    fn new() -> Self {
        let apic_info = &ACPI.apic_info;

        let io_apics = apic_info
            .io_apics
            .iter()
            .map(|io_apic_info| unsafe {
                let mut io_apic = IoApic::new(io_apic_info.address as u64);
                io_apic.init(IO_APIC_INTERRUPT_OFFSET);

                IoApicWrapper {
                    inner: io_apic,
                    info: io_apic_info,
                }
            })
            .collect();

        let overrides = apic_info.interrupt_source_overrides.iter().collect();

        Self {
            io_apics,
            overrides,
        }
    }

    fn irq_to_interrupt(&self, irq: u8) -> Option<(usize, u32)> {
        let overrided = self
            .overrides
            .iter()
            .find(|o| o.isa_source == irq)
            .map(|o| o.global_system_interrupt);

        let global_system_interrupt = overrided.unwrap_or(irq as u32);

        let io_apic_index = self
            .io_apics
            .iter()
            .enumerate()
            .find(|(_i, io_apic)| io_apic.handle(global_system_interrupt))?
            .0;

        Some((io_apic_index, global_system_interrupt))
    }

    fn enable_irq(&mut self, irq: u8) -> Option<RawUserInterrupt> {
        let (io_apic_index, global_system_interrupt) = self.irq_to_interrupt(irq)?;

        let user_interrupt = global_system_interrupt as u8 + IO_APIC_INTERRUPT_OFFSET;

        let mut entry = RedirectionTableEntry::default();
        entry.set_mode(IrqMode::Fixed);
        entry.set_flags(IrqFlags::MASKED | IrqFlags::LEVEL_TRIGGERED);
        entry.set_vector(user_interrupt);
        entry.set_dest(0); // CPU 0

        let io_apic = &mut self.io_apics[io_apic_index].inner;

        unsafe {
            io_apic.set_table_entry(global_system_interrupt as u8, entry);
            io_apic.enable_irq(global_system_interrupt as u8);
        }

        info!(
            "enabled irq #{} to user interrupt {} in io apic {}",
            irq, user_interrupt, io_apic_index
        );

        Some(user_interrupt)
    }
}

#[allow(dead_code)]
pub fn init_io_apic() {
    let mut io_apics = IoApics::new();
    io_apics.enable_irq(4);
}

#[allow(dead_code)]
pub fn enable() {
    instructions::interrupts::enable();
    info!("enabled interrupts");
}
