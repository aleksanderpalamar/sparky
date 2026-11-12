use std::path::PathBuf;

use crate::domain::image::ImageMime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Clip {
    Copy(Vec<PathBuf>),
    Cut(Vec<PathBuf>),
}

impl Clip {
    pub fn paths(&self) -> &[PathBuf] {
        match self {
            Clip::Copy(paths) | Clip::Cut(paths) => paths,
        }
    }

    pub fn verb(&self) -> &'static str {
        match self {
            Clip::Copy(_) => "copiado",
            Clip::Cut(_) => "recortado",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipOffer {
    pub clip: Clip,
    pub image: Option<(PathBuf, ImageMime)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pasted {
    Files(Clip),
    Other,
    Unavailable,
}
