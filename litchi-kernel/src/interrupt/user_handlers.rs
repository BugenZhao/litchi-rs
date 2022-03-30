use litchi_user_common::syscall::{self, Syscall, SyscallResponse};
use log::{error, info, warn};
use x86_64::{
    registers::segmentation::SegmentSelector,
    structures::idt::{InterruptStackFrame, PageFaultErrorCode},
    PrivilegeLevel,
};

use crate::{
    define_frame_saving_handler, print,
    qemu::{exit, ExitCode},
    task::{schedule_and_run, with_task_manager, TaskManager},
};

define_frame_saving_handler! { syscall, syscall_inner }
define_frame_saving_handler! { yield; apic_timer, apic_timer_inner }

fn syscall_inner() {
    let id = with_task_manager(|tm| tm.current_info().unwrap().id);
    info!("serving system call from {}", id);

    let response = match unsafe { syscall::get_syscall() } {
        Syscall::Print { str } => {
            let bytes = str.as_bytes();
            let legal = with_task_manager(|tm| {
                let page_table = tm.current_page_table().unwrap();
                page_table.check_user_accessible(bytes.as_ptr() as *const (), bytes.len())
            });

            if legal {
                print!("{}", str);
            } else {
                with_task_manager(|tm| {
                    let current_task = tm.current_info().unwrap().clone();
                    warn!(
                        "illegal access for printing, killed it: {:?}, bytes {:?}",
                        current_task,
                        bytes.as_ptr_range()
                    );
                    tm.drop_current();
                });
            }
            SyscallResponse::Ok
        }

        Syscall::ExtendHeap { top } => {
            with_task_manager(|tm| tm.extend_heap(top));
            SyscallResponse::Ok
        }

        Syscall::GetTaskId => SyscallResponse::GetTaskId { task_id: id },

        Syscall::Yield => {
            with_task_manager(TaskManager::yield_current);
            SyscallResponse::Ok
        }

        Syscall::Exit => {
            with_task_manager(TaskManager::drop_current);
            SyscallResponse::Ok
        }
    };

    // Maybe we've killed current task.
    if with_task_manager(|tm| tm.has_running()) {
        unsafe { syscall::response(response) };
    }
}

fn apic_timer_inner() {
    print!(".");

    unsafe {
        super::LOCAL_APIC.lock().end_of_interrupt();
    }
}

pub extern "x86-interrupt" fn page_fault(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    // We're not saving the task frame for this, since we're gonna kill it.

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
