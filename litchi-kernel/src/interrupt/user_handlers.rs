use litchi_user_common::syscall;
use log::{debug, error, warn};
use x86_64::{
    registers::segmentation::SegmentSelector,
    structures::idt::{InterruptStackFrame, PageFaultErrorCode},
    PrivilegeLevel,
};

use crate::{
    define_frame_saving_handler,
    interrupt::local_apic::end_of_interrupt,
    kernel_task, print,
    qemu::{exit, ExitCode},
    serial_log::DEBUG_SERIAL,
    syscall::handle_syscall,
    task::{schedule_and_run, with_task_manager},
};

define_frame_saving_handler! { syscall, syscall_inner }
define_frame_saving_handler! { yield; apic_timer, apic_timer_inner }
define_frame_saving_handler! { serial_in, serial_in_inner }

fn syscall_inner() {
    let info = with_task_manager(|tm| tm.current_info().cloned().unwrap());
    debug!("serving system call from {}", info.id);

    let response = handle_syscall(unsafe { syscall::get_syscall() }, info);

    // Maybe we've killed current task.
    if with_task_manager(|tm| tm.has_running()) {
        unsafe { syscall::response(response) };
    }
}

fn apic_timer_inner() {
    kernel_task::time::inc_slice();
    print!(".");

    end_of_interrupt();
}

fn serial_in_inner() {
    let byte = DEBUG_SERIAL.lock().receive();
    let ch = char::from_u32(byte as u32).unwrap_or('?');
    kernel_task::serial::push(ch);

    end_of_interrupt();
}

pub extern "x86-interrupt" fn page_fault(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    // It's okay that we're not saving the task frame, since we gonna kill it.

    let pl = SegmentSelector(stack_frame.code_segment as u16).rpl();
    if pl == PrivilegeLevel::Ring0 {
        error!(
            "kernel page fault: frame {:?}, error code: {:?}",
            stack_frame, error_code
        );
        exit(ExitCode::Failed)
    }

    with_task_manager(|tm| {
        let current_task = tm.current_info().unwrap().clone();
        warn!(
            "task page fault, kill it: {:?}, frame {:?}, error code: {:?}",
            current_task, stack_frame, error_code
        );
        tm.drop_current();
    });

    schedule_and_run();
}
