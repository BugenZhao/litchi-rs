use alloc::boxed::Box;
use alloc::vec::Vec;

use async_trait::async_trait;
use futures::StreamExt;
use litchi_user_common::resource::ResourceResult;
use spin::Mutex;

use super::Resource;
use crate::kernel_task::broadcast::Receiver;
use crate::kernel_task::serial;

pub struct Term {
    serial_input_rx: Mutex<Receiver<u8>>,
}

impl Term {
    pub fn new() -> Self {
        Self {
            serial_input_rx: Mutex::new(serial::subscribe()),
        }
    }
}

impl core::fmt::Debug for Term {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Term").finish_non_exhaustive()
    }
}

#[async_trait]
impl Resource for Term {
    async fn read(&self, max_len: usize) -> ResourceResult<Vec<u8>> {
        let stream = &mut *self.serial_input_rx.lock();
        let mut buf = Vec::new();

        while buf.len() < max_len {
            let byte = stream.next().await.unwrap();
            if byte == b'\x7f' {
                // delete (backspace)
                buf.pop();
            } else {
                buf.push(byte);
            }
            if byte == b'\n' {
                break;
            }
        }

        Ok(buf)
    }

    async fn write(&self, _data: &[u8]) -> ResourceResult<usize> {
        todo!()
    }
}
