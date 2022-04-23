use futures_async_stream::for_await;
use spin::Mutex;

use crate::{kernel_task::mpsc, print};

lazy_static::lazy_static! {
    static ref CHANNEL: (mpsc::Sender<char>, Mutex<Option<mpsc::Receiver<char>>>) = {
        let (tx, rx) = mpsc::mpsc_channel();
        (tx, Mutex::new(Some(rx)))
    };
}

pub fn push(ch: char) {
    CHANNEL.0.send(ch);
}

pub(super) async fn echo() {
    let rx = CHANNEL.1.lock().take().expect("echo can be run only once");
    #[for_await]
    for ch in rx {
        print!("{ch}")
    }
}
