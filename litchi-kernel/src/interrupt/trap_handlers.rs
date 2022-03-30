use core::arch::asm;

use log::{error, info};
use x86_64::structures::idt::InterruptStackFrame;

use crate::qemu::{exit, ExitCode};

pub fn unhandled(stack_frame: InterruptStackFrame, index: u8, error_code: Option<u64>) {
    error!(
        "unhandled interrupt {}: {:?}, error code: {:?}",
        index, stack_frame, error_code
    );
    exit(ExitCode::Failed)
}

pub extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
    info!("breakpoint: {:?}", stack_frame);
}

pub extern "x86-interrupt" fn double_fault(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    let stack_pointer: *const ();
    unsafe {
        asm!("mov {}, rsp", out(reg) stack_pointer);
    }

    error!(
        "double fault: {:?}, error code: {}; current stack ptr: {:p}",
        stack_frame, error_code, stack_pointer
    );

    exit(ExitCode::Failed)
}
