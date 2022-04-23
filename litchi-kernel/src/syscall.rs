use litchi_user_common::syscall::{Syscall, SyscallResponse};
use log::warn;

use crate::{
    kernel_task, print,
    task::{with_task_manager, TaskInfo, TaskManager},
};

pub fn handle_syscall(syscall: Syscall, task_info: TaskInfo) -> SyscallResponse {
    match syscall {
        Syscall::Print { str } => {
            let bytes = str.as_bytes();
            let legal = with_task_manager(|tm| {
                let page_table = tm.current_page_table().unwrap();
                page_table.check_user_accessible(bytes.as_ptr() as *const (), bytes.len())
            });

            if legal {
                print!("{}", str);
            } else {
                with_task_manager(|tm| {
                    let current_task = tm.current_info().unwrap().clone();
                    warn!(
                        "illegal access for printing, killed it: {:?}, bytes {:?}",
                        current_task,
                        bytes.as_ptr_range()
                    );
                    tm.drop_current();
                });
            }
            SyscallResponse::Ok
        }

        Syscall::ExtendHeap { top } => {
            with_task_manager(|tm| tm.extend_heap(top));
            SyscallResponse::Ok
        }

        Syscall::GetTaskId => SyscallResponse::GetTaskId {
            task_id: task_info.id,
        },

        Syscall::Yield => {
            with_task_manager(TaskManager::yield_current);
            SyscallResponse::Ok
        }

        Syscall::Sleep { slice } => {
            if slice != 0 {
                let task = with_task_manager(TaskManager::pend_current);
                kernel_task::spawn(async move {
                    kernel_task::time::sleep(slice).await;
                    task.resume_syscall_response(SyscallResponse::Ok)
                });
            }
            SyscallResponse::Ok
        }

        Syscall::Exit => {
            with_task_manager(TaskManager::drop_current);
            SyscallResponse::Ok
        }
    }
}
