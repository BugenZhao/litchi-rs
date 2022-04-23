#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct ResourceHandle(pub u64);

#[derive(Debug)]
pub enum ResourceError {
    NotSupported,
    NotExists,
    Closed,
}

pub type ResourceResult<T> = Result<T, ResourceError>;
