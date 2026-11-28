use std::path::PathBuf;

use iced::Task;
use sparky_ui::prompt;

use crate::app::message::Message;
use crate::app::state::{Files, Mode};
use crate::domain::entry::{Entry, Kind};
use crate::domain::location_input::{Completion, Pattern, Typed, matches, pattern, typed};
use crate::i18n;

impl Files {
    pub fn begin_location(&mut self) -> Task<Message> {
        self.input = self
            .network_uri
            .clone()
            .unwrap_or_else(|| self.location.display().to_string());
        self.mode = Mode::Location(None);
        prompt::focus()
    }

    pub fn complete_location(&mut self, forward: bool) {
        let Mode::Location(previous) = &self.mode else {
            return;
        };
        let cycling = previous
            .clone()
            .filter(|completion| completion.text() == self.input);
        let next = match cycling {
            Some(mut completion) => {
                completion.step(forward);
                Some(completion)
            }
            None => self.fresh_completion(),
        };
        let Some(completion) = next else {
            return;
        };
        self.input = completion.text();
        self.mode = match completion.names.len() {
            1 => Mode::Location(None),
            _ => Mode::Location(Some(completion)),
        };
    }

    pub(super) fn finish_location(&mut self, input: &str) -> Task<Message> {
        match typed(input, &self.location, &self.home) {
            None => {}
            Some(Typed::Remote(_)) if self.chooser.is_some() => {
                self.fail(i18n::text("location-remote-chooser"));
            }
            Some(Typed::Remote(uri)) => return self.load_network(uri),
            Some(Typed::Folder(path)) => self.open_typed(path, input),
        }
        Task::none()
    }

    fn open_typed(&mut self, path: PathBuf, input: &str) {
        let path = match self.services.disk.exists(&path) {
            true => path,
            false => self.first_completion(input).unwrap_or(path),
        };
        match self.services.disk.properties(&path) {
            Ok(details) if details.kind == Kind::Directory => {
                self.go(path);
            }
            Ok(_) => self.fail(i18n::text("location-not-folder")),
            Err(error) => self.fail(format!("{}: {error}", self.shown(&path))),
        }
    }

    fn fresh_completion(&self) -> Option<Completion> {
        let (pattern, found) = self.candidates(&self.input)?;
        let mut names: Vec<String> = found.iter().map(|entry| entry.name.clone()).collect();
        if let [only] = found.as_slice()
            && only.kind == Kind::Directory
        {
            names = vec![format!("{}/", only.name)];
        }
        Some(Completion {
            kept: pattern.kept,
            names,
            selected: 0,
        })
    }

    fn first_completion(&self, input: &str) -> Option<PathBuf> {
        let (pattern, found) = self.candidates(input)?;
        let first = found.first()?;
        Some(pattern.folder.join(&first.name))
    }

    fn candidates(&self, input: &str) -> Option<(Pattern, Vec<Entry>)> {
        let Typed::Folder(resolved) = typed(input, &self.location, &self.home)? else {
            return None;
        };
        let pattern = pattern(input, &resolved)?;
        let entries = self.services.disk.list(&pattern.folder).ok()?;
        let names = matches(
            &pattern.prefix,
            entries.iter().map(|entry| entry.name.clone()),
        );
        let found: Vec<Entry> = names
            .iter()
            .filter_map(|name| entries.iter().find(|entry| &entry.name == name).cloned())
            .collect();
        (!found.is_empty()).then_some((pattern, found))
    }
}
