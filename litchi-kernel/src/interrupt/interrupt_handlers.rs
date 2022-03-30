use litchi_user_common::syscall::{self, Syscall, SyscallResponse};
use log::info;

use crate::{
    define_frame_saving_handler, print,
    task::{with_task_manager, TaskManager},
};

define_frame_saving_handler! { syscall, syscall_inner }
define_frame_saving_handler! { yield; apic_timer, apic_timer_inner }

fn syscall_inner() {
    let id = with_task_manager(|tm| tm.current_info().unwrap().id);
    info!("serving system call from {}", id);

    let response = match unsafe { syscall::get_syscall() } {
        Syscall::Print { args } => {
            print!("{}", args);
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
