use crate::app::keys::Command;
use crate::app::state::Files;

impl Files {
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
        if !self.in_trash() {
            return;
        }
        let Some(name) = self.listing.selected().map(|entry| entry.name.clone()) else {
            return;
        };
        match self.services.trash.restore(&name) {
            Ok(path) => {
                self.refresh();
                self.inform(format!("Restaurado para {}", path.display()));
            }
            Err(error) => self.fail(format!("{name}: {error}")),
        }
    }

    pub fn restore_all(&mut self) {
        if !self.in_trash() {
            return;
        }
        let names = match self.services.trash.names() {
            Ok(names) => names,
            Err(error) => return self.fail(error),
        };
        let mut restored = 0;
        let mut failed = 0;
        let mut first = None;
        for name in names {
            match self.services.trash.restore(&name) {
                Ok(_) => restored += 1,
                Err(error) => {
                    failed += 1;
                    first.get_or_insert(format!("{name}: {error}"));
                }
            }
        }
        self.refresh();
        if let Some(first) = first {
            self.fail(format!(
                "{restored} restaurados; {failed} falharam. {first}"
            ));
            return;
        }
        self.inform(format!("{restored} itens restaurados"));
    }
}
