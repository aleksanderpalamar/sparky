use std::io;
use std::path::{Path, PathBuf};

use crate::adapters::disk_trash::DiskTrashes;
use crate::adapters::gio_listing;
use crate::adapters::home_trash::HomeTrash;
use crate::adapters::staged;
use crate::adapters::tree::{self, copy_tree, remove_tree};
use crate::domain::transfer::{Action, Step};
use crate::domain::undo::TrashEntry;
use crate::ports::Transfer;

pub struct LocalTransfer {
    trash: DiskTrashes,
    remote: Option<PathBuf>,
}

impl LocalTransfer {
    pub fn new(trash: HomeTrash, remote: Option<PathBuf>) -> Self {
        Self {
            trash: DiskTrashes::new(trash),
            remote,
        }
    }

    fn copy(&self, from: &Path, to: &Path, advance: &mut dyn FnMut(u64) -> bool) -> io::Result<()> {
        let remote = self
            .remote
            .as_deref()
            .is_some_and(|root| gio_listing::serves(root, to));
        if !remote {
            return staged::copy(from, to, advance);
        }
        if to.symlink_metadata().is_ok() {
            return Err(io::ErrorKind::AlreadyExists.into());
        }
        copy_tree(from, to, advance).inspect_err(|_| {
            let _ = remove_tree(to);
        })
    }

    fn relocate(
        &self,
        from: &Path,
        to: &Path,
        advance: &mut dyn FnMut(u64) -> bool,
    ) -> io::Result<()> {
        match staged::rename_noreplace(from, to) {
            Ok(()) => {
                advance(tree::size(to));
                Ok(())
            }
            Err(error) if error.kind() == io::ErrorKind::CrossesDevices => {
                self.copy(from, to, advance)?;
                remove_tree(from)
            }
            Err(error) => Err(error),
        }
    }
}

impl Transfer for LocalTransfer {
    fn size(&self, path: &Path) -> u64 {
        tree::size(path)
    }

    fn run(
        &self,
        step: &Step,
        advance: &mut dyn FnMut(u64) -> bool,
    ) -> io::Result<Option<TrashEntry>> {
        let displaced = match step.replace {
            true => Some(self.trash.for_item(&step.target)?.put(&step.target)?),
            false => None,
        };
        match step.action {
            Action::Copy => self.copy(&step.source, &step.target, advance)?,
            Action::Move => self.relocate(&step.source, &step.target, advance)?,
        }
        Ok(displaced)
    }
}
