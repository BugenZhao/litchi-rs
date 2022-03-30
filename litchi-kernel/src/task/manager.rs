use core::sync::atomic::{AtomicU64, Ordering};

use alloc::{collections::VecDeque, string::String};
use lazy_static::lazy_static;
use litchi_common::elf_loader::{ElfLoader, LoaderConfig};
use litchi_user_common::{
    heap::USER_HEAP_BASE_ADDR,
    syscall::{SYSCALL_BUFFER_PAGES, SYSCALL_IN_ADDR, SYSCALL_OUT_ADDR},
};
use log::{debug, info, warn};
use spin::Mutex;
use x86_64::{
    instructions,
    structures::{
        idt::InterruptStackFrameValue,
        paging::{Page, PageSize, PageTableFlags, Size4KiB},
    },
    VirtAddr,
};

use crate::{
    gdt::GDT,
    memory::PageTableWrapper,
    qemu::{exit, ExitCode},
    task::frame::Registers,
};

use super::TaskFrame;

#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: u64,

    pub name: String,
}

#[derive(Debug)]
struct Task {
    info: TaskInfo,

    heap_top: VirtAddr,

    page_table: PageTableWrapper,

    frame: Option<TaskFrame>,
}

lazy_static! {
    static ref TASK_MANAGER: Mutex<TaskManager> = Mutex::new(TaskManager::new());
}

pub struct TaskManager {
    next_task_id: AtomicU64,

    running: Option<Task>,

    ready: VecDeque<Task>,
}

impl TaskManager {
    fn new() -> Self {
        Self {
            next_task_id: 1024.into(),
            running: None,
            ready: Default::default(),
        }
    }

    fn allocate_id(&self) -> u64 {
        self.next_task_id.fetch_add(1, Ordering::SeqCst)
    }
}

impl TaskManager {
    pub fn load_user(&mut self, name: impl Into<String>, elf_bytes: &'static [u8]) {
        const USER_STACK_TOP: VirtAddr = VirtAddr::new_truncate(0x1889_0000_0000);
        const USER_STACK_PAGES: u64 = 10;

        let name = name.into();

        let page_table = PageTableWrapper::new_user();
        let loader_config = LoaderConfig {
            stack_top: USER_STACK_TOP,
            stack_pages: USER_STACK_PAGES,
            userspace: true,
        };

        let entry_point = page_table.with_allocator(|frame_allocator, page_table| {
            ElfLoader::new(&loader_config, elf_bytes, frame_allocator, page_table).load()
        });
        info!(
            "loaded user binary `{}`, entry point {:p}",
            name, entry_point
        );

        // Map syscall buffer.
        unsafe {
            let flags = PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::USER_ACCESSIBLE
                | PageTableFlags::NO_EXECUTE;

            for base_addr in [SYSCALL_IN_ADDR, SYSCALL_OUT_ADDR] {
                let base_page = Page::from_start_address(base_addr).unwrap();
                for page in (0..SYSCALL_BUFFER_PAGES).map(|i| base_page + i) {
                    page_table
                        .allocate_and_map_to(page, flags)
                        .expect("no enough memory");
                }
            }
        }

        let code_segment = GDT.user_code_selector.0 as u64;
        let data_segment = GDT.user_data_selector.0 as u64;

        let frame = TaskFrame {
            es: data_segment,
            ds: data_segment,
            regs: Registers::default(),
            frame: InterruptStackFrameValue {
                instruction_pointer: VirtAddr::from_ptr(entry_point),
                code_segment,
                cpu_flags: 0x0000_0200, // enable interrupts
                // cpu_flags: 0,
                stack_pointer: USER_STACK_TOP,
                stack_segment: data_segment,
            },
        };

        let task = Task {
            info: TaskInfo {
                id: self.allocate_id(),
                name,
            },
            heap_top: USER_HEAP_BASE_ADDR,
            page_table,
            frame: Some(frame),
        };

        info!("new task: {:?}", task);
        self.ready.push_back(task);
    }

    pub fn schedule(&mut self) -> TaskFrame {
        if self.running.is_none() {
            if let Some(task) = self.ready.pop_front() {
                task.page_table.load();
                debug!("loaded page table: {:?}", task.page_table);

                self.running = Some(task);
            } else {
                info!("no task to schedule");
                exit(ExitCode::Success);
            }
        }

        let task = self.running.as_mut().unwrap();
        assert!(task.page_table.is_current());

        info!("scheduled: {:?}", task.info);
        debug!("scheduled: {:?}", task);

        task.frame.take().expect("no frame for task")
    }

    pub fn put_back(&mut self, frame: TaskFrame, yield_task: bool) {
        if !frame.is_user() {
            debug!("frame not from user, ignored");
            return;
        }

        let task = self.running.as_mut().expect("no task running");
        let old_frame = task.frame.replace(frame);
        assert!(old_frame.is_none(), "task frame exists");

        info!(
            "returned from user: {:?}, yield = {}",
            task.info, yield_task
        );
        debug!("returned from user: {:?}, yield = {}", task, yield_task);

        if yield_task {
            if self.ready.is_empty() {
                debug!("empty ready queue, no need to yield");
            } else {
                let task = self.running.take().unwrap();
                self.ready.push_back(task);
            }
        }
    }

    pub fn drop_current(&mut self) {
        let task = self.running.take().expect("no task running");

        info!("dropped current task: {:?}", task.info);
    }

    pub fn extend_heap(&mut self, top: VirtAddr) {
        let top = top.align_up(Size4KiB::SIZE);
        let task = self.running.as_mut().expect("no task running");

        let mut success = true;

        if top >= task.heap_top {
            let base_page = Page::from_start_address(task.heap_top).unwrap();
            let top_page = Page::from_start_address(top).unwrap();

            let flags = PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::USER_ACCESSIBLE
                | PageTableFlags::NO_EXECUTE;

            unsafe {
                for page in Page::range(base_page, top_page) {
                    if task.page_table.allocate_and_map_to(page, flags).is_none() {
                        success = false;
                        break;
                    }
                }
            }
        }

        if !success {
            warn!(
                "no enough memory to extend heap to {:?} for task {}, kill it",
                top, task.info.id
            );
            self.drop_current();
        } else {
            task.heap_top = top;
            info!("extend heap to {:?} for task {}", task.heap_top, task.info.id);
        }
    }

    pub fn current_info(&self) -> Option<&TaskInfo> {
        self.running.as_ref().map(|task| &task.info)
    }
}

pub fn with_task_manager<F, R>(f: F) -> R
where
    F: FnOnce(&mut TaskManager) -> R,
{
    instructions::interrupts::without_interrupts(|| {
        let mut task_manager = TASK_MANAGER.lock();
        f(&mut *task_manager)
    })
}

pub fn schedule_and_run() -> ! {
    let task_frame = with_task_manager(TaskManager::schedule);
    unsafe { task_frame.pop() }
}
