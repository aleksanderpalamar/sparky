use std::mem;
use std::path::PathBuf;

use iced::Task;

use crate::app::keys::Answer;
use crate::app::message::Message;
use crate::app::state::{Files, Mode};
use crate::domain::clip::Clip;
use crate::domain::entry::Kind;
use crate::domain::naming::stem_length;
use crate::domain::wording::{agreed, items};
use crate::operations::{self, Failure};

impl Files {
    pub fn copy(&mut self) {
        self.hold(Clip::Copy);
    }

    pub fn cut(&mut self) {
        self.hold(Clip::Cut);
    }

    pub fn paste(&mut self) {
        let Some(clip) = self.clip.clone() else {
            self.inform("Nada para colar".to_owned());
            return;
        };
        let result = operations::paste(&clip, &self.location, self.services.disk.as_ref());
        self.refresh();
        match result {
            Ok(names) => {
                if let Some(last) = names.last() {
                    self.listing.select_name(last);
                }
                if matches!(clip, Clip::Cut(_)) {
                    self.clip = None;
                }
                self.inform(format!(
                    "{} {}",
                    items(names.len()),
                    agreed("colado", names.len())
                ));
            }
            Err(failure) => self.fail(failure),
        }
    }

    pub fn ask_trash(&mut self) {
        let names = self.listing.chosen();
        if !names.is_empty() {
            self.mode = Mode::ConfirmTrash(names);
        }
    }

    pub fn answer_trash(&mut self, answer: Answer) {
        let Mode::ConfirmTrash(names) = mem::replace(&mut self.mode, Mode::Browse) else {
            return;
        };
        if answer == Answer::No {
            return;
        }
        let result = operations::trash(&self.location, &names, self.services.disk.as_ref());
        self.listing.clear_marks();
        self.refresh();
        match result {
            Ok(count) => self.inform(format!("{} na lixeira", items(count))),
            Err(failure) => self.fail(failure),
        }
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
        self.input = match kind {
            Kind::Directory => "Nova pasta",
            Kind::File => "Novo documento",
        }
        .to_owned();
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
            Mode::Destination(clip) => self.finish_destination(&clip, &input),
            Mode::Compress(paths) => return self.start_archive(paths, &input),
            Mode::ConfirmTrash(names) => self.mode = Mode::ConfirmTrash(names),
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
        match mem::replace(&mut self.mode, Mode::Browse) {
            Mode::Browse if !self.listing.filter().is_empty() => self.listing.set_filter(""),
            Mode::Browse => self.listing.clear_marks(),
            Mode::Filter => self.listing.set_filter(""),
            Mode::Rename(_)
            | Mode::Create
            | Mode::New(_)
            | Mode::Destination(_)
            | Mode::Compress(_)
            | Mode::ConfirmTrash(_) => {}
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
                self.listing.select_name(new);
                self.inform(format!("Renomeado para {new}"));
            }
            Err(failure) => self.fail(failure),
        }
    }

    fn created(&mut self, result: Result<String, Failure>) {
        self.refresh();
        match result {
            Ok(name) => {
                self.listing.select_name(&name);
                self.inform(format!("Criado {name}"));
            }
            Err(failure) => self.fail(failure),
        }
    }
}
