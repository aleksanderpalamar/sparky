use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use crate::app::keys::Command;
use crate::app::message::TrashEvent;
use crate::app::state::Files;
use crate::domain::toast::Toast;
use crate::ports::TrashStore;

impl Files {
    pub fn handle_trash(&mut self, action: TrashEvent) {
        match action {
            TrashEvent::Open => self.open_trash(),
            TrashEvent::OpenOther(index) => self.open_other_trash(index),
            TrashEvent::RestoreSelected => self.restore_selected(),
            TrashEvent::RestoreAll => self.restore_all(),
            TrashEvent::Empty => self.ask_empty_trash(),
        }
    }

    pub fn blocks_in_trash(&self, command: Command) -> bool {
        self.in_trash()
            && matches!(
                command,
                Command::Copy
                    | Command::Cut
                    | Command::Paste
                    | Command::Create
                    | Command::NewFolder
                    | Command::Rename
                    | Command::Trash
                    | Command::Delete
                    | Command::Extract
            )
    }

    pub fn in_trash(&self) -> bool {
        self.trash_here().is_some()
    }

    pub fn trash_here(&self) -> Option<Arc<dyn TrashStore>> {
        if self.sidebar_page.is_some() {
            return None;
        }
        std::iter::once(&self.services.trash)
            .chain(&self.other_trashes)
            .find(|trash| trash.directory() == self.location)
            .cloned()
    }

    pub fn refresh_trashes(&mut self) {
        self.other_trashes = self.services.trash_disks.others();
    }

    pub fn open_other_trash(&mut self, index: usize) {
        if let Some(directory) = self.other_trashes.get(index).map(|trash| trash.directory()) {
            self.go(directory);
        }
    }

    pub fn open_trash(&mut self) {
        self.refresh_trashes();
        match self.services.trash.open() {
            Ok(path) => {
                self.go(path);
            }
            Err(error) => self.fail(error),
        }
    }

    pub fn restore_selected(&mut self) {
        if !self.in_trash() || self.emptying {
            return;
        }
        let Some(shown) = self.listing.selected().map(|entry| entry.name.clone()) else {
            return;
        };
        let Some(trash) = self.trash_here() else {
            return;
        };
        let name = match raw_name(trash.as_ref(), &shown) {
            Ok(name) => name,
            Err(error) => return self.fail(format!("{shown}: {error}")),
        };
        match trash.restore(&name) {
            Ok(path) => {
                self.refresh();
                self.toast = Some(Toast::restored(vec![path]));
            }
            Err(error) => self.fail(format!("{shown}: {error}")),
        }
    }

    pub fn restore_all(&mut self) {
        if !self.in_trash() || self.emptying {
            return;
        }
        let Some(trash) = self.trash_here() else {
            return;
        };
        let names = match trash.names() {
            Ok(names) => names,
            Err(error) => return self.fail(error),
        };
        let (restored, failures) = restore_items(trash.as_ref(), names);
        self.refresh();
        if let Some((name, error)) = failures.first() {
            self.fail(format!(
                "{} restaurados; {} falharam. {}: {error}",
                restored.len(),
                failures.len(),
                name.to_string_lossy()
            ));
            return;
        }
        if !restored.is_empty() {
            self.toast = Some(Toast::restored(restored));
        }
    }
}

fn raw_name(trash: &dyn TrashStore, shown: &str) -> io::Result<OsString> {
    let names = trash.names()?;
    let mut matching = names
        .into_iter()
        .filter(|name| name.to_string_lossy() == shown);
    match (matching.next(), matching.next()) {
        (Some(name), None) => Ok(name),
        (None, _) => Err(io::Error::new(
            io::ErrorKind::NotFound,
            crate::i18n::text("item-not-found"),
        )),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            crate::i18n::text("trash-ambiguous-name"),
        )),
    }
}

fn restore_items(
    trash: &dyn TrashStore,
    names: Vec<OsString>,
) -> (Vec<PathBuf>, Vec<(OsString, io::Error)>) {
    let mut pending = names;
    let mut restored = Vec::new();
    let mut failures = Vec::new();
    while !pending.is_empty() {
        let mut retry = Vec::new();
        let previous = restored.len();
        for name in pending {
            match trash.restore(&name) {
                Ok(path) => restored.push(path),
                Err(error) if error.kind() == io::ErrorKind::NotFound => retry.push((name, error)),
                Err(error) => failures.push((name, error)),
            }
        }
        if restored.len() == previous {
            failures.extend(retry);
            break;
        }
        pending = retry.into_iter().map(|(name, _)| name).collect();
    }
    (restored, failures)
}
