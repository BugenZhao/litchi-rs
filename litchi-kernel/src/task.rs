use align_data::{include_aligned, Align4K};

mod frame;
mod manager;

pub use frame::{Registers, TaskFrame};
use paste::paste;

pub use self::manager::schedule_and_run;
pub use self::manager::{with_task_manager, TaskInfo, TaskManager};

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

// include_binary!(evil_memory_access_1);
// include_binary!(evil_memory_access_2);
// include_binary!(evil_memory_access_3);
// include_binary!(evil_memory_access_4);
// include_binary!(evil_heap);
// include_binary!(loop);
include_binary!(sleep);
include_binary!(term);

pub fn load() {
    with_task_manager(|task_manager| {
        // task_manager.load_user("evil_heap", EVIL_HEAP_BIN);
        // task_manager.load_user("evil_memory_access_1", EVIL_MEMORY_ACCESS_1_BIN);
        // task_manager.load_user("evil_memory_access_2", EVIL_MEMORY_ACCESS_2_BIN);
        // task_manager.load_user("evil_memory_access_3", EVIL_MEMORY_ACCESS_3_BIN);
        // task_manager.load_user("evil_memory_access_4", EVIL_MEMORY_ACCESS_4_BIN);
        // task_manager.load_user("loop1", LOOP_BIN);
        // task_manager.load_user("loop2", LOOP_BIN);
        // task_manager.load_user("loop3", LOOP_BIN);
        task_manager.load_user("sleep1", SLEEP_BIN);
        task_manager.load_user("sleep2", SLEEP_BIN);
        task_manager.load_user("term", TERM_BIN);
    });
}

pub fn run() -> ! {
    schedule_and_run();
}
