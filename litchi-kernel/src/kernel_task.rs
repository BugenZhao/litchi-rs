use core::{
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll, Waker},
};

use alloc::{boxed::Box, collections::BTreeMap, sync::Arc, task::Wake};
use crossbeam_queue::ArrayQueue;
use futures::Future;
use lazy_static::lazy_static;
use log::info;
use spin::Mutex;
use x86_64::instructions;

pub trait TaskFuture = Future<Output = ()> + 'static;
pub type BoxedTaskFuture = Pin<Box<dyn TaskFuture>>;

pub struct KernelTask {
    id: u64,

    future: BoxedTaskFuture,
}

struct TaskWaker {
    id: u64,

    all_ready: Arc<ArrayQueue<u64>>,
}

impl TaskWaker {
    fn new(id: u64, all_ready: Arc<ArrayQueue<u64>>) -> Waker {
        Arc::new(Self { id, all_ready }).into()
    }

    fn push_to_ready(&self) {
        self.all_ready.push(self.id).expect("kernel task full");
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.push_to_ready()
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.push_to_ready()
    }
}

lazy_static! {
    static ref KERNEL_TASK_EXECUTOR: KernelTaskExecutor = KernelTaskExecutor::new();
}

pub struct KernelTaskExecutor {
    next_task_id: AtomicU64,

    tasks: Mutex<BTreeMap<u64, Option<(KernelTask, Waker)>>>,

    ready: Arc<ArrayQueue<u64>>,
}

unsafe impl Send for KernelTaskExecutor {}
unsafe impl Sync for KernelTaskExecutor {}

impl KernelTaskExecutor {
    fn new() -> Self {
        Self {
            next_task_id: 0.into(),
            tasks: Default::default(),
            ready: Arc::new(ArrayQueue::new(256)),
        }
    }

    fn allocate_id(&self) -> u64 {
        self.next_task_id.fetch_add(1, Ordering::SeqCst)
    }
}

impl KernelTaskExecutor {
    pub fn spawn(&self, fut: impl TaskFuture) {
        let id = self.allocate_id();
        let task = KernelTask {
            id,
            future: Box::pin(fut),
        };
        let waker = TaskWaker::new(id, self.ready.clone());
        self.tasks.lock().insert(id, Some((task, waker)));
        self.ready.push(id).expect("kernel task full");
    }

    pub fn is_idle(&self) -> bool {
        self.ready.is_empty()
    }

    fn run(&self) {
        while let Some(id) = self.ready.pop() {
            let (mut task, waker) = {
                let Some(task_entry) = self.tasks.lock().get_mut(&id) else {
                    continue;
                };
                task_entry.take().unwrap()
            };

            let mut context = Context::from_waker(&waker);
            match task.future.as_mut().poll(&mut context) {
                Poll::Ready(()) => {
                    self.tasks.lock().remove(&id);
                }
                Poll::Pending => {
                    *self.tasks.lock().get_mut(&id).unwrap() = Some((task, waker));
                }
            }
        }
    }
}

pub fn run() -> ! {
    KERNEL_TASK_EXECUTOR.spawn(async {
        info!("example kernel task");
    });

    loop {
        KERNEL_TASK_EXECUTOR.run();
        instructions::hlt();
    }
}
