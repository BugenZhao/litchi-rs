use futures_async_stream::for_await;
use spin::Mutex;

use crate::{kernel_task::broadcast, print};

lazy_static::lazy_static! {
    static ref CHANNEL: (broadcast::Sender<u8>, Mutex<Option<broadcast::Receiver<u8>>>) = {
        let (tx, rx) = broadcast::channel();
        (tx, Mutex::new(Some(rx)))
    };
}

pub fn push(byte: u8) {
    CHANNEL.0.send_one(byte);
}

pub fn subscribe() -> broadcast::Receiver<u8> {
    CHANNEL.0.subscribe()
}

pub(super) async fn echo() {
    let rx = CHANNEL.1.lock().take().expect("echo can be run only once");
    #[for_await]
    for byte in rx {
        let ch = char::from_u32(byte as u32).unwrap_or('?');
        print!("{ch}")
    }
}
