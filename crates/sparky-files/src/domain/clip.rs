use std::path::PathBuf;

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
