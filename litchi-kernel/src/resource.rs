use alloc::{boxed::Box, string::String};

pub type BoxedResource = Box<dyn Resource>;

pub trait Resource: Send + Sync + core::fmt::Debug {
    fn read(&self, buf: &mut [u8]) -> Option<usize>;

    fn write(&self, data: &[u8]) -> Option<usize>;
}

pub fn open(path: String) -> Option<BoxedResource> {
    match path.as_str() {
        "/device/term_input" => todo!(),
        _ => None,
    }
}
