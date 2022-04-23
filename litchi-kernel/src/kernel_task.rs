mod executor;
pub mod broadcast;
pub mod serial;
pub mod time;

use self::executor::{TaskFuture, KERNEL_TASK_EXECUTOR};

pub fn init() {
    lazy_static::initialize(&KERNEL_TASK_EXECUTOR);
    KERNEL_TASK_EXECUTOR.spawn(serial::echo());
    KERNEL_TASK_EXECUTOR.spawn(time::sleep_2_example());
}

/// Run all of the kernel tasks until they're all pending.
/// Kernel tasks must ensure they're not waking infinitely.
pub fn poll() {
    KERNEL_TASK_EXECUTOR.poll();
}

pub fn spawn(fut: impl TaskFuture) {
    KERNEL_TASK_EXECUTOR.spawn(fut)
}
