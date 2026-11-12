use std::fs::{self, OpenOptions};
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::adapters::disk_trash::DiskTrashes;
use crate::adapters::gio_listing;
use crate::adapters::home_trash::HomeTrash;
use crate::adapters::staged;
use crate::adapters::tree::remove_tree;
use crate::domain::entry::{Entry, Kind, Meta};
use crate::domain::properties::Properties;
use crate::domain::undo::TrashEntry;
use crate::ports::{Disk, TrashStore};

pub struct LocalDisk {
    trash: DiskTrashes,
    remote: Option<PathBuf>,
}

impl LocalDisk {
    pub fn new(trash: HomeTrash) -> Self {
        Self {
            trash: DiskTrashes::new(trash),
            remote: None,
        }
    }

    pub fn with_remote(self, remote: Option<PathBuf>) -> Self {
        Self { remote, ..self }
    }

    fn is_remote(&self, dir: &Path) -> bool {
        self.remote
            .as_deref()
            .is_some_and(|root| gio_listing::serves(root, dir))
    }
}

impl Disk for LocalDisk {
    fn list(&self, dir: &Path) -> io::Result<Vec<Entry>> {
        if self.is_remote(dir) {
            return gio_listing::list(dir);
        }
        Ok(fs::read_dir(dir)?
            .filter_map(Result::ok)
            .map(|item| entry_of(&item))
            .collect())
    }

    fn detail(&self, path: &Path) -> Meta {
        meta_of(path)
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
        match staged::rename_noreplace(from, to) {
            Err(error) if error.kind() == io::ErrorKind::CrossesDevices => {
                staged::copy(from, to, &mut |_| true)?;
                remove_tree(from)
            }
            other => other,
        }
    }

    fn trash(&self, path: &Path) -> io::Result<TrashEntry> {
        if self.is_remote(path) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                crate::i18n::text("trash-unavailable"),
            ));
        }
        self.trash.for_item(path)?.put(path)
    }

    fn untrash(&self, entry: &TrashEntry) -> io::Result<PathBuf> {
        let trash = match &entry.topdir {
            Some(topdir) => HomeTrash::on_disk(entry.root.clone(), topdir.clone()),
            None => HomeTrash::new(entry.root.clone()),
        };
        trash.restore(&entry.name)
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
    let directory = match item.file_type() {
        Ok(kind) if kind.is_symlink() => fs::metadata(item.path()).is_ok_and(|meta| meta.is_dir()),
        Ok(kind) => kind.is_dir(),
        Err(_) => false,
    };
    Entry {
        name: item.file_name().to_string_lossy().into_owned(),
        kind: match directory {
            true => Kind::Directory,
            false => Kind::File,
        },
        meta: Meta::Pending,
    }
}

fn meta_of(path: &Path) -> Meta {
    let Ok(metadata) = fs::metadata(path).or_else(|_| fs::symlink_metadata(path)) else {
        return Meta::Known {
            size: 0,
            modified: None,
        };
    };
    Meta::Known {
        size: metadata.len(),
        modified: metadata.modified().ok(),
    }
}
