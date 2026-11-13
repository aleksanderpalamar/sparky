use std::path::PathBuf;

use crate::app::jobs::Pending;
use crate::domain::clip::Clip;
use crate::domain::entry::Kind;

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
    ConfirmDelete(Vec<String>),
    ConfirmEmptyTrash(usize),
    Conflict(Pending),
}
