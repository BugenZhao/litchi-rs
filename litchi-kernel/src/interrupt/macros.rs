#[macro_export]
macro_rules! define_frame_saving_handler {
    (yield; $handler_name: ident, $handler_inner: ident) => {
        define_frame_saving_handler!(true, $handler_name, $handler_inner);
    };
    ($handler_name: ident, $handler_inner: ident) => {
        define_frame_saving_handler!(false, $handler_name, $handler_inner);
    };

    ($yield: expr, $handler_name: ident, $handler_inner: ident) => {
        #[naked]
        /// Note: With `naked`, this function is exactly a naked procedure without any abi.
        /// The `x86-interrupt` is just for type checking of setting handler with `x86_64` crate.
        pub extern "x86-interrupt" fn $handler_name(frame: x86_64::structures::idt::InterruptStackFrame) {
            use core::arch::asm;
            use x86_64::registers::{self, segmentation::Segment};
            use $crate::task::{schedule_and_run, with_task_manager, TaskFrame};

            extern "C" fn _frame_saving_inner(mut frame: TaskFrame) {
                assert!(! x86_64::instructions::interrupts::are_enabled());

                frame.ds = registers::segmentation::DS::get_reg().0 as u64;
                frame.es = registers::segmentation::ES::get_reg().0 as u64;

                // Put the task frame back to the `Task` struct of the running task.
                with_task_manager(|task_manager| task_manager.put_back(frame, $yield));
                // Then run the given handler inner.
                let _ = $handler_inner();
                // After that, we schedule a next task to run. This function never returns.
                schedule_and_run();
            }

            unsafe {
                asm!(
                    // The order must be consistent with [`TaskFrame`].
                    //
                    // I've no idea about how to saving ds & es with inline assembly...
                    // So let's just make two placeholders and save them with Rust.
                    "mov    qword ptr [rsp - 136], 0", // Placeholder for es
                    "mov    qword ptr [rsp - 128], 0", // Placeholder for ds
                    "mov    qword ptr [rsp - 120], r15",
                    "mov    qword ptr [rsp - 112], r14",
                    "mov    qword ptr [rsp - 104], r13",
                    "mov    qword ptr [rsp - 96], r12",
                    "mov    qword ptr [rsp - 88], r11",
                    "mov    qword ptr [rsp - 80], r10",
                    "mov    qword ptr [rsp - 72], r9",
                    "mov    qword ptr [rsp - 64], r8",
                    "mov    qword ptr [rsp - 56], rsi",
                    "mov    qword ptr [rsp - 48], rdi",
                    "mov    qword ptr [rsp - 40], rbp",
                    "mov    qword ptr [rsp - 32], rdx",
                    "mov    qword ptr [rsp - 24], rcx",
                    "mov    qword ptr [rsp - 16], rbx",
                    "mov    qword ptr [rsp - 8],  rax",
                    "lea    rdi, [rsp - 136]",
                    "sub    rsp, 136",
                    "call   {}",
                    sym _frame_saving_inner,
                    options(noreturn)
                )
            }
        }
    };
}
