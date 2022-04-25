mod term;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use async_trait::async_trait;
use litchi_user_common::resource::{ResourceError, ResourceResult};

use self::term::Term;

pub type BoxedResource = Box<dyn Resource>;

#[async_trait]
pub trait Resource: Send + Sync + core::fmt::Debug {
    async fn read(&self, max_len: usize) -> ResourceResult<Vec<u8>>;

    async fn write(&self, data: &[u8]) -> ResourceResult<usize>;

    fn boxed(self) -> BoxedResource
    where
        Self: Sized + Send + 'static,
    {
        Box::new(self)
    }
}

pub fn open(path: String) -> ResourceResult<BoxedResource> {
    let res = match path.as_str() {
        "/device/term" => Term::new().boxed(),
        _ => return Err(ResourceError::NotSupported),
    };

    Ok(res)
}
