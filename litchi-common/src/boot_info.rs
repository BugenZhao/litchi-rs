use alloc::vec::Vec;
use core::fmt::Debug;

use size_format::SizeFormatterBinary;
use uefi::table::boot::MemoryDescriptor;
use uefi::table::cfg::ACPI2_GUID;
use uefi::table::{Runtime, SystemTable};
use x86_64::structures::paging::PhysFrame;
use x86_64::{PhysAddr, VirtAddr};

pub struct BootInfo {
    pub name: &'static str,

    pub kernel_entry: VirtAddr,

    pub kernel_stack_top: VirtAddr,

    pub kernel_page_table: PhysFrame,

    pub system_table: SystemTable<Runtime>,

    pub phys_offset: VirtAddr,

    pub memory_descriptors: Vec<&'static MemoryDescriptor>,
}

// TODO: `SystemTable` should not be shared across threads
unsafe impl Sync for BootInfo {}
unsafe impl Send for BootInfo {}

impl BootInfo {
    pub fn usable_memory_ranges(&self) -> impl Iterator<Item = &MemoryDescriptor> {
        use uefi::table::boot::MemoryType;

        self.memory_descriptors
            .iter()
            .copied()
            .filter(|desc| -> bool {
                let ty = desc.ty;

                ty == MemoryType::BOOT_SERVICES_CODE
                    || ty == MemoryType::BOOT_SERVICES_DATA
                    || ty == MemoryType::CONVENTIONAL
            })
    }

    pub fn usable_memory(&self) -> usize {
        let pages = self
            .usable_memory_ranges()
            .map(|desc| desc.page_count)
            .sum::<u64>();

        (pages * 4096) as usize
    }

    pub fn acpi_rsdp_addr(&self) -> Option<PhysAddr> {
        self.system_table
            .config_table()
            .iter()
            .find(|table| table.guid == ACPI2_GUID)
            .map(|table| PhysAddr::new(table.address as u64))
    }
}

impl Debug for BootInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        struct UsableMemory(usize);

        impl Debug for UsableMemory {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{:.10}B", SizeFormatterBinary::new(self.0 as u64))
            }
        }

        f.debug_struct("BootInfo")
            .field("name", &self.name)
            .field("kernel_entry", &self.kernel_entry)
            .field("kernel_stack_top", &self.kernel_stack_top)
            .field("kernel_page_table", &self.kernel_page_table)
            .field("phys_offset", &self.phys_offset)
            .field("usable_memory", &UsableMemory(self.usable_memory()))
            .field("acpi_rsdp_addr", &self.acpi_rsdp_addr())
            .finish_non_exhaustive()
    }
}
