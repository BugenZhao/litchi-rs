use core::ops::Deref;

use lazy_static::lazy_static;
use log::info;
use x86_64::structures::idt::InterruptDescriptorTable;

use crate::gdt::DOUBLE_FAULT_IST_INDEX;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = new_idt();
}

fn new_idt() -> InterruptDescriptorTable {
    use handlers::*;

    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint);
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX);
    }

    idt
}

pub fn init() {
    IDT.load();

    info!("loaded idt at {:p}", IDT.deref());
}

mod handlers {
    use core::arch::asm;

    use log::{info, warn};
    use x86_64::structures::idt::InterruptStackFrame;

    use crate::qemu::{exit, ExitCode};

    pub extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
        info!("breakpoint: {:?}", stack_frame)
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
}
