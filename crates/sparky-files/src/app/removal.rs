use std::mem;

use fluent_bundle::FluentArgs;
use iced::Task;

use crate::app::keys::Answer;
use crate::app::message::Message;
use crate::app::state::{Files, Mode};
use crate::domain::toast::Toast;
use crate::domain::undo::Record;
use crate::i18n;
use crate::operations;

impl Files {
    pub fn ask_trash(&mut self) {
        let names = self.listing.chosen();
        self.trash_or_ask(names);
    }

    pub(super) fn trash_or_ask(&mut self, names: Vec<String>) {
        if names.is_empty() {
            return;
        }
        match self.services.disk.has_trash(&self.location) {
            true => self.trash(names),
            false => self.mode = Mode::ConfirmDelete(names),
        }
    }

    pub fn ask_delete(&mut self) {
        let names = self.listing.chosen();
        if !names.is_empty() {
            self.mode = Mode::ConfirmDelete(names);
        }
    }

    pub fn answer_delete(&mut self, answer: Answer) {
        let Mode::ConfirmDelete(names) = mem::replace(&mut self.mode, Mode::Browse) else {
            return;
        };
        if answer == Answer::Yes {
            self.delete(&names);
        }
    }

    pub fn undo_toast(&mut self) -> Task<Message> {
        let Some(toast) = self.toast.take() else {
            return Task::none();
        };
        match self.journal.last() == Some(&Record::Trashed(toast.entries)) {
            true => self.undo(),
            false => Task::none(),
        }
    }

    pub fn tick_toast(&mut self) {
        if self.toast.as_mut().is_some_and(|toast| !toast.tick()) {
            self.toast = None;
        }
    }

    fn trash(&mut self, names: Vec<String>) {
        let (entries, result) =
            operations::trash(&self.location, &names, self.services.disk.as_ref());
        self.listing.clear_marks();
        self.refresh();
        self.refresh_trashes();
        self.toast = None;
        if !entries.is_empty() {
            self.journal.push(Record::Trashed(entries.clone()));
        }
        match result {
            Ok(()) => self.toast = Some(Toast::trashed(names, entries)),
            Err(failure) => self.fail(failure),
        }
    }

    fn delete(&mut self, names: &[String]) {
        let (count, result) =
            operations::delete(&self.location, names, self.services.disk.as_ref());
        self.listing.clear_marks();
        self.refresh();
        self.toast = None;
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
