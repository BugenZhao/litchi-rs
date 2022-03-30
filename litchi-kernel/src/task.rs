use align_data::{include_aligned, Align4K};

mod frame;
mod task;

pub use frame::{Registers, TaskFrame};

use self::task::TASK_MANAGER;

static EMBEDDED_USER_BIN: &[u8] = include_aligned!(
    Align4K,
    "../../target/x86_64-unknown-litchi-user/debug/loop"
);

pub fn run() {
    let frame = {
        let mut task_manager = TASK_MANAGER.lock();
        task_manager.load_user("loop", EMBEDDED_USER_BIN);
        task_manager.schedule()
    };

    unsafe { frame.pop() }
}
