use std::ffi::OsString;
use std::io;

use crate::app::keys::Command;
use crate::app::message::TrashEvent;
use crate::app::state::Files;
use crate::ports::TrashStore;

impl Files {
    pub fn handle_trash(&mut self, action: TrashEvent) {
        match action {
            TrashEvent::Open => self.open_trash(),
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
            )
    }

    pub fn in_trash(&self) -> bool {
        self.location == self.services.trash.directory()
    }

    pub fn open_trash(&mut self) {
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
        let name = match raw_name(self.services.trash.as_ref(), &shown) {
            Ok(name) => name,
            Err(error) => return self.fail(format!("{shown}: {error}")),
        };
        match self.services.trash.restore(&name) {
            Ok(path) => {
                self.refresh();
                self.inform(format!("Restaurado para {}", path.display()));
            }
            Err(error) => self.fail(format!("{shown}: {error}")),
        }
    }

    pub fn restore_all(&mut self) {
        if !self.in_trash() || self.emptying {
            return;
        }
        let names = match self.services.trash.names() {
            Ok(names) => names,
            Err(error) => return self.fail(error),
        };
        let (restored, failures) = restore_items(self.services.trash.as_ref(), names);
        self.refresh();
        if let Some((name, error)) = failures.first() {
            self.fail(format!(
                "{restored} restaurados; {} falharam. {}: {error}",
                failures.len(),
                name.to_string_lossy()
            ));
            return;
        }
        self.inform(format!("{restored} itens restaurados"));
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
            "item não encontrado",
        )),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "nome ambíguo na lixeira",
        )),
    }
}

fn restore_items(
    trash: &dyn TrashStore,
    names: Vec<OsString>,
) -> (usize, Vec<(OsString, io::Error)>) {
    let mut pending = names;
    let mut restored = 0;
    let mut failures = Vec::new();
    while !pending.is_empty() {
        let mut retry = Vec::new();
        let previous = restored;
        for name in pending {
            match trash.restore(&name) {
                Ok(_) => restored += 1,
                Err(error) if error.kind() == io::ErrorKind::NotFound => retry.push((name, error)),
                Err(error) => failures.push((name, error)),
            }
        }
        if restored == previous {
            failures.extend(retry);
            break;
        }
        pending = retry.into_iter().map(|(name, _)| name).collect();
    }
    (restored, failures)
}
