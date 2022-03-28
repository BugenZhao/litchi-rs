use core::fmt::Debug;

use alloc::vec::Vec;
use size_format::SizeFormatterBinary;
use uefi::table::{boot::MemoryDescriptor, Runtime, SystemTable};
use x86_64::VirtAddr;

pub struct BootInfo {
    pub name: &'static str,

    pub kernel_entry: VirtAddr,

    pub kernel_stack_top: VirtAddr,

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
            .field("phys_offset", &self.phys_offset)
            .field("usable_memory", &UsableMemory(self.usable_memory()))
            .finish_non_exhaustive()
    }
}
