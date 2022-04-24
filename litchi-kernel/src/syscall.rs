use alloc::{borrow::ToOwned, vec};
use litchi_user_common::{
    resource::ResourceError,
    syscall::{Syscall, SyscallResponse},
};
use log::warn;

use crate::{
    kernel_task, print, resource,
    task::{with_task_manager, TaskInfo, TaskManager},
};

pub fn check_syscall_legal(syscall: &Syscall) -> bool {
    fn str_addr(s: &str) -> (*const (), usize) {
        (s.as_ptr() as *const (), s.as_bytes().len())
    }

    let addrs = match syscall {
        Syscall::Print { str } => vec![str_addr(str)],
        Syscall::Open { path } => vec![str_addr(path)],
        _ => vec![],
    };

    with_task_manager(|tm| {
        let page_table = tm.current_page_table().unwrap();
        let illegal = addrs
            .into_iter()
            .find(|(base, len)| !page_table.check_user_accessible(*base, *len));

        if let Some(illegal) = illegal {
            let current_task = tm.current_info().unwrap().clone();
            warn!(
                "illegal access to {:?}, killed it: {:?}",
                illegal, current_task,
            );
            tm.drop_current();
            false
        } else {
            true
        }
    })
}

pub fn handle_syscall(syscall: Syscall<'static>, task_info: TaskInfo) -> SyscallResponse {
    if !check_syscall_legal(&syscall) {
        return SyscallResponse::Ok;
    }

    match syscall {
        Syscall::Print { str } => {
            print!("{}", str);
            SyscallResponse::Ok
        }

        Syscall::ExtendHeap { top } => {
            with_task_manager(|tm| tm.extend_current_heap(top));
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
                    task.resume_syscall_response(|| SyscallResponse::Ok)
                });
            }
            SyscallResponse::Ok
        }

        Syscall::Open { path } => {
            let handle = resource::open(path.to_owned())
                .map(|resource| with_task_manager(|tm| tm.add_current_resources(resource.into())));
            SyscallResponse::Open { handle }
        }

        Syscall::Read { handle, buf } => {
            match with_task_manager(|tm| tm.get_current_resource(handle)) {
                Some(resource) => {
                    let task = with_task_manager(TaskManager::pend_current);
                    kernel_task::spawn(async move {
                        match resource.read(buf.len()).await {
                            Ok(read) => task.resume_syscall_response(move || {
                                let len = read.len();
                                buf[..len].copy_from_slice(&read);
                                SyscallResponse::Read { len: Ok(len) }
                            }),
                            Err(err) => task.resume_syscall_response(|| SyscallResponse::Read {
                                len: Err(err),
                            }),
                        }
                    });
                    SyscallResponse::Ok
                }
                None => SyscallResponse::Read {
                    len: Err(ResourceError::NotExists),
                },
            }
        }

        Syscall::Halt => {
            crate::qemu::exit(crate::qemu::ExitCode::Success);
        }

        Syscall::Exit => {
            with_task_manager(TaskManager::drop_current);
            SyscallResponse::Ok
        }
    }
}
