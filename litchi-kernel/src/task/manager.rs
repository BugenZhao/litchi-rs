use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::string::String;
use alloc::sync::{Arc, Weak};
use core::ops::Deref;
use core::sync::atomic::{AtomicU64, Ordering};

use lazy_static::lazy_static;
use litchi_common::elf_loader::{ElfLoader, LoaderConfig};
use litchi_user_common::heap::USER_HEAP_BASE_ADDR;
use litchi_user_common::resource::ResourceHandle;
use litchi_user_common::syscall::buffer::{
    SYSCALL_BUFFER_PAGES, SYSCALL_IN_ADDR, SYSCALL_OUT_ADDR,
};
use litchi_user_common::syscall::SyscallResponse;
use log::{debug, info, trace, warn};
use spin::Mutex;
use x86_64::structures::idt::InterruptStackFrameValue;
use x86_64::structures::paging::{Page, PageSize, PageTableFlags, Size4KiB};
use x86_64::{instructions, VirtAddr};

use super::TaskFrame;
use crate::gdt::GDT;
use crate::memory::{PageTableWrapper, KERNEL_PAGE_TABLE};
use crate::resource::BoxedResource;
use crate::task::frame::Registers;
use crate::{kernel_task, BOOT_INFO};

#[derive(Debug)]
enum TaskPageTable {
    User(PageTableWrapper),
    Kernel(&'static PageTableWrapper),
}

impl Deref for TaskPageTable {
    type Target = PageTableWrapper;

    fn deref(&self) -> &Self::Target {
        match self {
            TaskPageTable::User(p) => p,
            TaskPageTable::Kernel(p) => p,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: u64,

    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Priority(u8);

impl Priority {
    const fn user() -> Self {
        Self(128)
    }

    const fn idle() -> Self {
        Self(255)
    }
}

struct PreScheduling(Box<dyn FnOnce() + Send>);

impl core::fmt::Debug for PreScheduling {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("PreScheduling").finish()
    }
}

#[derive(Debug)]
struct Task {
    info: TaskInfo,

    priority: Priority,

    heap_top: VirtAddr,

    page_table: TaskPageTable,

    frame: Option<TaskFrame>,

    resources: BTreeMap<ResourceHandle, Arc<BoxedResource>>,

    pre_schduling: Option<PreScheduling>,
}

impl Task {
    const IDLE_ID: u64 = 0;
    const USER_START_ID: u64 = 1024;

    fn idle() -> Self {
        fn idle() -> ! {
            loop {
                instructions::hlt();
            }
        }

        let segment = GDT.kernel_code_selector.0 as u64;

        let frame = TaskFrame {
            es: segment,
            ds: segment,
            regs: Registers::default(),
            frame: InterruptStackFrameValue {
                instruction_pointer: VirtAddr::from_ptr(idle as *const fn() -> !),
                code_segment: segment,
                cpu_flags: 0x0000_0200, // enable interrupts
                stack_pointer: BOOT_INFO.get().unwrap().kernel_stack_top,
                stack_segment: 0,
            },
        };

        Self {
            info: TaskInfo {
                id: Self::IDLE_ID,
                name: "idle".to_owned(),
            },
            priority: Priority::idle(),
            heap_top: VirtAddr::zero(), // unused
            page_table: TaskPageTable::Kernel(&KERNEL_PAGE_TABLE),
            frame: Some(frame),
            resources: Default::default(),
            pre_schduling: None,
        }
    }
}

struct PendingTaskToken;

pub struct PendingTaskHandle {
    id: u64,

    _token: Arc<PendingTaskToken>,
}

impl PendingTaskHandle {
    /// Resume this task and lazily call the closure to get the syscall response on next scheduling.
    pub fn resume_syscall_response(
        self,
        response: impl FnOnce() -> SyscallResponse + Send + 'static,
    ) {
        with_task_manager(|tm| {
            tm.resume_task(self, move || {
                let response = response();
                unsafe { litchi_user_common::syscall::response(response) };
            })
        })
    }
}

lazy_static! {
    static ref TASK_MANAGER: Mutex<TaskManager> = Mutex::new(TaskManager::new());
}

pub struct TaskManager {
    next_task_id: AtomicU64,

    running: Option<Task>,

    ready: BTreeMap<Priority, VecDeque<Task>>,

    pending: BTreeMap<u64, (Task, Weak<PendingTaskToken>)>,
}

impl TaskManager {
    fn new() -> Self {
        let mut tm = Self {
            next_task_id: Task::USER_START_ID.into(),
            running: None,
            ready: Default::default(),
            pending: Default::default(),
        };
        tm.add_to_ready(Task::idle());
        tm
    }

    fn allocate_id(&self) -> u64 {
        self.next_task_id.fetch_add(1, Ordering::SeqCst)
    }
}

impl TaskManager {
    fn add_to_ready(&mut self, task: Task) {
        self.ready.entry(task.priority).or_default().push_back(task);
    }

    fn take_one_ready(&mut self) -> Task {
        self.ready
            .values_mut()
            .find(|q| !q.is_empty())
            .expect("there should be always an idle task")
            .pop_front()
            .unwrap()
    }

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
                stack_pointer: USER_STACK_TOP,
                stack_segment: data_segment,
            },
        };

        let task = Task {
            info: TaskInfo {
                id: self.allocate_id(),
                name,
            },
            priority: Priority::user(),
            heap_top: USER_HEAP_BASE_ADDR,
            page_table: TaskPageTable::User(page_table),
            frame: Some(frame),
            resources: Default::default(),
            pre_schduling: None,
        };

