mod guard;
mod run;
mod update;

#[cfg(test)]
mod chooser_tests;

use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::domain::choice::{Outcome, Purpose, Request};
use crate::domain::entry::{Entry, Kind};
use crate::i18n;
use crate::ports::chooser::Answer;

pub use run::run;
pub use update::update;

pub const APPLICATION_ID: &str = "sparky-file-chooser";
pub const NAME_INPUT: &str = "chooser-name";

#[derive(Debug, Clone)]
pub enum ChooserEvent {
    Name(String),
    Filter(usize),
    Accept,
    Replace,
    KeepEditing,
    Cancel,
}

pub struct Chooser {
    pub request: Request,
    pub name: String,
    pub filter: Option<usize>,
    pub replace: Option<Vec<PathBuf>>,
    pub touched: Option<PathBuf>,
    answer: Rc<dyn Answer>,
    mime: fn(&str) -> String,
}

impl Chooser {
    pub fn new(request: Request, answer: Rc<dyn Answer>, mime: fn(&str) -> String) -> Self {
        let name = match &request.purpose {
            Purpose::Save { name } => name.clone(),
            _ => String::new(),
        };
        let filter = request
            .filter
            .or_else(|| (!request.filters.is_empty()).then_some(0));
        Self {
            request,
            name,
            filter,
            replace: None,
            touched: None,
            answer,
            mime,
        }
    }

    pub fn naming(&self) -> bool {
        matches!(self.request.purpose, Purpose::Save { .. })
    }

    pub fn folders_only(&self) -> bool {
        matches!(
            self.request.purpose,
            Purpose::Open {
                directory: true,
                ..
            } | Purpose::SaveFiles { .. }
        )
    }

    pub fn purpose(&self) -> Purpose {
        match &self.request.purpose {
            Purpose::Save { .. } => Purpose::Save {
                name: self.name.clone(),
            },
            other => other.clone(),
        }
    }

    pub fn keep(&self, entries: &mut Vec<Entry>) {
        let filter = self
            .filter
            .and_then(|index| self.request.filters.get(index));
        match (self.folders_only(), filter) {
            (true, _) => entries.retain(|entry| entry.kind == Kind::Directory),
            (false, Some(filter)) => entries.retain(|entry| {
                entry.kind == Kind::Directory || filter.accepts(&entry.name, self.mime)
            }),
            (false, None) => {}
        }
    }

    pub fn answer(&self, paths: Vec<PathBuf>) {
        self.answer.send(&Outcome {
            paths,
            filter: self.filter,
        });
    }

    pub fn touched_here(&self, location: &Path) -> bool {
        self.touched.as_deref() == Some(location)
    }

    pub fn title(&self) -> String {
        match self.request.title.trim() {
            "" if self.naming() => i18n::text("chooser-title-save"),
            "" => i18n::text("chooser-title-open"),
            title => title.to_owned(),
        }
    }

    pub fn accept_label(&self) -> String {
        self.request.accept_label().unwrap_or_else(|| {
            let key = match self.request.purpose {
                Purpose::Save { .. } | Purpose::SaveFiles { .. } => "chooser-save",
                Purpose::Open {
                    directory: true, ..
                } => "chooser-select",
                Purpose::Open { .. } => "chooser-open",
            };
            i18n::text(key)
        })
    }
}
