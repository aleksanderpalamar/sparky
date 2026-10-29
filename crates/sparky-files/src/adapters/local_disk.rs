use std::fs::{self, OpenOptions};
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, Local};

use crate::adapters::home_trash::HomeTrash;
use crate::adapters::tree::{copy_tree, relocate};
use crate::domain::entry::{Entry, Kind};
use crate::domain::properties::Properties;
use crate::ports::Disk;

pub struct LocalDisk {
    trash: HomeTrash,
}

impl LocalDisk {
    pub fn new(trash: HomeTrash) -> Self {
        Self { trash }
    }
}

impl Disk for LocalDisk {
    fn list(&self, dir: &Path) -> io::Result<Vec<Entry>> {
        Ok(fs::read_dir(dir)?
            .filter_map(Result::ok)
            .map(|item| entry_of(&item))
            .collect())
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        fs::canonicalize(path)
    }

    fn exists(&self, path: &Path) -> bool {
        path.symlink_metadata().is_ok()
    }

    fn stamp(&self, dir: &Path) -> Option<SystemTime> {
        fs::metadata(dir)
            .and_then(|metadata| metadata.modified())
            .ok()
    }

    fn create_dir(&self, path: &Path) -> io::Result<()> {
        fs::create_dir(path)
    }

    fn create_file(&self, path: &Path) -> io::Result<()> {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map(|_| ())
    }

    fn relocate(&self, from: &Path, to: &Path) -> io::Result<()> {
        relocate(from, to)
    }

    fn copy(&self, from: &Path, to: &Path) -> io::Result<()> {
        copy_tree(from, to)
    }

    fn trash(&self, path: &Path) -> io::Result<()> {
        self.trash.put(path)
    }

    fn properties(&self, path: &Path) -> io::Result<Properties> {
        let metadata = fs::metadata(path)?;
        Ok(Properties {
            path: path.to_path_buf(),
            kind: if metadata.is_dir() {
                Kind::Directory
            } else {
                Kind::File
            },
            size: metadata.len(),
            modified: metadata.modified().ok(),
            permissions: metadata.permissions().mode() & 0o7777,
        })
    }
}

fn entry_of(item: &fs::DirEntry) -> Entry {
    let path = item.path();
    let metadata = fs::metadata(&path)
        .or_else(|_| fs::symlink_metadata(&path))
        .ok();
    let kind = match metadata.as_ref().map(fs::Metadata::is_dir) {
        Some(true) => Kind::Directory,
        _ => Kind::File,
    };
    Entry {
        name: item.file_name().to_string_lossy().into_owned(),
        kind,
        size: metadata.as_ref().map_or(0, fs::Metadata::len),
        modified: metadata
            .and_then(|metadata| metadata.modified().ok())
            .map(|time| DateTime::<Local>::from(time).naive_local()),
    }
}
