use log::info;

use crate::{define_frame_saving_handler, print, task::with_task_manager};

define_frame_saving_handler! { print_hello, print_hello_inner }
define_frame_saving_handler! { yield; apic_timer, apic_timer_inner }

fn print_hello_inner() {
    let id = with_task_manager(|task_manager| task_manager.current_info().unwrap().id);
    info!("Hello from user task {}.", id);
}

fn apic_timer_inner() {
    print!(".");

    unsafe {
        super::LOCAL_APIC.lock().end_of_interrupt();
    }
}
