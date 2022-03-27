use lazy_static::lazy_static;
use log::{debug, info};
use x86_64::{
    instructions,
    registers::{
        self,
        segmentation::{Segment, SegmentSelector},
    },
    structures::{gdt::GlobalDescriptorTable, tss::TaskStateSegment},
    VirtAddr,
};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
const INTERRUPT_STACK_SIZE: usize = 4096 * 5;

lazy_static! {
    static ref KERNEL_TSS: TaskStateSegment = new_kernel_tss();
}

fn new_kernel_tss() -> TaskStateSegment {
    let mut tss = TaskStateSegment::new();

    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
        static STACK: &[u8] = &[0; INTERRUPT_STACK_SIZE];
        let stack_top = STACK.as_ptr_range().end;

        debug!("stack for double fault: {:?}", STACK.as_ptr_range());

        VirtAddr::from_ptr(stack_top)
    };

    // TODO: privilege stack table
    tss
}

struct GlobalDescriptorTableWrapper {
    gdt: GlobalDescriptorTable,

    kernel_code_selector: SegmentSelector,
    kernel_tss_selector: SegmentSelector,
}

lazy_static! {
    static ref GDT: GlobalDescriptorTableWrapper = new_gdt();
}

fn new_gdt() -> GlobalDescriptorTableWrapper {
    use x86_64::structures::gdt::Descriptor;

    let mut gdt = GlobalDescriptorTable::new();

    let kernel_code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
    let kernel_tss_selector = gdt.add_entry(Descriptor::tss_segment(&KERNEL_TSS));

    GlobalDescriptorTableWrapper {
        gdt,
        kernel_code_selector,
        kernel_tss_selector,
    }
}

pub fn init() {
    GDT.gdt.load();

    unsafe {
        registers::segmentation::CS::set_reg(GDT.kernel_code_selector);
        registers::segmentation::SS::set_reg(SegmentSelector(0)); // important
        instructions::tables::load_tss(GDT.kernel_tss_selector);
    }

    info!("loaded gdt at {:p} and kernel tss", &GDT.gdt)
}
