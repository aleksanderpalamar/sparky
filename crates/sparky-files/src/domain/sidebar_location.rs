use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarLocation {
    Recent,
    Favorites,
    Network,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidebarItem {
    pub target: SidebarTarget,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SidebarTarget {
    Local(PathBuf),
    Remote { uri: String, directory: bool },
}