        info!("new task: {:?}", task);
        self.add_to_ready(task);
    }

    fn cleanup_zombies(&mut self) {
        self.pending.retain(|_, (task, token)| {
            let zombie = token.strong_count() == 0;
            if zombie {
                warn!("zombie task: {:?}", task.info);
            }
            !zombie
        });
    }

    fn schedule(&mut self) -> TaskFrame {
        self.cleanup_zombies();

        if self.running.is_none() {
            let task = self.take_one_ready();

            task.page_table.load();
            debug!("loaded page table: {:?}", task.page_table);

            self.running = Some(task);
        }

        let task = self.running.as_mut().unwrap();
        assert!(task.page_table.is_current());

        debug!("scheduled: {:?}", task.info);
        trace!("scheduled: {:?}", task);

        // Run pre scheduling callback. For example, syscall response after pending.
        if let Some(f) = task.pre_schduling.take() {
            (f.0)();
        }

        task.frame.take().expect("no frame for task")
    }

    /// Put back the task frame for the current running task. Used everytime coming from the task by
    /// interrupts.
    ///
    /// For task preemption based on the timer, the `yield_task` will be true, which means to put
    /// the running task to the back of the ready queue. For others like serial interrupt or system
    /// calls, we may want to preserve the time slice of this task, so `yield_task` will be false
    /// and we'll keep this task running on next scheduling.
    pub fn put_back(&mut self, frame: TaskFrame, yield_task: bool) {
        let task = self.running.as_mut().expect("no task running");

        if !frame.is_user() {
            assert_eq!(task.info.id, Task::IDLE_ID);
        }

        let old_frame = task.frame.replace(frame);
        assert!(old_frame.is_none(), "task frame exists");

        debug!(
            "returned from task: {:?}, yield = {}",
            task.info, yield_task
        );
        trace!("returned from task: {:?}, yield = {}", task, yield_task);

        if yield_task {
            self.yield_current();
        }
    }

    /// Put the current running task to the back of the ready queue.
    pub fn yield_current(&mut self) {
        if self.ready.is_empty() {
            debug!("empty ready queue, no need to yield");
        } else {
            let task = self.running.take().unwrap();
            self.add_to_ready(task);
        }
    }

    /// Drop the current running task. Based on the RAII, all of the other resources will be
    /// released as well.
    pub fn drop_current(&mut self) {
        KERNEL_PAGE_TABLE.load();

        let task = self.running.take().expect("no task running");
        info!("dropped current task: {:?}", task.info);
    }

    /// Pend the current running task by putting it to the pending queue.
    ///
    /// Returns a [`PendingTaskHandle`] which can be used to resume the task. If the caller dropped
    /// the handle instead of resuming the task, The task manager will find it on next scheduling
    /// and clean-up the resources by killing the zombie task.
    pub fn pend_current(&mut self) -> PendingTaskHandle {
        KERNEL_PAGE_TABLE.load();

        let task = self.running.take().expect("no task running");
        let id = task.info.id;
        assert!(task.frame.is_some(), "empty frame while pending task");

        let token = Arc::new(PendingTaskToken);
        let weak_token = Arc::downgrade(&token);

        self.pending.insert(id, (task, weak_token));
        PendingTaskHandle { id, _token: token }
    }

    /// Resume the given task by transfering it from the pending task queue to the ready queue.
    ///
    /// The given `pre_scheduling` closure will be saved to the task frame and be called RIGHT
    /// BEFORE this task will be scheduled and AFTER the page table is loaded, since it may rely on
    /// the memory space of this task. For example, we can copy the kernel buffer to the user's and
    /// place the syscall response.
    pub fn resume_task(
        &mut self,
        task_handle: PendingTaskHandle,
        pre_scheduling: impl FnOnce() + Send + 'static,
    ) {
        let id = task_handle.id;
        let mut task = self
            .pending
            .remove(&id)
            .unwrap_or_else(|| panic!("no pending task {id}"))
            .0;
        task.pre_schduling = Some(PreScheduling(Box::new(pre_scheduling)));

        self.add_to_ready(task);
    }

    pub fn extend_current_heap(&mut self, top: VirtAddr) {
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
            info!(
                "extend heap to {:?} for task {}",
                task.heap_top, task.info.id
            );
        }
    }

    pub fn add_current_resources(&mut self, resource: Arc<BoxedResource>) -> ResourceHandle {
        let task = self.running.as_mut().expect("no task running");
        let map = &mut task.resources;
        let new_handle = map
            .keys()
            .last()
            .copied()
            .map(|h| ResourceHandle(h.0 + 1))
            .unwrap_or_default();
        map.insert(new_handle, resource);
        new_handle
    }

    pub fn get_current_resource(&self, handle: ResourceHandle) -> Option<Arc<BoxedResource>> {
        let task = self.running.as_ref().expect("no task running");
        task.resources.get(&handle).cloned()
    }

    pub fn has_running(&self) -> bool {
        self.running.is_some()
    }

    pub fn current_info(&self) -> Option<&TaskInfo> {
        self.running.as_ref().map(|task| &task.info)
    }

    pub fn current_page_table(&self) -> Option<&PageTableWrapper> {
        self.running.as_ref().map(|task| task.page_table.deref())
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
    kernel_task::poll(); // Poll the kernel task first
    let task_frame = with_task_manager(TaskManager::schedule);
    unsafe { task_frame.pop() }
}
