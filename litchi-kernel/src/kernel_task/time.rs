use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use futures::StreamExt;
use spin::Mutex;

use super::broadcast;

static SLICE_COUNT: AtomicU64 = AtomicU64::new(0);

type Notifier = broadcast::Sender<()>;

lazy_static::lazy_static! {
    static ref NOTIFIERS: Mutex<BTreeMap<u64, Vec<Notifier>>> = Mutex::new(BTreeMap::new());
}

pub fn inc_slice() {
    // crate::print!(".");
    let old_count = SLICE_COUNT.fetch_add(1, Ordering::SeqCst);
    let count = old_count + 1;
    if let Some(notifers) = NOTIFIERS.lock().remove(&count) {
        notifers.into_iter().for_each(|n| n.send_one(()));
    }
}

pub async fn sleep(slice: usize) {
    if slice == 0 {
        return;
    }
    let (tx, mut rx) = broadcast::channel();
    let current = SLICE_COUNT.load(Ordering::Acquire);
    NOTIFIERS
        .lock()
        .entry(current + slice as u64)
        .or_default()
        .push(tx);

    rx.next().await.unwrap();
}
