use lazy_static::lazy_static;
use spin::Mutex;
use x2apic::lapic::{self, LocalApic};

use super::UserInterrupt;
use crate::acpi::ACPI;

lazy_static! {
    static ref LOCAL_APIC: Mutex<LocalApic> = Mutex::new(new_local_apic());
}

const TIMER_INTERVAL: u32 = 10_000_000;

fn new_local_apic() -> LocalApic {
    lapic::LocalApicBuilder::new()
        .error_vector(UserInterrupt::ApicError.as_index())
        .spurious_vector(UserInterrupt::ApicSpurious.as_index())
        .timer_vector(UserInterrupt::ApicTimer.as_index())
        .timer_initial(TIMER_INTERVAL)
        .set_xapic_base(ACPI.apic_info.local_apic_address) // or lapic::xapic_base()
        .build()
        .expect("failed to build lapic")
}

pub fn enable() {
    unsafe { LOCAL_APIC.lock().enable() };
}

pub fn end_of_interrupt() {
    unsafe { LOCAL_APIC.lock().end_of_interrupt() };
}
