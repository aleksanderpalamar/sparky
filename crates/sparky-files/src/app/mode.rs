use std::path::PathBuf;

use crate::app::jobs::Pending;
use crate::domain::clip::Clip;
use crate::domain::entry::Kind;
use crate::domain::removal::Removal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Browse,
    Filter,
    Search,
    Rename(String),
    BatchRename(Vec<String>),
    Create,
    New(Kind),
    Destination(Clip),
    Compress(Vec<PathBuf>),
    ConfirmRemoval(Removal, Vec<String>),
    ConfirmEmptyTrash(usize),
    Conflict(Pending),
}
