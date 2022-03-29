use core::arch::asm;

pub fn sys_print_hello() {
    unsafe { asm!("int 98") }
}
