use alloc::slice;

use uefi::prelude::BootServices;
use uefi::proto::media::file::{File, FileAttribute, FileInfo, FileMode, FileType, RegularFile};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::{AllocateType, MemoryType};
use uefi::CStr16;
use x86_64::structures::paging::{PageSize, Size4KiB};

pub fn open(boot_services: &BootServices, path: impl AsRef<str>) -> RegularFile {
    let mut cstr_path_buf = [0u16; 0x50];
    let cstr_path =
        CStr16::from_str_with_buf(path.as_ref(), &mut cstr_path_buf).expect("path is not valid");

    let fs = boot_services.locate_protocol::<SimpleFileSystem>().unwrap();
    let fs = unsafe { &mut *fs.get() };

    let mut dir = fs.open_volume().expect("failed to open volume");
    let file = dir
        .open(cstr_path, FileMode::Read, FileAttribute::SYSTEM)
        .expect("failed to open file");

    match file.into_type().unwrap() {
        FileType::Regular(regular_file) => regular_file,
        FileType::Dir(_) => panic!("open a directory"),
    }
}

pub fn read(boot_services: &BootServices, mut file: RegularFile) -> &'static [u8] {
    let file_size = {
        let mut info_buf = [0u8; 0x100];
        let info = file
            .get_info::<FileInfo>(&mut info_buf)
            .expect("failed to get file info");
        info.file_size()
    };

    let pages = file_size.div_ceil(Size4KiB::SIZE) as usize;
    let file_mem_start = boot_services
        .allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, pages)
        .expect("failed to allocate memory for file");

    let buf = unsafe { slice::from_raw_parts_mut(file_mem_start as *mut u8, file_size as usize) };
    let loaded = file.read(buf).expect("failed to read file");
    assert_eq!(loaded, file_size as usize, "failed to read the whole file");

    buf
}
