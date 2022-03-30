use align_data::{include_aligned, Align4K};

mod frame;
mod manager;

pub use frame::{Registers, TaskFrame};

pub use self::manager::schedule_and_run;
pub use self::manager::{with_task_manager, TaskManager};

static LOOP_BIN: &[u8] = include_aligned!(
    Align4K,
    "../../target/x86_64-unknown-litchi-user/debug/loop"
);

static EVIL_HEAP_BIN: &[u8] = include_aligned!(
    Align4K,
    "../../target/x86_64-unknown-litchi-user/debug/evil_heap"
);

pub fn run() -> ! {
    with_task_manager(|task_manager| {
        task_manager.load_user("evil_heap", EVIL_HEAP_BIN);
        task_manager.load_user("loop1", LOOP_BIN);
        task_manager.load_user("loop2", LOOP_BIN);
        task_manager.load_user("loop3", LOOP_BIN);
    });

    schedule_and_run();
}
