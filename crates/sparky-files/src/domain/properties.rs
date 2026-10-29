use std::path::PathBuf;
use std::time::SystemTime;

use crate::domain::entry::Kind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Properties {
    pub path: PathBuf,
    pub kind: Kind,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub permissions: u32,
}
