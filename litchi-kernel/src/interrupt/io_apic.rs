use acpi::platform::interrupt::{InterruptSourceOverride, IoApic as IoApicInfo};
use alloc::vec::Vec;
use log::info;
use spin::Mutex;
use x2apic::ioapic::{IoApic, IrqFlags, IrqMode, RedirectionTableEntry};

use crate::{acpi::ACPI, interrupt::UserInterrupt};

use super::{RawUserInterrupt, IO_APIC_INTERRUPT_OFFSET};

lazy_static::lazy_static! {
    static ref IO_APICS: Mutex<IoApics> = Mutex::new(IoApics::new_and_init());
}

struct IoApicWrapper {
    inner: IoApic,

    info: &'static IoApicInfo,
}

impl IoApicWrapper {
    fn handle(&self, global_system_interrupt: u32) -> bool {
        global_system_interrupt >= self.info.global_system_interrupt_base
            && global_system_interrupt < (self.info.global_system_interrupt_base + 24)
    }
}

struct IoApics {
    io_apics: Vec<IoApicWrapper>,

    overrides: Vec<&'static InterruptSourceOverride>,
}

impl IoApics {
    fn new_and_init() -> Self {
        let apic_info = &ACPI.apic_info;

        let io_apics = apic_info
            .io_apics
            .iter()
            .map(|io_apic_info| unsafe {
                let mut io_apic = IoApic::new(io_apic_info.address as u64);
                io_apic.init(IO_APIC_INTERRUPT_OFFSET);

                IoApicWrapper {
                    inner: io_apic,
                    info: io_apic_info,
                }
            })
            .collect();

        let overrides = apic_info.interrupt_source_overrides.iter().collect();

        Self {
            io_apics,
            overrides,
        }
    }

    fn irq_to_interrupt(&self, irq: u8) -> Option<(usize, u32)> {
        let overrided = self
            .overrides
            .iter()
            .find(|o| o.isa_source == irq)
            .map(|o| o.global_system_interrupt);

        let global_system_interrupt = overrided.unwrap_or(irq as u32);

        let io_apic_index = self
            .io_apics
            .iter()
            .enumerate()
            .find(|(_i, io_apic)| io_apic.handle(global_system_interrupt))?
            .0;

        Some((io_apic_index, global_system_interrupt))
    }

    fn enable_irq(&mut self, irq: u8) -> Option<RawUserInterrupt> {
        let (io_apic_index, global_system_interrupt) = self.irq_to_interrupt(irq)?;

        let user_interrupt = global_system_interrupt as u8 + IO_APIC_INTERRUPT_OFFSET;

        let mut entry = RedirectionTableEntry::default();
        entry.set_mode(IrqMode::Fixed);
        entry.set_flags(IrqFlags::MASKED | IrqFlags::LEVEL_TRIGGERED);
        entry.set_vector(user_interrupt);
        entry.set_dest(0); // CPU 0

        let io_apic = &mut self.io_apics[io_apic_index].inner;

        unsafe {
            io_apic.set_table_entry(global_system_interrupt as u8, entry);
            io_apic.enable_irq(global_system_interrupt as u8);
        }

        info!(
            "enabled irq #{} to user interrupt {} in io apic {}",
            irq, user_interrupt, io_apic_index
        );

        Some(user_interrupt)
    }
}

pub fn enable_irqs() {
    let mut io_apics = IO_APICS.lock();

    let serial_in_raw = io_apics
        .enable_irq(4)
        .expect("failed to enable irq for serial in");
    assert_eq!(serial_in_raw, UserInterrupt::SerialIn as _);
}
