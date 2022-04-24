use enum_as_inner::EnumAsInner;
use x86_64::VirtAddr;

use crate::resource::{ResourceHandle, ResourceResult};

use self::buffer::{SYSCALL_IN_BUFFER, SYSCALL_OUT_BUFFER};

pub mod buffer;

pub const SYSCALL_INTERRUPT: u8 = 114;

#[derive(Debug)]
pub enum Syscall<'a> {
    Print {
        str: &'a str,
    },
    ExtendHeap {
        top: VirtAddr,
    },
    GetTaskId,
    Yield,
    Sleep {
        slice: usize,
    },
    Open {
        path: &'a str,
    },
    Read {
        handle: ResourceHandle,
        buf: &'a mut [u8],
    },
    Halt,
    Exit,
}

#[derive(Debug, EnumAsInner)]
pub enum SyscallResponse {
    Ok,
    GetTaskId {
        task_id: u64,
    },
    Open {
        handle: ResourceResult<ResourceHandle>,
    },
    Read {
        len: ResourceResult<usize>,
    },
}

// For user
//

pub unsafe fn syscall(syscall: Syscall) -> SyscallResponse {
    SYSCALL_IN_BUFFER.lock().call(syscall);
    SYSCALL_OUT_BUFFER.lock().get_response()
}

// For kernel
//

pub unsafe fn get_syscall() -> Syscall<'static> {
    SYSCALL_IN_BUFFER.lock().get_syscall()
}

pub unsafe fn response(response: SyscallResponse) {
    SYSCALL_OUT_BUFFER.lock().response(response);
}
