#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

use litchi_user::syscall::sys_extend_heap;
use litchi_user_common::heap::USER_HEAP_BASE_ADDR;

#[no_mangle]
extern "C" fn main() {
    sys_extend_heap(USER_HEAP_BASE_ADDR + 0x0100_0000_0000u64);

    unreachable!("we should be killed");
}
