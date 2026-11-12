use std::mem;

use fluent_bundle::FluentArgs;

use crate::app::keys::Answer;
use crate::app::state::{Files, Mode};
use crate::domain::removal::Removal;
use crate::domain::undo::Record;
use crate::domain::wording::items;
use crate::i18n;
use crate::operations;

impl Files {
    pub fn ask_trash(&mut self) {
        let removal = match self.services.disk.has_trash(&self.location) {
            true => Removal::Trash,
            false => Removal::Delete,
        };
        self.ask_removal(removal);
    }

    pub fn ask_removal(&mut self, removal: Removal) {
        let names = self.listing.chosen();
        if !names.is_empty() {
            self.mode = Mode::ConfirmRemoval(removal, names);
        }
    }

    pub fn answer_removal(&mut self, answer: Answer) {
        let Mode::ConfirmRemoval(removal, names) = mem::replace(&mut self.mode, Mode::Browse)
        else {
            return;
        };
        if answer == Answer::No {
            return;
        }
        match removal {
            Removal::Trash => self.trash(&names),
            Removal::Delete => self.delete(&names),
        }
    }

    fn trash(&mut self, names: &[String]) {
        let (entries, result) =
            operations::trash(&self.location, names, self.services.disk.as_ref());
        self.listing.clear_marks();
        self.refresh();
        let count = entries.len();
        self.refresh_trashes();
        if !entries.is_empty() {
            self.journal.push(Record::Trashed(entries));
        }
        match result {
            Ok(()) => self.inform(format!("{} na lixeira", items(count))),
            Err(failure) => self.fail(failure),
        }
    }

    fn delete(&mut self, names: &[String]) {
        let (count, result) =
            operations::delete(&self.location, names, self.services.disk.as_ref());
        self.listing.clear_marks();
        self.refresh();
        match result {
            Ok(()) => self.inform(deleted(count)),
            Err(failure) => self.fail(failure),
        }
    }
}

fn deleted(count: usize) -> String {
    let mut args = FluentArgs::new();
    args.set("count", count);
    i18n::message("deleted-items", Some(&args))
}
