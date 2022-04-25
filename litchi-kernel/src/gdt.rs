use lazy_static::lazy_static;
use log::{debug, info};
use x86_64::registers::segmentation::{Segment, SegmentSelector};
use x86_64::structures::gdt::GlobalDescriptorTable;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::{instructions, registers, VirtAddr};

#[repr(u16)]
pub enum IstIndex {
    DoubleFault = 0,
    UserInterrupt,
}

const INTERRUPT_STACK_SIZE: usize = 4096 * 5;

// Note: avoid declaring as &[u8] here.
#[repr(C, align(4096))]
struct TrapStack([u8; INTERRUPT_STACK_SIZE]);

impl TrapStack {
    const fn new() -> Self {
        Self([0; INTERRUPT_STACK_SIZE])
    }
}

static DOUBLE_FAULT_STACK: TrapStack = TrapStack::new();
static USER_INTERRUPT_STACK: TrapStack = TrapStack::new();

lazy_static! {
    static ref KERNEL_TSS: TaskStateSegment = new_kernel_tss();
}

fn new_kernel_tss() -> TaskStateSegment {
    let mut tss = TaskStateSegment::new();

    tss.interrupt_stack_table[IstIndex::DoubleFault as usize] = {
        let stack_top = DOUBLE_FAULT_STACK.0.as_ptr_range().end;
        debug!(
            "stack for double fault: {:?}",
            DOUBLE_FAULT_STACK.0.as_ptr_range()
        );
        VirtAddr::from_ptr(stack_top)
    };

    tss.interrupt_stack_table[IstIndex::UserInterrupt as usize] = {
        let stack_top = USER_INTERRUPT_STACK.0.as_ptr_range().end;
        debug!(
            "stack for user interrupt: {:?}",
            USER_INTERRUPT_STACK.0.as_ptr_range()
        );
        VirtAddr::from_ptr(stack_top)
    };

    // TODO: privilege stack table
    tss
}

pub struct GlobalDescriptorTableWrapper {
    gdt: GlobalDescriptorTable,

    pub kernel_code_selector: SegmentSelector,
    pub kernel_tss_selector: SegmentSelector,

    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
}

lazy_static! {
    pub static ref GDT: GlobalDescriptorTableWrapper = new_gdt();
}

fn new_gdt() -> GlobalDescriptorTableWrapper {
    use x86_64::structures::gdt::Descriptor;

    let mut gdt = GlobalDescriptorTable::new();

    let kernel_code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
    let user_code_selector = gdt.add_entry(Descriptor::user_code_segment());
    let user_data_selector = gdt.add_entry(Descriptor::user_data_segment());
    let kernel_tss_selector = gdt.add_entry(Descriptor::tss_segment(&KERNEL_TSS));

    GlobalDescriptorTableWrapper {
        gdt,
        kernel_code_selector,
        kernel_tss_selector,
        user_code_selector,
        user_data_selector,
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
