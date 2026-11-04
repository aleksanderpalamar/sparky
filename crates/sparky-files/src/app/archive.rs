use std::path::PathBuf;

use iced::Task;

use crate::app::message::Message;
use crate::app::state::{Files, Mode};
use crate::domain::naming::valid;
use crate::i18n;

impl Files {
    pub fn begin_archive(&mut self, paths: Vec<PathBuf>) -> Option<usize> {
        if paths.is_empty() {
            return None;
        }
        if self.archive_work.is_some() {
            self.fail(i18n::text("archive-busy"));
            return None;
        }
        let stem = match paths.as_slice() {
            [only] => only
                .file_name()
                .map(|name| name.to_string_lossy().into_owned()),
            _ => None,
        }
        .unwrap_or_else(|| i18n::text("archive-default-name"));
        self.input = format!("{stem}.tar.gz");
        self.mode = Mode::Compress(paths);
        Some(stem.chars().count())
    }

    pub fn start_archive(&mut self, paths: Vec<PathBuf>, input: &str) -> Task<Message> {
        let name = input.trim();
        if !valid(name) || !name.ends_with(".tar.gz") {
            self.fail(i18n::text("archive-extension"));
            return Task::none();
        }
        let target = self.location.join(name);
        if self.services.disk.exists(&target) {
            self.fail(i18n::with("already-exists", "name", name.to_owned()));
            return Task::none();
        }
        if self.archive_work.is_some() {
            self.fail(i18n::text("archive-busy"));
            return Task::none();
        }
        self.archive_work = Some(target.clone());
        self.inform(i18n::with("compressing", "count", paths.len().to_string()));
        let archiver = self.services.archiver.clone();
        let finished = target.clone();
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || archiver.create(&paths, &target))
                    .await
                    .map_err(|error| error.to_string())
                    .and_then(|result| result.map_err(|error| error.to_string()))
            },
            move |result| Message::ArchiveCompleted(finished.clone(), result),
        )
    }

    pub fn archive_completed(&mut self, target: PathBuf, result: Result<(), String>) {
        if self.archive_work.as_ref() != Some(&target) {
            return;
        }
        self.archive_work = None;
        match result {
            Ok(()) => {
                if target.parent() == Some(self.location.as_path()) {
                    self.refresh();
                    if let Some(name) = target.file_name().and_then(|name| name.to_str()) {
                        self.listing.select_name(name);
                    }
                }
                self.inform(i18n::with(
                    "archive-created",
                    "path",
                    target.display().to_string(),
                ));
            }
            Err(error) => self.fail(error),
        }
    }
}
