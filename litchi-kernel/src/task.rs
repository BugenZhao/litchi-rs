use align_data::{include_aligned, Align4K};

mod frame;
mod manager;

pub use frame::{Registers, TaskFrame};

pub use self::manager::schedule_and_run;
pub use self::manager::with_task_manager;

static EMBEDDED_USER_BIN: &[u8] = include_aligned!(
    Align4K,
    "../../target/x86_64-unknown-litchi-user/debug/loop"
);

pub fn run() -> ! {
    with_task_manager(|task_manager| {
        task_manager.load_user("loop", EMBEDDED_USER_BIN);
    });

    schedule_and_run();
}
