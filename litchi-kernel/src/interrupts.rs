use core::ops::Deref;

use lazy_static::lazy_static;
use log::info;
use spin::Mutex;
use x2apic::{
    ioapic::{IoApic, IrqFlags, IrqMode, RedirectionTableEntry},
    lapic::{self, LocalApic},
};
use x86_64::{
    instructions, set_general_handler,
    structures::{
        idt::InterruptDescriptorTable,
        paging::{Page, PageTableFlags, PhysFrame},
    },
    PhysAddr, VirtAddr,
};

use crate::{gdt::DOUBLE_FAULT_IST_INDEX, memory::KERNEL_PAGE_TABLE};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = new_idt();
}

fn new_idt() -> InterruptDescriptorTable {
    use handlers::*;

    let mut idt = InterruptDescriptorTable::new();

    // default unhandled
    set_general_handler!(&mut idt, unhandled);

    // Breakpoint
    idt.breakpoint.set_handler_fn(breakpoint);

    // Double fault
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX);
    }

    // APIC Timer
    idt[UserInterrupt::ApicTimer.as_index()].set_handler_fn(apic_timer);

    // Serial
    idt[UserInterrupt::Serial.as_index()].set_handler_fn(serial);

    idt
}

pub const USER_INTERRUPT_OFFSET: u8 = 32;
pub const IO_APIC_INTERRUPT_OFFSET: u8 = 128;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum UserInterrupt {
    ApicTimer = USER_INTERRUPT_OFFSET,
    ApicError = USER_INTERRUPT_OFFSET + 19,
    ApicSpurious = USER_INTERRUPT_OFFSET + 31,

    Serial = IO_APIC_INTERRUPT_OFFSET + 4,
}

impl UserInterrupt {
    fn as_index(self) -> usize {
        self as u8 as _
    }

    fn irq_number(self) -> u8 {
        self as u8 - IO_APIC_INTERRUPT_OFFSET
    }
}

lazy_static! {
    static ref LOCAL_APIC: Mutex<LocalApic> = Mutex::new(new_local_apic());
}

fn new_local_apic() -> LocalApic {
    unsafe {
        lapic::LocalApicBuilder::new()
            .error_vector(UserInterrupt::ApicError.as_index())
            .spurious_vector(UserInterrupt::ApicSpurious.as_index())
            .timer_vector(UserInterrupt::ApicTimer.as_index())
            .timer_initial(10_000_000 * 10)
            .set_xapic_base(lapic::xapic_base())
            .build()
            .expect("failed to build lapic")
    }
}

pub fn init() {
    IDT.load();
    info!("loaded idt at {:p}", IDT.deref());

    unsafe {
        LOCAL_APIC.lock().enable();
    }
    info!("enabled apic with timer");
}

#[allow(dead_code)]
pub fn init_io_apic() {
    const IO_APIC_BASE: VirtAddr = VirtAddr::new_truncate(0x2222_0000_0000);

    unsafe {
        let frame = PhysFrame::containing_address(PhysAddr::new(lapic::xapic_base()));
        let page = Page::containing_address(IO_APIC_BASE);
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE;

        KERNEL_PAGE_TABLE.map_to(page, frame, flags);
    }

    // Need ACPI info.
    unsafe {
        let mut io_apic = IoApic::new(IO_APIC_BASE.as_u64());
        io_apic.init(IO_APIC_INTERRUPT_OFFSET);

        let mut entry = RedirectionTableEntry::default();
        entry.set_mode(IrqMode::Fixed);
        entry.set_flags(IrqFlags::LEVEL_TRIGGERED | IrqFlags::LOW_ACTIVE | IrqFlags::MASKED);
        entry.set_dest(0); // CPU 0
        io_apic.set_table_entry(UserInterrupt::Serial.irq_number(), entry);

        io_apic.enable_irq(UserInterrupt::Serial.irq_number());
    }
}

pub fn enable() {
    instructions::interrupts::enable();
    info!("enabled interrupts");
}

mod handlers {
    use core::arch::asm;

    use log::{error, info};
    use x86_64::structures::idt::InterruptStackFrame;

    use crate::{
        print,
        qemu::{exit, ExitCode},
    };

    pub extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
        info!("breakpoint: {:?}", stack_frame);
    }

    pub extern "x86-interrupt" fn double_fault(
        stack_frame: InterruptStackFrame,
        error_code: u64,
    ) -> ! {
        let stack_pointer: *const ();
        unsafe {
            asm!("mov {}, rsp", out(reg) stack_pointer);
        }

        error!(
            "double fault: {:?}, error code: {}; current stack ptr: {:p}",
            stack_frame, error_code, stack_pointer
        );

        exit(ExitCode::Failed)
    }

    pub extern "x86-interrupt" fn apic_timer(_: InterruptStackFrame) {
        print!(".");

        unsafe {
            super::LOCAL_APIC.lock().end_of_interrupt();
        }
    }

    pub extern "x86-interrupt" fn serial(_: InterruptStackFrame) {
        print!("s");

        unsafe {
            super::LOCAL_APIC.lock().end_of_interrupt();
        }
    }

    pub fn unhandled(stack_frame: InterruptStackFrame, index: u8, error_code: Option<u64>) {
        error!(
            "unhandled interrupt {}: {:?}, error code: {:?}",
            index, stack_frame, error_code
        );
        exit(ExitCode::Failed)
    }
}
