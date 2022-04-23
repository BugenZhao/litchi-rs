mod term;

use alloc::{boxed::Box, string::String, vec::Vec};
use async_trait::async_trait;

use self::term::Term;

pub type BoxedResource = Box<dyn Resource>;

#[async_trait]
pub trait Resource: Send + Sync + core::fmt::Debug {
    async fn read(&self, max_len: usize) -> Option<Vec<u8>>;

    async fn write(&self, data: &[u8]) -> Option<usize>;

    fn boxed(self) -> BoxedResource
    where
        Self: Sized + Send + 'static,
    {
        Box::new(self)
    }
}

pub fn open(path: String) -> Option<BoxedResource> {
    let res = match path.as_str() {
        "/device/term" => Term::new().boxed(),
        _ => return None,
    };

    Some(res)
}
