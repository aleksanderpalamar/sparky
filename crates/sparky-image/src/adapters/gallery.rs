use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::format::Format;
use crate::ports::Gallery;

pub struct FolderGallery;

impl Gallery for FolderGallery {
    fn images(&self, folder: &Path) -> Vec<PathBuf> {
        let Ok(entries) = fs::read_dir(folder) else {
            return Vec::new();
        };
        entries
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| Format::from_path(path).is_some() && path.is_file())
            .collect()
    }
}
