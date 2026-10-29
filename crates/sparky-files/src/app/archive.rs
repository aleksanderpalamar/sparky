use std::path::PathBuf;

use iced::Task;

use crate::app::message::Message;
use crate::app::state::{Files, Mode};
use crate::domain::naming::valid;

impl Files {
    pub fn begin_archive(&mut self, paths: Vec<PathBuf>) -> Option<usize> {
        if paths.is_empty() {
            return None;
        }
        if self.archive_work.is_some() {
            self.fail("Já há uma compactação em andamento");
            return None;
        }
        let stem = match paths.as_slice() {
            [only] => only
                .file_name()
                .map(|name| name.to_string_lossy().into_owned()),
            _ => None,
        }
        .unwrap_or_else(|| "arquivos".to_owned());
        self.input = format!("{stem}.tar.gz");
        self.mode = Mode::Compress(paths);
        Some(stem.chars().count())
    }

    pub fn start_archive(&mut self, paths: Vec<PathBuf>, input: &str) -> Task<Message> {
        let name = input.trim();
        if !valid(name) || !name.ends_with(".tar.gz") {
            self.fail("Use um nome de arquivo terminado em .tar.gz");
            return Task::none();
        }
        let target = self.location.join(name);
        if self.services.disk.exists(&target) {
            self.fail(format!("Já existe um item chamado {name}"));
            return Task::none();
        }
        if self.archive_work.is_some() {
            self.fail("Já há uma compactação em andamento");
            return Task::none();
        }
        self.archive_work = Some(target.clone());
        self.inform(format!("Compactando {}...", paths.len()));
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
                self.inform(format!("Arquivo criado: {}", target.display()));
            }
            Err(error) => self.fail(error),
        }
    }
}
