// https://wiki.osdev.org/MADT
// https://uefi.org/specs/ACPI/6.4/05_ACPI_Software_Programming_Model/ACPI_Software_Programming_Model.html#finding-the-rsdp-on-uefi-enabled-systems

use alloc::vec::Vec;
use core::ptr::NonNull;

use acpi::platform::interrupt::Apic;
use acpi::platform::{Processor, ProcessorInfo};
use acpi::{AcpiHandler, AcpiTables};
use lazy_static::lazy_static;
use log::info;

use crate::BOOT_INFO;

#[derive(Clone)]
struct OffsetAcpiHandler;

impl AcpiHandler for OffsetAcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Self, T> {
        acpi::PhysicalMapping::new(
            physical_address,
            NonNull::new_unchecked(physical_address as *mut _),
            size,
            size,
            self.clone(),
        )
    }

    fn unmap_physical_region<T>(_region: &acpi::PhysicalMapping<Self, T>) {}
}

#[derive(Debug)]
pub struct Processors {
    pub boot: Processor,

    pub applications: Vec<Processor>,
}

#[derive(Debug)]
pub struct Acpi {
    pub apic_info: Apic,

    pub processor_info: Processors,
}

impl Acpi {
    fn new() -> Self {
        let boot_info = BOOT_INFO.get().unwrap();
        let acpi_rsdp_addr = boot_info
            .acpi_rsdp_addr()
            .expect("failed to locate acpi rsdp address");

        let acpi_tables = unsafe {
            AcpiTables::from_rsdp(OffsetAcpiHandler, acpi_rsdp_addr.as_u64() as usize)
                .expect("failed to validate acpi rsdp tables")
        };

        let platform_info = acpi_tables
            .platform_info()
            .expect("failed to get platform info");

        let apic_info = match platform_info.interrupt_model {
            acpi::InterruptModel::Apic(apic_info) => apic_info,
            _ => panic!("no apic in this system"),
        };

        let processor_info = {
            let ProcessorInfo {
                boot_processor,
                application_processors,
            } = platform_info.processor_info.expect("no processor info");
            Processors {
                boot: boot_processor,
                applications: application_processors,
            }
        };

        Self {
            apic_info,
            processor_info,
        }
    }
}

lazy_static! {
    pub static ref ACPI: Acpi = Acpi::new();
}

pub fn init() {
    lazy_static::initialize(&ACPI);

    info!("initialized acpi info: {:x?}", *ACPI);
}
