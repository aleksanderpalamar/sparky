use std::fmt;
use std::path::{Path, PathBuf};

use crate::domain::picture::Picture;

pub const MAX_SIDE: u32 = 16384;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    NotAFile,
    Unreadable(String),
    Unsupported,
    TooLarge,
    Corrupted,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::NotAFile => formatter.write_str(&crate::i18n::text("not-file")),
            DecodeError::Unreadable(reason) => {
                formatter.write_str(&crate::i18n::with("unreadable", "reason", reason.clone()))
            }
            DecodeError::Unsupported => formatter.write_str(&crate::i18n::text("unsupported")),
            DecodeError::TooLarge => formatter.write_str(&crate::i18n::with(
                "too-large",
                "limit",
                MAX_SIDE.to_string(),
            )),
            DecodeError::Corrupted => formatter.write_str(&crate::i18n::text("corrupted")),
        }
    }
}

pub trait Decoder: Send + Sync {
    fn decode(&self, path: &Path) -> Result<Picture, DecodeError>;
}

pub trait Gallery {
    fn images(&self, folder: &Path) -> Vec<PathBuf>;
}
