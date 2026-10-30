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
            DecodeError::NotAFile => write!(formatter, "Não é um arquivo de imagem"),
            DecodeError::Unreadable(reason) => {
                write!(formatter, "Não foi possível ler o arquivo: {reason}")
            }
            DecodeError::Unsupported => write!(formatter, "Formato de imagem não suportado"),
            DecodeError::TooLarge => write!(
                formatter,
                "Imagem grande demais (máximo de {MAX_SIDE} pixels por lado)"
            ),
            DecodeError::Corrupted => write!(formatter, "Imagem corrompida ou incompleta"),
        }
    }
}

pub trait Decoder: Send + Sync {
    fn decode(&self, path: &Path) -> Result<Picture, DecodeError>;
}

pub trait Gallery {
    fn images(&self, folder: &Path) -> Vec<PathBuf>;
}
