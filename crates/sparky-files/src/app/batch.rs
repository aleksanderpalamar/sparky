use std::path::PathBuf;

use iced::Task;
use sparky_ui::prompt;

use crate::app::message::Message;
use crate::app::state::{Files, Mode};
use crate::batch_rename::rename_all;
use crate::domain::batch_name::{Problem, check, parse, plan};
use crate::domain::undo::Record;
use crate::i18n;

impl Files {
    pub fn start_rename(&mut self) -> Task<Message> {
        if self.listing.marked_count() < 2 {
            return self.begin_rename().map_or_else(Task::none, prompt::select);
        }
        let names = self
            .listing
            .visible()
            .filter(|entry| self.listing.is_marked(entry))
            .map(|entry| entry.name.clone())
            .collect();
        self.input.clear();
        self.mode = Mode::BatchRename(names);
        prompt::focus()
    }

    pub(super) fn finish_batch(&mut self, names: &[String], input: &str) {
        let Some(pattern) = parse(input) else {
            self.fail(i18n::text("batch-hint"));
            return;
        };
        let pairs = plan(names, &pattern);
        let disk = self.services.disk.as_ref();
        if let Err(problem) = check(&pairs, |name| disk.exists(&self.location.join(name))) {
            self.fail(described(&problem));
            return;
        }
        let paths: Vec<(PathBuf, PathBuf)> = pairs
            .iter()
            .filter(|(old, new)| old != new)
            .map(|(old, new)| (self.location.join(old), self.location.join(new)))
            .collect();
        let result = rename_all(&paths, disk);
        self.listing.clear_marks();
        self.refresh();
        if let Err(failure) = result {
            self.fail(failure);
            return;
        }
        for (from, to) in &paths {
            if let Err(error) = self.services.sidebar.relocated(from, to) {
                self.fail(error);
                return;
            }
        }
        if let Some((_, last)) = pairs.last() {
            self.listing.select_name(last);
        }
        self.inform(i18n::counted("batch-renamed", paths.len()));
        self.journal.push(Record::RenamedMany(paths));
    }
}

pub fn described(problem: &Problem) -> String {
    match problem {
        Problem::Invalid(name) => i18n::with("batch-invalid", "name", name.clone()),
        Problem::Repeated(name) => i18n::with("batch-repeated", "name", name.clone()),
        Problem::Taken(name) => i18n::with("batch-taken", "name", name.clone()),
        Problem::Unchanged => i18n::text("batch-unchanged"),
    }
}
