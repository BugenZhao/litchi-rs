use log::info;
use spin::{Mutex, Once};
use x86_64::{
    registers,
    structures::paging::{OffsetPageTable, PageTable},
};

use crate::BOOT_INFO;

pub static PAGE_TABLE: Once<Mutex<OffsetPageTable>> = Once::new();

pub fn init() {
    let boot_info = BOOT_INFO.get().expect("boot info not set");
    PAGE_TABLE.call_once(|| {
        let frame = registers::control::Cr3::read().0;
        let l4_table = frame.start_address().as_u64() as *mut PageTable;
        unsafe {
            let l4_table = l4_table.as_mut().unwrap();
            Mutex::new(OffsetPageTable::new(l4_table, boot_info.phys_offset))
        }
    });

    info!("prepared page table")
}
