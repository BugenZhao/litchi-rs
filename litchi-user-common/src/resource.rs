#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct ResourceHandle(pub u64);

impl ResourceHandle {
    pub const TERM_INPUT: Self = Self(0);
    pub const TERM_OUTPUT: Self = Self(1);
}
