use core::{
    pin::Pin,
    task::{Context, Poll},
};

use alloc::{collections::VecDeque, sync::Arc};
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

pub struct Sender<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Sender<T> {
    pub fn send(&self, item: T) {
        self.inner.buffer.lock().push_back(item);
        self.inner.waker.wake();
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
            return Poll::Ready(Some(item));
        }

        self.inner.waker.register(cx.waker());
        match buffer.pop_front() {
            Some(item) => {
                self.inner.waker.take();
                Poll::Ready(Some(item))
            }
            None => Poll::Pending,
        }
    }
}

pub fn mpsc_channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new(Inner::default());
    let tx = Sender {
        inner: inner.clone(),
    };
    let rx = Receiver { inner };
    (tx, rx)
}
