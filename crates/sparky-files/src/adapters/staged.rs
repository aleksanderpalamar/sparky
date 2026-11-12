use std::fs;
use std::io;
use std::path::Path;

use rustix::fs::{CWD, RenameFlags, renameat_with};
use rustix::io::Errno;

use crate::adapters::tree::copy_tree;

const STAGING: &str = ".sparky-files-";

pub fn copy(from: &Path, to: &Path) -> io::Result<()> {
    let parent = to
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "destino sem pasta"))?;
    let staging = tempfile::Builder::new()
        .prefix(STAGING)
        .tempdir_in(parent)?;
    let staged = staging.path().join("item");
    copy_tree(from, &staged)?;
    rename_noreplace(&staged, to)
}

pub fn rename_noreplace(from: &Path, to: &Path) -> io::Result<()> {
    match renameat_with(CWD, from, CWD, to, RenameFlags::NOREPLACE) {
        Ok(()) => Ok(()),
        Err(Errno::INVAL | Errno::NOSYS | Errno::OPNOTSUPP) => rename_checked(from, to),
        Err(error) => Err(error.into()),
    }
}

fn rename_checked(from: &Path, to: &Path) -> io::Result<()> {
    if to.symlink_metadata().is_ok() {
        return Err(io::ErrorKind::AlreadyExists.into());
    }
    fs::rename(from, to)
}
