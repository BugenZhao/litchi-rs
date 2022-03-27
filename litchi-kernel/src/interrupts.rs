use lazy_static::lazy_static;
use log::info;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = new_idt();
}

fn new_idt() -> InterruptDescriptorTable {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint);

    idt
}

pub fn init() {
    IDT.load()
}

extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
    info!("breakpoint: {:?}", stack_frame)
}
