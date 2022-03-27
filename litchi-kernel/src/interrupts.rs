use core::ops::Deref;

use lazy_static::lazy_static;
use log::info;
use spin::Mutex;
use x2apic::lapic::{self, LocalApic};
use x86_64::{instructions, structures::idt::InterruptDescriptorTable};

use crate::gdt::DOUBLE_FAULT_IST_INDEX;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = new_idt();
}

fn new_idt() -> InterruptDescriptorTable {
    use handlers::*;

    let mut idt = InterruptDescriptorTable::new();

    // Breakpoint
    idt.breakpoint.set_handler_fn(breakpoint);

    // Double fault
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX);
    }

    // APIC Timer
    idt[UserInterrupt::ApicTimer as u8 as usize].set_handler_fn(apic_timer);

    idt
}

pub const USER_INTERRUPT_OFFSET: u8 = 32;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum UserInterrupt {
    ApicTimer = USER_INTERRUPT_OFFSET,
    ApicError = USER_INTERRUPT_OFFSET + 19,
    ApicSpurious = USER_INTERRUPT_OFFSET + 31,
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
    unsafe {
        lapic::LocalApicBuilder::new()
            .error_vector(UserInterrupt::ApicError.as_index())
            .spurious_vector(UserInterrupt::ApicSpurious.as_index())
            .timer_vector(UserInterrupt::ApicTimer.as_index())
            .timer_initial(10_000_000 * 10)
            .set_xapic_base(lapic::xapic_base())
            .build()
            .expect("failed to build lapic")
    }
}

pub fn init() {
    IDT.load();
    info!("loaded idt at {:p}", IDT.deref());

    unsafe {
        LOCAL_APIC.lock().enable();
    }
    info!("enabled apic with timer");
}

pub fn enable() {
    instructions::interrupts::enable();
    info!("enabled interrupts");
}

mod handlers {
    use core::arch::asm;

    use log::{info, warn};
    use x86_64::structures::idt::InterruptStackFrame;

    use crate::{
        print,
        qemu::{exit, ExitCode},
    };

    pub extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
        info!("breakpoint: {:?}", stack_frame);
    }

    pub extern "x86-interrupt" fn double_fault(
        stack_frame: InterruptStackFrame,
        error_code: u64,
    ) -> ! {
        let stack_pointer: *const ();
        unsafe {
            asm!("mov {}, rsp", out(reg) stack_pointer);
        }

        warn!(
            "double fault: {:?}, error code: {}; current stack ptr: {:p}",
            stack_frame, error_code, stack_pointer
        );

        exit(ExitCode::Failed)
    }

    pub extern "x86-interrupt" fn apic_timer(_: InterruptStackFrame) {
        print!(".");

        unsafe {
            super::LOCAL_APIC.lock().end_of_interrupt();
        }
    }
}
