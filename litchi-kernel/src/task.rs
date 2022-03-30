use align_data::{include_aligned, Align4K};

mod frame;
mod manager;

use alloc::string::String;
pub use frame::{Registers, TaskFrame};

use crate::memory::PageTableWrapper;

use self::manager::TASK_MANAGER;

#[derive(Debug)]
pub struct Task {
    pub id: u64,
    pub name: String,
    pub page_table: PageTableWrapper,
    pub frame: Option<TaskFrame>,
}

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
