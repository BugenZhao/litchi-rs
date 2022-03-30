use litchi_user_common::syscall::{get_syscall, Syscall};
use log::info;

use crate::{
    define_frame_saving_handler, print, println,
    task::{with_task_manager, TaskManager},
};

define_frame_saving_handler! { syscall, syscall_inner }
define_frame_saving_handler! { yield; apic_timer, apic_timer_inner }

fn syscall_inner() {
    let id = with_task_manager(|tm| tm.current_info().unwrap().id);
    info!("serving system call from {}", id);

    match unsafe { get_syscall() } {
        Syscall::PrintHello { name } => println!("Hello, `{}`!", name),

        Syscall::ExtendHeap { top } => with_task_manager(|tm| tm.extend_heap(top)),

        Syscall::Exit => with_task_manager(TaskManager::drop_current),
    }
}

fn apic_timer_inner() {
    print!(".");

    unsafe {
        super::LOCAL_APIC.lock().end_of_interrupt();
    }
}
