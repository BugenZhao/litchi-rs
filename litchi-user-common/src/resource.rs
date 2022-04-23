#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct ResourceHandle(pub u64);

#[derive(Debug)]
pub enum ResourceError {
    NotSupported,
    NotExists,
    Closed,
}

impl core::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

pub type ResourceResult<T> = Result<T, ResourceError>;
