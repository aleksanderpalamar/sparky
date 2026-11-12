use std::{mem, path::PathBuf};

use iced::Task;

use crate::app::message::Message;
use crate::app::state::{Files, Mode};
use crate::domain::clip::Clip;
use crate::domain::entry::Kind;
use crate::domain::naming::stem_length;
use crate::domain::undo::Record;
use crate::domain::wording::{agreed, items};
use crate::i18n;
use crate::operations::{self, Failure};

impl Files {
    pub fn copy(&mut self) {
        self.hold(Clip::Copy);
        self.publish_clip();
    }

    pub fn cut(&mut self) {
        self.hold(Clip::Cut);
        self.publish_clip();
    }

    pub fn begin_rename(&mut self) -> Option<usize> {
        let name = self.listing.selected()?.name.clone();
        let stem = stem_length(&name);
        self.input = name.clone();
        self.mode = Mode::Rename(name);
        Some(stem)
    }

    pub fn begin_create(&mut self) {
        self.input.clear();
        self.mode = Mode::Create;
    }

    pub fn begin_new(&mut self, kind: Kind) -> usize {
        self.input = i18n::text(match kind {
            Kind::Directory => "new-folder",
            Kind::File => "new-document",
        });
        self.mode = Mode::New(kind);
        self.input.chars().count()
    }

    pub fn begin_filter(&mut self) {
        self.input = self.listing.filter().to_owned();
        self.mode = Mode::Filter;
    }

    pub fn type_text(&mut self, text: String) {
        if self.mode == Mode::Filter {
            self.listing.set_filter(&text);
        }
        self.input = text;
    }

    pub fn submit(&mut self) -> Task<Message> {
        let input = mem::take(&mut self.input);
        match mem::replace(&mut self.mode, Mode::Browse) {
            Mode::Rename(original) => self.finish_rename(&original, input.trim()),
            Mode::BatchRename(names) => self.finish_batch(&names, &input),
            Mode::Create => {
                let result =
                    operations::create(&self.location, &input, self.services.disk.as_ref());
                self.created(result);
            }
            Mode::New(kind) => {
                let disk = self.services.disk.as_ref();
                let result = operations::create_as(&self.location, &input, kind, disk);
                self.created(result);
            }
            Mode::Destination(clip) => return self.finish_destination(clip, &input),
            Mode::Compress(paths) => return self.start_archive(paths, &input),
            Mode::ConfirmRemoval(removal, names) => {
                self.mode = Mode::ConfirmRemoval(removal, names)
            }
            Mode::ConfirmEmptyTrash(count) => self.mode = Mode::ConfirmEmptyTrash(count),
            Mode::Conflict(pending) => self.mode = Mode::Conflict(pending),
            Mode::Browse | Mode::Filter => {}
        }
        Task::none()
    }

    pub fn escape(&mut self) {
        if self.open_with.take().is_some() {
            return;
        }
        if self.details.take().is_some() {
            return;
        }
        if self.context.take().is_some() {
            return;
        }
        if self.mode == Mode::Browse && self.cancel_job() {
            return;
        }
        match mem::replace(&mut self.mode, Mode::Browse) {
            Mode::Browse if !self.listing.filter().is_empty() => self.listing.set_filter(""),
            Mode::Browse => self.listing.clear_marks(),
            Mode::Filter => self.listing.set_filter(""),
            Mode::Rename(_)
            | Mode::BatchRename(_)
            | Mode::Create
            | Mode::New(_)
            | Mode::Destination(_)
            | Mode::Compress(_)
            | Mode::ConfirmRemoval(..)
            | Mode::ConfirmEmptyTrash(_)
            | Mode::Conflict(_) => {}
        }
        self.input.clear();
    }

    fn hold(&mut self, clip: fn(Vec<PathBuf>) -> Clip) {
        let paths = self.chosen_paths();
        if paths.is_empty() {
            return;
        }
        let clip = clip(paths);
        let count = clip.paths().len();
        self.inform(format!("{} {}", items(count), agreed(clip.verb(), count)));
        self.clip = Some(clip);
        self.listing.clear_marks();
    }

    fn finish_rename(&mut self, original: &str, new: &str) {
        let result = operations::rename(&self.location, original, new, self.services.disk.as_ref());
        self.refresh();
        match result {
            Ok(()) => {
                let from = self.location.join(original);
                let to = self.location.join(new);
                if let Err(error) = self.services.sidebar.relocated(&from, &to) {
                    self.fail(error);
                    return;
                }
                if from != to {
                    self.journal.push(Record::Renamed { from, to });
                }
                self.listing.select_name(new);
                self.inform(i18n::with("renamed-to", "name", new.to_owned()));
            }
            Err(failure) => self.fail(failure),
        }
    }

    fn created(&mut self, result: Result<String, Failure>) {
        self.refresh();
        match result {
            Ok(name) => {
                self.journal
                    .push(Record::Created(self.location.join(&name)));
                self.listing.select_name(&name);
                self.inform(format!("Criado {name}"));
            }
            Err(failure) => self.fail(failure),
        }
    }
}
