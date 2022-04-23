use core::{
    iter::once,
    pin::Pin,
    task::{Context, Poll},
};

use alloc::{
    collections::{LinkedList, VecDeque},
    sync::{Arc, Weak},
};
use futures::{task::AtomicWaker, Stream};
use spin::Mutex;

struct Inner<T> {
    buffer: Mutex<VecDeque<T>>,

    waker: AtomicWaker,
}

impl<T> Default for Inner<T> {
    fn default() -> Self {
        Self {
            buffer: Default::default(),
            waker: AtomicWaker::new(),
        }
    }
}

impl<T> Inner<T> {
    fn push_and_wake(&self, items: impl IntoIterator<Item = T>) {
        self.buffer.lock().extend(items);
        self.waker.wake();
    }
}

pub struct Sender<T> {
    inners: Mutex<LinkedList<Weak<Inner<T>>>>,
}

impl<T> Sender<T> {
    pub fn new() -> Self {
        Self {
            inners: Default::default(),
        }
    }

    pub fn send_one(&self, item: T) {
        let mut inners = self.inners.lock();
        inners.drain_filter(|inner| inner.strong_count() == 0);

        assert!(inners.len() <= 1);
        if let Some(inner) = inners.iter().next().and_then(|i| i.upgrade()) {
            inner.push_and_wake(once(item));
        }
    }

    pub fn subscribe(&self) -> Receiver<T> {
        let inner = Arc::new(Inner::default());
        self.inners.lock().push_back(Arc::downgrade(&inner));
        Receiver { inner }
    }
}

impl<T> Sender<T>
where
    T: Clone,
{
    pub fn send_all(&self, item: T) {
        let mut inners = self.inners.lock();
        inners.drain_filter(|inner| inner.strong_count() == 0);

        for inner in inners.iter() {
            let inner = inner.upgrade().unwrap();
            inner.push_and_wake(once(item.clone()));
        }
    }
}

pub struct Receiver<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buffer = self.inner.buffer.lock();

        if let Some(item) = buffer.pop_front() {
            Poll::Ready(Some(item))
        } else {
            self.inner.waker.register(cx.waker());
            Poll::Pending
        }
    }
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let tx = Sender::new();
    let rx = tx.subscribe();
    (tx, rx)
}
