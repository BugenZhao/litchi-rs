use core::{
    pin::Pin,
    task::{Context, Poll},
};

use alloc::collections::VecDeque;
use futures::{task::AtomicWaker, Stream};
use futures_async_stream::for_await;
use spin::Mutex;

use crate::print;

lazy_static::lazy_static! {
    pub static ref SERIAL_STREAM: SerialStream = SerialStream::default();
}

#[derive(Default)]
pub struct SerialStream {
    buffer: Mutex<VecDeque<char>>,

    waker: AtomicWaker,
}

impl SerialStream {
    pub fn push(&self, ch: char) {
        self.buffer.lock().push_back(ch);
        self.waker.wake();
    }
}

impl Stream for &'static SerialStream {
    type Item = char;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buffer = self.buffer.lock();
        if let Some(ch) = buffer.pop_front() {
            return Poll::Ready(Some(ch));
        }

        self.waker.register(cx.waker());
        match buffer.pop_front() {
            Some(ch) => {
                self.waker.take();
                Poll::Ready(Some(ch))
            }
            None => Poll::Pending,
        }
    }
}

pub(super) async fn echo() {
    #[for_await]
    for ch in &*SERIAL_STREAM {
        print!("{ch}")
    }
}
