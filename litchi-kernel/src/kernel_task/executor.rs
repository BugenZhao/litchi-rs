use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::task::Wake;
use core::pin::Pin;
use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{Context, Poll, Waker};

use crossbeam_queue::ArrayQueue;
use futures::Future;
use lazy_static::lazy_static;
use spin::Mutex;

pub trait TaskFuture = Future<Output = ()> + 'static;
pub type BoxedTaskFuture = Pin<Box<dyn TaskFuture>>;

pub struct KernelTask {
    #[allow(unused)]
    id: u64,

    future: BoxedTaskFuture,
}

struct TaskWaker {
    id: u64,

    all_ready: Arc<ArrayQueue<u64>>,
}

impl TaskWaker {
    fn new_waker(id: u64, all_ready: Arc<ArrayQueue<u64>>) -> Waker {
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
    pub(super) static ref KERNEL_TASK_EXECUTOR: KernelTaskExecutor = KernelTaskExecutor::new();
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
        let waker = TaskWaker::new_waker(id, self.ready.clone());
        self.tasks.lock().insert(id, Some((task, waker)));
        self.ready.push(id).expect("kernel task full");
    }

    pub(super) fn poll(&self) {
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
