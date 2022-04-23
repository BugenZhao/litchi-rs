pub mod broadcast;
mod executor;
pub mod serial;
pub mod time;

use self::executor::{TaskFuture, KERNEL_TASK_EXECUTOR};

pub fn init() {
    lazy_static::initialize(&KERNEL_TASK_EXECUTOR);
    KERNEL_TASK_EXECUTOR.spawn(serial::echo());
}

/// Run all of the kernel tasks until they're all pending.
/// Kernel tasks must ensure they're not waking infinitely.
pub fn poll() {
    KERNEL_TASK_EXECUTOR.poll();
}

pub fn spawn(fut: impl TaskFuture) {
    KERNEL_TASK_EXECUTOR.spawn(fut)
}
