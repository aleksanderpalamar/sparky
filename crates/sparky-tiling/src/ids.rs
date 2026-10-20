#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WindowId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WorkspaceId(pub u32);

impl WorkspaceId {
    pub const FIRST: Self = Self(1);
    pub const LAST: Self = Self(10);

    pub fn is_numbered(&self) -> bool {
        (Self::FIRST.0..=Self::LAST.0).contains(&self.0)
    }
}
