use align_data::{include_aligned, Align4K};

mod frame;
mod manager;

pub use frame::{Registers, TaskFrame};
use paste::paste;

pub use self::manager::schedule_and_run;
pub use self::manager::{with_task_manager, TaskManager};

macro_rules! include_binary {
    ($name: ident) => {
        paste! {
            static [<$name:upper _BIN>]: &[u8] = include_aligned!(
                Align4K,
                concat!("../../target/x86_64-unknown-litchi-user/debug/", stringify!($name), ".lit")
            );
        }
    };
}

include_binary!(evil_memory_access);
include_binary!(evil_heap);
include_binary!(loop);

pub fn run() -> ! {
    with_task_manager(|task_manager| {
        task_manager.load_user("evil_heap", EVIL_HEAP_BIN);
        task_manager.load_user("loop1", LOOP_BIN);
        task_manager.load_user("loop2", LOOP_BIN);
        task_manager.load_user("loop3", LOOP_BIN);
        task_manager.load_user("evil_memory_access", EVIL_MEMORY_ACCESS_BIN);
    });

    schedule_and_run();
}
