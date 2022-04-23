mod executor;
pub mod mpsc;
pub mod serial;
mod sleep;

use self::executor::KERNEL_TASK_EXECUTOR;

pub fn init() {
    lazy_static::initialize(&KERNEL_TASK_EXECUTOR);
    KERNEL_TASK_EXECUTOR.spawn(serial::echo());
}

/// Run all of the kernel tasks until they're all pending.
/// Kernel tasks must ensure they're not waking infinitely.
pub fn poll() {
    KERNEL_TASK_EXECUTOR.poll();
}
